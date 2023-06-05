#![allow(non_snake_case)]
use axum::{
    extract::Query,
    response::{IntoResponse, Json},
    routing::get,
    routing::post,
    Router,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/add_title", get(add_title))
        .route("/download_chapter", get(download_chapter));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    titles: Vec<UserTitle>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserTitle {
    id: u32,
    title: String,
    cover_url: String,
    current_chapter: u32,
    chapters: Vec<UserChapter>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserChapter {
    id: u32,
    text: String,
}

async fn add_title(_url: String) -> Json<User> {
    // Scrape Chapter URLs
    // Create Folder + Metadata
    // (optional) Download Chapters
    // Add Title to User JSON and return it
    let user = load_user().await.unwrap();

    Json(user)
}

async fn load_user() -> Result<User, tokio::io::Error> {
    let mut file = File::open("./public/users/steven.json")
        .await
        .expect("Could not open steven.json");
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    Ok(serde_json::from_str(&content).expect("Could not parse steven.json"))
}

#[derive(Deserialize, Debug)]
struct DownloadRequest {
    url: String,
    folder: String,
}

async fn download_chapter(q: Query<DownloadRequest>) -> axum::response::Html<String> {
    let DownloadRequest { url, folder } = q.0;

    // Set up client
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_str(&url.to_string()).expect("Bad chapter URL given"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let mut threads = Vec::new();
    {
        // Parse HTML for image URLs
        let response = client.get(&url).send().await.unwrap();
        let body = response.text().await.unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse(".container-chapter-reader > img").unwrap();

        // Download images
        for (i, element) in document.select(&selector).enumerate() {
            let img_url = element.value().attr("src").unwrap().to_string();
            let folder = folder.clone();
            let client_clone = client.clone();
            let thread = tokio::spawn(download_image(client_clone, img_url, folder, i));
            threads.push(thread);
        }
    }

    // Wait for all threads to finish
    for thread in threads {
        thread.await;
    }

    axum::response::Html("Download Complete!".to_string())
}

async fn download_image(
    client: Client,
    url: String,
    folder: String,
    eee: usize,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut response = client.get(&url).send().await.unwrap();
    let mut file = File::create(format!("{}/{}.png", folder, eee)).await.unwrap();
    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).await.unwrap();
    }
    Ok("Saved to file".to_string())
}
