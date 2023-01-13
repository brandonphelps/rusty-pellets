use askama::Template;
use axum::{Router, routing::get};



#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
}


async fn home() -> HomeTemplate {
    HomeTemplate { }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let app = Router::new()
        .route("/", get(home));


    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
