use std::time::Duration;

use tokio::{fs::{File, self}, io::AsyncReadExt, sync::RwLock, signal};
use axum::{
    extract::{Query, Path},
    response::Json,
    routing::{get, post},
    Router,
    body::Body,
    http::StatusCode,
};
// use axum_macros::debug_handler;
use serde::Deserialize;
use tower_http::cors;

mod library;
mod user;
mod timestamp;
mod web;
mod storage;
mod latency;

// use library::*;
use user::*;

static UPDATING_CHAPTERS: RwLock<bool> = RwLock::const_new(false);
const MAX_AGE_SECONDS: u64 = 60 * 30; // 30m

#[tokio::main]
async fn main() {
    // cleanup loop
    tokio::spawn(async {
        loop {
            clean().await;
            tokio::time::sleep(Duration::from_secs(MAX_AGE_SECONDS)).await;
        }
    });

    // CORS setup
    let cors = cors::CorsLayer::permissive();
    // build our application with a single router
    let app = Router::new()
    
    // account endpoints
    .route("/register", post(register_handler))
    .route("/login", post(login_handler))
    .route("/save_user", post(save_user_handler))
    
    // image-related endpoints
    .route("/cover/:title_id", get(cover_handler))
    .route("/img/:title_id/:chapter_id/:image_id", get(image_request))
    .route("/image_sources", get(srcs_handler))
    .route("/proxy", get(proxy_handler))
    
    // title-related endpoints
    .route("/new_title", post(new_title_handler))
    .route("/remove_title", post(remove_title_handler))
    .route("/download_chapter", post(download_chapter_handler))
    .route("/update_title", post(update_title_handler))

    .layer(cors);
    // run it with hyper on localhost:3000
    let server = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service());

    tokio::select! {
        _ = server => { println!("FATAL: Server crashed"); },
        _ = signal::ctrl_c() => {
            println!("Received Ctrl-C SIG, saving and shutting down server...");
        },
    }
}

async fn clean() {
    // scan titles and delete unused ones
    let mut title_dirs = fs::read_dir("./public/titles").await.unwrap();
    while let Some(dir) = title_dirs.next_entry().await.unwrap() {
        if dir.metadata().await.unwrap().modified().unwrap().elapsed().unwrap().as_secs() > MAX_AGE_SECONDS {
            fs::remove_dir_all(dir.path()).await.unwrap();
        }
    }
}

#[derive(Deserialize)]
struct RegisterBody {
    username: String,
    password: String,
    action: String,
}
async fn register_handler(Json(RegisterBody { username, password, action}): Json<RegisterBody>) -> StatusCode {
    match action.as_str() {
        "register" => {
            let Some(user) = User::new(username, password).await else { return StatusCode::BAD_REQUEST; };
            user.save_to_disk().await.unwrap();
        }
        "unregister" => {
            let Some(user) = User::from(&username).await else { return StatusCode::BAD_REQUEST; };
            if user.password == password {
                user.delete_from_disk().await.unwrap();
            } else {
                return StatusCode::BAD_REQUEST;
            }
        }
        _ => {}
    }
    StatusCode::OK
}


#[derive(Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}
async fn login_handler(Json(LoginBody { username, password }): Json<LoginBody>) -> Json<User> {
    if let Some(user) = User::from(&username).await {
        if user.password == password {
            return Json(user);
        } else { return Json(User::empty_with_message("Wrong Password".to_string())); }
    }

    Json(User::empty_with_message("User Does Not Exist".to_string()))
}


async fn cover_handler(Path(title_id): Path<u32>) -> axum::http::Response<Body> {
    if let Ok(mut file) = File::open(format!("./public/covers/{}.jpeg", title_id)).await {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();
        let body = Body::from(buf);
        return axum::http::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "image/jpeg")
            .body(body).unwrap();
    }
    axum::http::Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()
}


#[derive(Deserialize)]
struct ImageUrlsQuery {
    chapter_url: String
}
async fn srcs_handler(Query(query): Query<ImageUrlsQuery>) -> Json<Vec<String>> {
    Json(web::get_images_src(&query.chapter_url).await.unwrap())
}


#[derive(Deserialize)]
struct ProxyQuery {
    url: String
}
async fn proxy_handler(Query(query): Query<ProxyQuery>) -> axum::http::Response<Body> {
    let client = web::create_client().await;
    let data = client.get(&query.url).send().await.unwrap().bytes().await.unwrap();

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(data))
        .unwrap();

    response
}


#[derive(Deserialize)]
struct NewTitleBody {
    username: String,
    url: String,
}
async fn new_title_handler(Json(NewTitleBody { username, url }): Json<NewTitleBody>) -> Json<user::User> {
    let Some(mut user) = User::from(&username).await else {
        return Json(User::empty_with_message("User Does Not Exist".to_string()));
    };

    // ? What if another User has this title?

    if !user.has_title_url(&url) {
        let web_result = web::extract_title(&url).await;
        let web::WebResult {
            title,
            chap_prefix,
            last_updated,
            chapters,
            cover
        } = web_result;

        // Save Details to User
        let new_title_id = user.add_title(title, url, chap_prefix, last_updated, chapters).unwrap();

        // Save Cover to Disk
        storage::save_cover(new_title_id, cover).await;
        
        // Save User
        user.save_to_disk().await.unwrap();
    }

    Json(user)
}



#[derive(Deserialize)]
struct RemoveTitleBody {
    username: String,
    id: u32,
}
async fn remove_title_handler(Json(RemoveTitleBody { username, id }): Json<RemoveTitleBody>) -> StatusCode  {
    let Some(mut user) = User::from(&username).await else {
        return StatusCode::NOT_FOUND;
    };
    user.remove_title(id);
    user.save_to_disk().await.unwrap();
    StatusCode::OK  
}


#[derive(Deserialize)]
struct DownloadChapterBody {
    title_id: u32,
    chapter_id: u32,
    url: String,
}
async fn download_chapter_handler(Json(DownloadChapterBody { title_id, chapter_id, url}): Json<DownloadChapterBody>) -> StatusCode {
    storage::setup_title(&title_id).await;
    storage::setup_chapter(&title_id, &chapter_id).await;
    if let Ok(()) = web::download_chapter(&format!("./public/titles/{title_id}/{chapter_id}"), &url).await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}


#[derive(Deserialize)]
struct UpdateChaptersBody {
    username: String,
    title_id: u32,
}
async fn update_title_handler(Json(UpdateChaptersBody { username, title_id }): Json<UpdateChaptersBody>) -> StatusCode {
    let Some(mut user) = User::from(&username).await else {
        return StatusCode::NOT_FOUND;
    };
    let title_ref: &mut Title = user.titles.iter_mut().find(|t| t.id == title_id).unwrap();
    
    if let Some(()) = web::update_title(title_ref).await {
        user.save_to_disk().await.unwrap();
    }

    StatusCode::OK
}


async fn image_request(Path((title_id, chapter_id, image_id)): Path<(u32, u32, u32)>) -> axum::http::Response<Body> {
    if let Ok(mut file) = File::open(format!("./public/titles/{title_id}/{chapter_id}/{image_id}.jpeg")).await {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();

        let body = Body::from(buf);
        let response = axum::http::Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "image/jpeg")
            .body(body).unwrap();

        return response;
    }

    axum::http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty()).unwrap()
}


async fn save_user_handler(Json(user): Json<user::User>) -> StatusCode {
    user.save_to_disk().await.unwrap();
    StatusCode::OK
}