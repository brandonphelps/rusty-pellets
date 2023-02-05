use std::{sync::Arc, time::Duration};

use kinect_control::freenect_sys::{
    freenect_device, freenect_led_options_LED_BLINK_GREEN, freenect_set_depth_callback,
    freenect_set_video_callback,
};

use kinect_control::FreenectContext;
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

use axum_extra::routing::SpaRouter;

use futures::{sink::SinkExt, stream::StreamExt};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

mod can;
mod servo_controller;

use servo_controller::{ServoController, ServoState};

// use futures_util::{future, StreamExt, TryStreamExt};
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> IndexTemplate {
    IndexTemplate {}
}

#[derive(Template)]
#[template(path = "tictactoe.html")]
struct TicTacToeTemplate {}
async fn tictactoe() -> TicTacToeTemplate {
    TicTacToeTemplate {}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControllerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
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
                    Err(e) => {
                        println!("Failed to send message: {:?}", e);
                        // when the reciver end has closed then we just terminate.
                        break;
                    }
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
        if let Err(_e) = sender.send(Message::Text(response_json.to_string())).await {
            eprintln!("error while sending, halting");
            break;
        } else {
            println!("Sent servo state as a response");
        }

        let f = rx.try_recv();
        tokio::time::sleep(Duration::from_millis(100)).await;
        match f {
            Ok(input_string) => {
                // todo change to log debug.
                println!("Got an input string: {:?}", input_string);
                if let Ok(command) = serde_json::from_str::<CommandInput>(&input_string) {
                    let response = state.lock().await.handle_command(command);
                    println!("Got command from server");
                    let response_json = serde_json::to_string(&response).unwrap();

                    sender.send(Message::Text(response_json.to_string())).await;

                    if let StateResponse::Disconnect = response {
                        println!("Disconnecting");
                        break;
                    }
                } else {
                    println!("failed to deserialize input");
                }
            }
            // do nothing if we don't receive anything.
            Err(Empty) => {}
            Err(f) => {
                println!("error occured: {:?}", f);
            }
        }
    }

    state.lock().await.client_connected = false;
}

// messages that are sent back to the client.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum StateResponse {
    ServoState(Vec<ServoState>),
    // if provided marks a successful disconnect.
    Disconnect,
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
                self.controller.handle_command(command).unwrap();
                StateResponse::None
            }
            CommandInput::Disconnect => {
                self.client_connected = false;
                StateResponse::Disconnect
            }
        }
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

// width = 640
// height = 480
// data bits per pixel 24
// padding 0
// frame rate 30

const RGB_WIDTH: usize = 640;
const RGB_HEIGHT: usize = 480;
const RGB_BYTES_COUNT: usize = (RGB_WIDTH * RGB_HEIGHT * 3);

// let bytes = 921600;
// let width = 640;
// let height = 480;

extern "C" fn video_cb(
    dev: *mut freenect_device,
    data: *mut ::std::os::raw::c_void,
    timestamp: u32,
) {
    unsafe {
        let rgb_data: *mut u8 = data as *mut _;
        let mut back_data = &mut back_rgb.lock().unwrap();
        // todo: should really be able to do memcpy
        let mut i = 0;
        // todo: is this the fastest way to copy data? 
        //for i in 0..RGB_BYTES_COUNT {
        while i < RGB_BYTES_COUNT {
            back_data[i] = *(rgb_data.add(i));
            back_data[i+1] = *(rgb_data.add(i+1));
            back_data[i+2] = *(rgb_data.add(i+2));
            i += 3;
        }
    };
}

async fn image_handle_socket(mut socket: WebSocket, state: Arc<Mutex<AppState>>) {
    println!("image handle socket got opened");
    #[derive(Serialize, Deserialize)]
    struct tmp {
        width: usize,
        height: usize,
        data: Vec<u8>,
    }

    loop {
        let f = {
            let back_d: &[u8] = &back_rgb.lock().unwrap();
            tmp {
                width: RGB_WIDTH,
                height: RGB_HEIGHT,
                data: back_d[..RGB_BYTES_COUNT].to_vec(),
            }
        };

        println!("Sending data");
        // todo: see if we can remove Text and use Binary.
        let send_ret = socket
            .send(Message::Text(serde_json::to_string(&f).unwrap()))
            .await
            .unwrap();
        println!("Send ret: {:?}", send_ret);

        println!("Finished processing events");
        tokio::time::sleep(std::time::Duration::from_millis(125)).await;
    }
}

async fn websocket_image_data(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| image_handle_socket(socket, state))
        .into_response()
}

fn app(state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/tictactoe", get(tictactoe))
        .route("/ws", get(websocket_test))
        .route("/ws_image", get(websocket_image_data))
        .layer(Extension(state))
        .merge(SpaRouter::new("/static", "static_gen"))
}

use once_cell::sync::Lazy;

// static back_rgb: Lazy<std::sync::Mutex<[u8; 640 * 480 * 4]>> = Lazy::new(|| std::sync::Mutex::new([0u8; 640 * 480 * 4]));
// static front_rgb: Lazy<std::sync::Mutex<[u8; 640 * 480 * 4]>> = Lazy::new(|| std::sync::Mutex::new([0u8; 640 * 480 * 4]));

static back_rgb: Lazy<std::sync::Mutex<Vec<u8>>> =
    Lazy::new(|| std::sync::Mutex::new(Vec::with_capacity(RGB_BYTES_COUNT)));
static front_rgb: Lazy<std::sync::Mutex<Vec<u8>>> =
    Lazy::new(|| std::sync::Mutex::new(Vec::with_capacity(RGB_BYTES_COUNT)));

