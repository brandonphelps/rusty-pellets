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
    Extension, Router, body::{Full, Bytes},
};
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

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // this is done so that we can continue doing processing
    // from the server controller without waiting for messages
    // to be sent from the client.
    tokio::spawn(async move {
        while let Some(Ok(msg)) = socket.recv().await {
            if let Message::Text(msg) = msg {

                tx.send(msg.clone()).await.unwrap();

                if socket
                    .send(Message::Text(format!("You said: {msg}")))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    println!("waiting at recv q");
    loop {
        let f = rx.try_recv();
        println!(" got= {:?}", f);
        tokio::time::sleep(Duration::from_secs(1)).await;
        match f {
            Ok(e) => {
                if e.contains("e") {
                    state.lock().await.handle_message(&e);
                    break;
                }
            }
            Err(_) => {
                println!("No message currently");
            }
        }
    }
}

struct AppState {
    controller: TestController,
    client_connected: bool,
}

impl AppState {
    pub fn handle_message(&mut self, msg: &str) {
        println!("AppState got a message: {}", msg);
        self.client_connected = false;
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
