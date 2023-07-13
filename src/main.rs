#![allow(non_snake_case)]
#![allow(unused_imports)]
#![warn(dead_code)]
use std::error::Error;
use tokio::{fs::File, io::AsyncReadExt};
use axum::{
    extract::{Query, Path},
    response::{Json, IntoResponse, Response},
    routing::{get, post},
    Router,
    body::Body,
    http::{StatusCode, HeaderValue},
};
// use axum_macros::debug_handler;
use serde::Deserialize;
use tower_http;

mod library;
mod user;
mod story;
mod timestamp;
mod web;
mod storage;
mod latency;

use library::*;
use user::*;


#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
    .route("/load", get(load_handler))
    .route("/img/:title_id/:chapter_id/:image_id", get(image_request))
    .route("/cover/:title_id", get(cover_request))

    .route("/new_title", post(new_title_handler))
    .route("/remove_title", post(remove_title_handler))
    .route("/download_chapter", post(download_chapter_handler))
    .route("/update_title", post(update_title_handler))
    .route("/save_user", post(save_user_handler))

    .layer(tower_http::cors::CorsLayer::new().allow_origin("*".parse::<HeaderValue>().unwrap()));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}



#[derive(Deserialize)]
struct LoadQuery {
    username: String
}
/// Loading page data
/// - fetches user's data from ./public/users
async fn load_handler(Query(LoadQuery {username}): Query<LoadQuery>) -> Json<user::User> {
    let user = User::new(&username).await.unwrap();
    
    Json(user)
}



#[derive(Deserialize)]
struct NewTitleBody {
    username: String,
    url: String,
}
/// # Adding new title for user
/// - if title doesn't exist in library, create it
/// - add title to user profile
async fn new_title_handler(Json(NewTitleBody { username, url }): Json<NewTitleBody>) -> Json<user::User> {
    let mut user = User::new(&username).await.unwrap();

    if !user.has_title(&url) {
        let title = Library::new().await.unwrap().add_title(&url).await.unwrap();
        user.add_title(&title);
        user.save_user().await.unwrap();
    }

    Json(user)
}



#[derive(Deserialize)]
struct RemoveTitleBody {
    username: String,
    id: u32,
}
/// # Removing title from user
/// - if title doesn't exist in any other user profile, remove it from library
async fn remove_title_handler(Json(RemoveTitleBody { username, id }): Json<RemoveTitleBody>) -> StatusCode  {
    let mut user = User::new(&username).await.unwrap();
    user.remove_title(id);
    user.save_user().await.unwrap();

    // ! ASSUMES NO OTHER USER RIGHT NOW
    Library::new().await.unwrap().remove_title(id).await.unwrap();

    StatusCode::OK
}



#[derive(Deserialize)]
struct DownloadChapterBody {
    title_id: u32,
    chapter_id: u32,
}
/// # Request chapter download
/// - request chapter download
async fn download_chapter_handler(Json(DownloadChapterBody { title_id, chapter_id }): Json<DownloadChapterBody>) -> String {
    Library::new().await.unwrap().add_chapter(&title_id, &chapter_id).await.unwrap().to_string()
}


#[derive(Deserialize)]
struct UpdateChaptersBody {
    title_id: u32,
}
async fn update_title_handler(Json(UpdateChaptersBody { title_id }): Json<UpdateChaptersBody>) -> StatusCode {
    Library::new().await.unwrap().update_title(title_id).await.unwrap();
    StatusCode::OK
}

/// # Image request
/// - read buffer and send it to client
async fn image_request(Path((title_id, chapter_id, image_id)): Path<(u32, u32, u32)>) -> axum::http::Response<Body> {
    if let Ok(mut file) = File::open(format!("./public/titles/{}/{}/{}.jpg", title_id, chapter_id, image_id)).await {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();

        let body = Body::from(buf);
        let response = axum::http::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "image/png")
            .body(body)
            .unwrap();

        return response;
    }

    axum::http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}


/// # Cover request
/// - read buffer and send it to client
async fn cover_request(Path(title_id): Path<u32>) -> axum::http::Response<Body> {
    if let Ok(mut file) = File::open(format!("./public/titles/{}/cover.jpg", title_id)).await {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();

        let body = Body::from(buf);
        let response = axum::http::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "image/png")
            .body(body)
            .unwrap();

        return response;
    }

    axum::http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}


async fn save_user_handler(Json(user): Json<user::User>) -> StatusCode {
    user.save_user().await.unwrap();
    StatusCode::OK
}