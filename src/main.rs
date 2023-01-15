use std::{sync::Arc, time::Duration};

// this pulls in some of functions
use askama::Template;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};

use futures::{sink::SinkExt, stream::StreamExt};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

mod can;
mod servo_controller;

use servo_controller::{ServoController, ServoState};



// use futures_util::{future, StreamExt, TryStreamExt};
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

async fn home() -> HomeTemplate {
    HomeTemplate {}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum ControllerInput {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum CommandInput {
    // inputs passed to the controller for movement etc.
    Servo(ControllerInput),
    Disconnect,
}

// handle_socket currently expects to only be called once per client
// if called twice without the client disconnecting things could get wonky.
async fn handle_socket(socket: WebSocket, state: Arc<Mutex<AppState>>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // this spins up a "background worker"
    // that will take messages from the websocket and push them
    // into a queue so that the Controller is able to process
    // the messages asyncorniously
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(msg) = msg {
                match tx.send(msg.clone()).await {
                    Ok(_) => {}
                    Err(e) => println!("Failed to send message: {:?}", e),
                }
            }
        }
    });

    // This is the main update loop for the controller.
    // basically this gets called every so often and allows
    // for the controller to respond to events coming from
    // the arduino, rather than relying on messages sent from the client.
    loop {
        let s = state.lock().await.update();
        let response_json = serde_json::to_string(&s).unwrap();
        sender
            .send(Message::Text(response_json.to_string()))
            .await
            .unwrap();

        let f = rx.try_recv();
        tokio::time::sleep(Duration::from_millis(100)).await;
        match f {
            Ok(e) => {
                let command: Result<CommandInput, serde_json::Error> = serde_json::from_str(&e);
                if let Ok(c) = command {
                    let response: StateResponse = state.lock().await.handle_command(c);
                    match response {
                        StateResponse::ServoState(s) => {
                            let response_json = serde_json::to_string(&s).unwrap();
                            sender
                                .send(Message::Text(response_json.to_string()))
                                .await
                                .unwrap();
                        }
                        StateResponse::None => {}
                    }
                }
            }
            Err(_) => {
                // println!("No message currently");
            }
        }
    }
}

// messages that are sent back to the client.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum StateResponse {
    ServoState(Vec<ServoState>),
    None,
}

/// sort of global state that is shared amoungst all the api end point.  
struct AppState {
    controller: ServoController,
    client_connected: bool,
}

impl AppState {
    pub fn new(controller: ServoController) -> Self {
        Self {
            controller,
            client_connected: false,
        }
    }

    /// Update function that should be called at a specified rate.
    // used for updating the client or getting the current state of stuff.
    pub fn update(&mut self) -> StateResponse {
        self.controller.update().unwrap();

        // report out servo states.
        let servo_states = self.controller.get_servo_state().to_vec();
        StateResponse::ServoState(servo_states)
    }

    pub fn handle_command(&mut self, msg: CommandInput) -> StateResponse {
        match msg {
            CommandInput::Servo(command) => {
                println!("Command: {:?}", command);
                self.controller.handle_command(command).unwrap();
            }
            CommandInput::Disconnect => self.client_connected = false,
        }
        StateResponse::None
    }
}

async fn websocket_test(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    ws: WebSocketUpgrade,
) -> Response {
    println!("Blah blah websocket test");
    if state.lock().await.client_connected {
        (StatusCode::CONFLICT, String::from("already connected")).into_response()
    } else {
        state.lock().await.client_connected = true;
        ws.on_upgrade(|socket| handle_socket(socket, state))
            .into_response()
    }
}

fn app(state: Arc<Mutex<AppState>>) -> Router
{
    Router::new()
        .route("/", get(home))
        .route("/ws", get(websocket_test))
        .layer(Extension(state))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = can::WindowsCANHandle::open(0).unwrap();

    let state = Arc::new(Mutex::new(AppState::new(
                                    ServoController::new(Box::new(handle), 2))));

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();


    let app = app(state);
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::can::MockHandle;

    use super::*;

    use std::net::{Ipv4Addr, SocketAddr};

    use can::CANHandle;
    use tokio_tungstenite::tungstenite;


    // test for ensuring that only a single controller connection
    // is allowed. 
    #[tokio::test]
    async fn single_controller_connection() {

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let state = Arc::new(Mutex::new(AppState::new(
            ServoController::new(Box::new(MockHandle::open(0).unwrap()), 2))));
        let app = app(state);

        let server = axum::Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .serve(app.into_make_service());
        let addr = server.local_addr();
        tokio::spawn(server);

        println!("Connecting to: {}", format!("ws://{addr}/ws"));
        let (mut socket, _response) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .unwrap();

        tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .expect_err("Failed to connect");

        // socket.send(tungstenite::Message::text("foo")).await.unwrap();
    }

    #[test]
    fn test_command_serde() {
        let f = CommandInput::Servo(ControllerInput::Left);
        let expected_output = r#"{"t":"Servo","c":{"t":"Left"}}"#;
        let k = serde_json::to_string(&f).unwrap();
        assert_eq!(k, expected_output);

        let f = CommandInput::Servo(ControllerInput::Right);
        let expected_output = r#"{"t":"Servo","c":{"t":"Right"}}"#;
        let k = serde_json::to_string(&f).unwrap();
        assert_eq!(k, expected_output);
    }
}
