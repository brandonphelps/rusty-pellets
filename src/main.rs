use askama::Template;
use axum::{Router, extract::{Query, WebSocketUpgrade, ws::{Message, WebSocket}}, response::Response, routing::get};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::{collections::HashMap, net::SocketAddr};

use tokio::net::{TcpListener, TcpStream};

// use futures_util::{future, StreamExt, TryStreamExt};


#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
}


async fn home() -> HomeTemplate {
    HomeTemplate { }
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(msg) = msg {
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


async fn websocket_test(ws: WebSocketUpgrade, Query(query): Query<HashMap<String, String>>) -> Response {
    println!("Got websocket upgrade request: {:?}", query);
    ws.on_upgrade(handle_socket)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // // construct a subscriber that prints formatted traces to stdout
    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // // use that subscriber to process traces emitted after this point
    // tracing::subscriber::set_global_default(subscriber)?;

    println!("Hello, world!");

    let app = Router::new()
        .route("/", get(home))
        .route("/soc", get(websocket_test));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
