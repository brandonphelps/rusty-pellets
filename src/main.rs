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

use futures::{
    sink::SinkExt,
    stream::StreamExt,
};


use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

mod can;

use can::CANMessage;

// use futures_util::{future, StreamExt, TryStreamExt};
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

async fn home() -> HomeTemplate {
    HomeTemplate {}
}

struct ServoState {
}

enum ControllerError {}

struct TestController {
    handle: can::CANHandle,
}

impl TestController {
    pub fn up(&mut self) -> Result<(), ControllerError> {
        println!("up");
        let response = self.handle.write(&CANMessage::new(0x200, &[0x0], false));
        Ok(())
    }

    pub fn down(&mut self) -> Result<(), ControllerError> {
        println!("down");
        let response = self.handle.write(&CANMessage::new(0x200, &[0x3], false));
        Ok(())
    }
    pub fn left(&mut self) -> Result<(), ControllerError> {
        println!("left");
        let response = self.handle.write(&CANMessage::new(0x200, &[0x2], false));
        Ok(())
    }

    pub fn right(&mut self) -> Result<(), ControllerError> {
        println!("right");
        let response = self.handle.write(&CANMessage::new(0x200, &[0x1], false));
        Ok(())
    }

    pub fn handle_command(&mut self, command: ControllerInput) -> Result<(), ControllerError> {
        match command {
            ControllerInput::Left => self.left(),
            ControllerInput::Right => self.right(),
            ControllerInput::Up => self.up(),
            ControllerInput::Down => self.down(),
        }
    }

    // 
    pub fn update(&mut self) -> Result<(), ControllerError> {
        // read in can message and handle incoming can messages. 
        if let Ok(Some(msg)) = self.handle.read() {
            println!("Got a message: {:?}", msg);
        }
        Ok(())
    }

    pub fn get_servo_state(&self) -> Result<Vec<ServoState>, ControllerError> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum ControllerInput {
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
        state.lock().await.update();
        let f = rx.try_recv();
        tokio::time::sleep(Duration::from_millis(100)).await;
        match f {
            Ok(e) => {
                let command: Result<CommandInput, serde_json::Error> = serde_json::from_str(&e);
                if let Ok(c) = command {
                    let response = state.lock().await.handle_command(c);
                    sender.send(Message::Text(response)).await.unwrap();
                }
            }
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
        self.controller.update();
        "update".into()
    }

    pub fn handle_command(&mut self, msg: CommandInput) -> String {
        match msg {
            CommandInput::Servo(command) => {
                println!("Command: {:?}", command);
                self.controller.handle_command(command);
            }
            CommandInput::Disconnect => self.client_connected = false,
        }
        "hello".into()
    }
}

async fn websocket_test(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    ws: WebSocketUpgrade,
) -> Response {
    if state.lock().await.client_connected {
        (StatusCode::CONFLICT, String::from("already connected")).into_response()
    } else {
        state.lock().await.client_connected = true;
        ws.on_upgrade(|socket| handle_socket(socket, state))
            .into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = can::CANHandle::open(0).unwrap();

    let state = Arc::new(Mutex::new(AppState {
        controller: TestController { handle },
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
