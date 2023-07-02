#![allow(non_snake_case)]
#![allow(unused_imports)]
#![warn(dead_code)]
use std::error::Error;
use tokio::{fs::File, io::AsyncReadExt};
use axum::{
    extract::{Query, Path},
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
    body::Body,
    http::StatusCode,
};
// use axum_macros::debug_handler;
use serde::Deserialize;

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
        .route("/new_title", post(new_title_handler))
        .route("/remove_title", post(remove_title_handler))
        .route("/download_chapter", post(download_chapter_handler))
        .route("/img/:title_id/:chapter_id/:image_id", get(image_request));

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
/// 
/// - fetches user's data from ./public/users
async fn load_handler(Query(LoadQuery {username}): Query<LoadQuery>) -> Json<user::User> {
    let user = User::new(&username).await.unwrap();
    Json(user)
}

#[derive(Deserialize)]
struct NewTitleQuery {
    username: String,
    url: String,
}
/// # Adding new title for user
/// 
/// - ERROR HANDLING
///     - title already exist in user profile
/// - if title doesn't exist in library, create it
/// - add title to user profile
async fn new_title_handler(Json(NewTitleQuery { username, url }): Json<NewTitleQuery>) -> Json<user::User> {
    let mut user = User::new(&username).await.unwrap();

    if !user.has_title(&url) {
        let title = Library::new().await.unwrap().add_title(&url).await.unwrap();
        user.add_title(&title);
        user.save_user().await.unwrap();
    }

    Json(user)
}

#[derive(Deserialize)]
struct RemoveTitleQuery {
    username: String,
    id: u32,
}
/// # Removing title from user
/// 
/// - ERROR HANDLING
///    - title doesn't exist in user profile
/// - remove title from user profile
/// - if title doesn't exist in any other user profile, remove it from library
async fn remove_title_handler(Json(RemoveTitleQuery { username, id }): Json<RemoveTitleQuery>) -> StatusCode  {
    let mut user = User::new(&username).await.unwrap();
    user.remove_title(id);
    user.save_user().await.unwrap();

    // ! ASSUMES NO OTHER USER RIGHT NOW
    Library::new().await.unwrap().remove_title(id).await.unwrap();

    StatusCode::OK
}

#[derive(Deserialize)]
struct DownloadChapterQuery {
    title_id: u32,
    chapter_id: u32,
}
/// # Request chapter download
/// 
/// - ERROR HANDLING
///     - title doesn't exist
///     - chapter already downloaded
/// - request chapter download
/// - return OK
async fn download_chapter_handler(Json(DownloadChapterQuery { title_id, chapter_id }): Json<DownloadChapterQuery>) -> StatusCode {
    Library::new().await.unwrap().add_chapter(&title_id, &chapter_id).await.unwrap();
    StatusCode::OK
}

/// # Image request
/// 
/// - ERROR HANDLING
///     - path doesn't exist
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