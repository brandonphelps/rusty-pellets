use std::sync::Arc;

// this pulls in some of functions 
use askama::Template;
use axum::{Router, extract::{WebSocketUpgrade, ws::{Message, WebSocket}, State}, response::Response, routing::get, Extension};
use tokio::sync::Mutex;




// use futures_util::{future, StreamExt, TryStreamExt};
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
}

async fn home() -> HomeTemplate {
    HomeTemplate { }
}

enum ControllerError {
}

struct TestController {
    
}

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
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(msg) = msg {
            state.lock().await.handle_message(&msg);

            if socket
                .send(Message::Text(format!("You said: {msg}")))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}

struct AppState {
    controller: TestController,
}

impl AppState {
    pub fn handle_message(&self, msg: &str) {
        println!("AppState got a message: {}", msg);
    }
}


async fn websocket_test(Extension(state): Extension<Arc<Mutex<AppState>>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(AppState {
        controller: TestController { }
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
