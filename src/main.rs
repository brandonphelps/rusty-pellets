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

async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<AppState>>) {
    println!("Do Handle socket stuff");

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // this spins up a "background worker"
    // that will take messages from the websocket and push them
    // into a queue so that the Controller is able to process
    // the messages asyncorniously 
    tokio::spawn(async move {
        println!("Blah: {:?}", receiver);
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(msg) = msg { 
                match tx.send(msg.clone()).await {
                    Ok(_) => { },
                    Err(e) => println!("Failed to send message: {:?}", e)
                }
            }
        }

        // while let Some(Ok(msg)) = receiver.recv().await {
        //     if let Message::Text(msg) = msg {
        //         tx.send(msg.clone()).await.unwrap();
        //         // if socket
        //         //     .send(Message::Text(format!("You said: {msg}")))
        //         //     .await
        //         //     .is_err()
        //         // {
        //         //     break;
        //         // }
        //     }
        // }
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
                println!("got: {}", e);
                let response = state.lock().await.handle_message(&e);
                println!("Exiting from loop");
                sender.send(Message::Text(response)).await.unwrap();
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
    pub fn update(&mut self) -> String {
        todo!()
    }

    pub fn handle_message(&mut self, msg: &str) -> String {
        println!("AppState got a message: {}", msg);
        self.client_connected = false;
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