fn image_process_thread() {
    let kinect_context = FreenectContext::new();
    kinect_context.set_led(freenect_led_options_LED_BLINK_GREEN);

    let mut angle = 0.1;

    let r = kinect_context.set_video_mode();

    println!("Set video mode result: {}", r);

    kinect_context.start_video();

    unsafe { freenect_set_video_callback(kinect_context.dev, Some(video_cb)) }

    loop {
        let r = kinect_context.process_events();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut back_data = &mut back_rgb.lock().unwrap();
        for i in 0..(RGB_BYTES_COUNT) {
            back_data.push(0);
        }
    }

    {
        let mut back_data = &mut front_rgb.lock().unwrap();
        for i in 0..(RGB_BYTES_COUNT) {
            back_data.push(0);
        }
    }

    #[cfg(target_os = "windows")]
    let handle = can::WindowsCANHandle::open(0).unwrap();

    #[cfg(not(target_os = "windows"))]
    let handle = can::MockHandle::open(0).unwrap();

    let state = Arc::new(Mutex::new(AppState::new(ServoController::new(
        Box::new(handle),
        2,
    ))));

    let image_thread = std::thread::spawn(image_process_thread);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = app(state);

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio_tungstenite::tungstenite;

    use crate::can::MockHandle;

    use super::*;

    use std::net::{Ipv4Addr, SocketAddr};

    // test for ensuring that only a single controller connection
    // is allowed.
    #[tokio::test]
    async fn single_controller_connection() {
        let state = Arc::new(Mutex::new(AppState::new(ServoController::new(
            Box::new(MockHandle::open(0).unwrap()),
            2,
        ))));
        let app = app(state);

        let server = axum::Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .serve(app.into_make_service());
        let addr = server.local_addr();
        tokio::spawn(server);
        let (socket, _response) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .unwrap();

        tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .expect_err("Failed to connect");

        // socket.send(tungstenite::Message::text("foo")).await.unwrap();
    }

    #[tokio::test]
    async fn websocket_disconnect() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let state = Arc::new(Mutex::new(AppState::new(ServoController::new(
            Box::new(MockHandle::open(0).unwrap()),
            2,
        ))));

        let local_s = state.clone();

        let app = app(state);

        let server = axum::Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .serve(app.into_make_service());
        let addr = server.local_addr();
        tokio::spawn(server);
        let (mut socket, _response) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .unwrap();

        socket
            .send(tungstenite::Message::text(
                serde_json::to_string(&CommandInput::Disconnect).unwrap(),
            ))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        assert_eq!(local_s.lock().await.client_connected, false);

        // socket.send(tungstenite::Message::text("foo")).await.unwrap();
    }

    #[test]
    fn app_state_update() {
        let mut state = AppState::new(ServoController::new(
            Box::new(MockHandle::open(0).unwrap()),
            2,
        ));
        let update = state.update();
        match update {
            StateResponse::ServoState(f) => {
                assert_eq!(f.len(), 2);
            }
            StateResponse::None => assert!(false),
            StateResponse::Disconnect => assert!(false),
        }
    }

    #[test]
    fn app_state_disconnect() {
        let mut state = AppState::new(ServoController::new(
            Box::new(MockHandle::open(0).unwrap()),
            2,
        ));
        let update = state.handle_command(CommandInput::Disconnect);
        assert_eq!(state.client_connected, false);

        match update {
            StateResponse::ServoState(_) => assert!(false),
            StateResponse::Disconnect => assert!(true),
            StateResponse::None => assert!(false),
        }
        // how to do the equvalent of this without partial eq.
        // assert_eq!(update, StateResponse::Disconnect);
    }

    #[test]
    fn app_state_servo_up() {
        let mut state = AppState::new(ServoController::new(
            Box::new(MockHandle::open(0).unwrap()),
            2,
        ));
        let update = state.handle_command(CommandInput::Servo(ControllerInput {
            up: true,
            down: true,
            left: true,
            right: false,
        }));

        match update {
            StateResponse::ServoState(_) => assert!(false),
            StateResponse::Disconnect => assert!(false),
            StateResponse::None => assert!(true),
        }
        // how to do the equvalent of this without partial eq.
        // assert_eq!(update, StateResponse::Disconnect);
    }

    #[test]
    fn test_command_serde() {
        let f = CommandInput::Servo(ControllerInput {
            up: true,
            down: false,
            left: true,
            right: false,
        });

        let expected_output =
            r#"{"t":"Servo","c":{"up":true,"down":false,"left":true,"right":false}}"#;
        let k = serde_json::to_string(&f).unwrap();
        assert_eq!(k, expected_output);
    }

    #[test]
    fn test_command_disconnect() {
        let f = CommandInput::Disconnect;
        let expected_output = r#"{"t":"Disconnect"}"#;
        let k = serde_json::to_string(&f).unwrap();
        assert_eq!(k, expected_output);
    }
}

// extern "C" fn depth_cb(
//     dev: *mut freenect_device,
//     data: *mut ::std::os::raw::c_void,
//     timestamp: u32,
// ) {
//     println!("depth callback: {}", timestamp);

//     // todo: calculate size based on the various dimensions.

//     let bytes = 921600;
//     let width = 640;
//     let height = 480;

//     unsafe {
//         let rgb_data: &[u8] = std::slice::from_raw_parts(data as *mut u8, bytes);
//         println!(
//             "Some data: ({}, {}, {})",
//             rgb_data[0], rgb_data[1], rgb_data[2]
//         );
//         // save_buffer_with_format("myimg.jpg", rgb_data, width, height, image::ColorType::Rgb8, image::ImageFormat::Jpeg).unwrap();
//     }

//     // The following three lines simply load a test image and convert it into buffer
//     // let (width, height) = (img.width(), img.height());
//     // let img_byte_vec = img.into_raw();
//     // // The next line is what you want
// }
