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

use futures::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};

use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;



// use futures_util::{future, StreamExt, TryStreamExt};
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

async fn home() -> HomeTemplate {
    HomeTemplate {}
}

enum ControllerError {}

struct TestController {}

impl TestController {
    pub fn up(&mut self) -> Result<(), ControllerError> {
        println!("up");
        Ok(())
    }

    pub fn down(&mut self) -> Result<(), ControllerError> {
        println!("down");
        Ok(())
    }
    pub fn left(&mut self) -> Result<(), ControllerError> {
        println!("left");
        Ok(())
    }

    pub fn right(&mut self) -> Result<(), ControllerError> {
        println!("right");
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="t", content="c")]
enum ControllerInput {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="t", content="c")]
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
                    Ok(_) => { },
                    Err(e) => println!("Failed to send message: {:?}", e)
                }
            }
        }
    });


    // This is the main update loop for the controller.
    // basically this gets called every so often and allows
    // for the controller to respond to events coming from
    // the arduino, rather than relying on messages sent from the client. 
    loop {
        let f = rx.try_recv();
        tokio::time::sleep(Duration::from_millis(100)).await;
        match f {
            Ok(e) => {
                let command: Result<CommandInput, serde_json::Error> = serde_json::from_str(&e);
                if let Ok(c) = command {
                    let response = state.lock().await.handle_command(c);
                    sender.send(Message::Text(response)).await.unwrap();
                }
            },
            Err(_) => {
                // println!("No message currently");
            }
        }
    }
}

struct AppState {
    controller: TestController,
    client_connected: bool,
}

impl AppState {

    /// Update function that should be called at a specified rate. 
    // used for updating the client or getting the current state of stuff. 
    pub fn update(&mut self) -> String {
        todo!()
    }

    pub fn handle_command(&mut self, msg: CommandInput) -> String {
        match msg {
            CommandInput::Servo(command) => {
                println!("Command: {:?}", command);
            },
            CommandInput::Disconnect => self.client_connected = false
        }
        "hello".into()
    }
}

async fn websocket_test(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    ws: WebSocketUpgrade,
) -> Response {
    println!("upgrade {:?}", ws);

    // let r = Response::builder()
    //     .status(StatusCode::CONFLICT)
    //     .body(BoxBody::Full::from("not found")).unwrap();
    if state.lock().await.client_connected {
        (StatusCode::CONFLICT, String::from("already connected")).into_response()
    } else { 
        state.lock().await.client_connected = true;
        ws.on_upgrade(|socket| handle_socket(socket, state)).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(AppState {
        controller: TestController {},
        client_connected: false,
    }));

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // // construct a subscriber that prints formatted traces to stdout
    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // // use that subscriber to process traces emitted after this point
    // tracing::subscriber::set_global_default(subscriber)?;
    let app = Router::new()
        .route("/", get(home))
        .route("/ws", get(websocket_test))
        .layer(Extension(state));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}


#[cfg(test)]
mod tests {

    use super::*;

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
