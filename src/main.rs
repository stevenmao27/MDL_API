#![allow(non_snake_case)]
use axum::{
    extract::Query,
    response::{IntoResponse, Json},
    routing::get,
    routing::post,
    Router,
};

mod story;
mod timestamp;
mod user;
mod library;
mod storage;
mod web;
mod latency;

use std::io::{self, Write};
#[tokio::main]
async fn main() {
    // // build our application with a single route
    // let app = Router::new()
    //     .route("/add_title", get(add_title))
    //     .route("/download_chapter", get(download_chapter));

    // // run it with hyper on localhost:3000
    // axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    println!("--- Testing MD_API ---");
    loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                let user = library::add_title("https://chapmanganato.com/manga-iw985379".to_string()).await;
            }
            "2" => {
                library::remove_title("https://chapmanganato.com/manga-iw985379".to_string()).await.unwrap();
            }
            "3" => {
                let id = 1;
                web::download_chapter(&id, "https://chapmanganato.com/manga-iw985379/chapter-103").await;
            }
            _ => {
                println!("Invalid input");
                use std::process::exit;
                exit(0);
            }
        }
    }
}