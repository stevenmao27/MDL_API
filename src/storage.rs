use axum::body::Bytes;
use tokio::{
    fs::{create_dir, remove_dir_all, File},
    io::{AsyncWriteExt, AsyncReadExt, ErrorKind},
};
type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const TITLE_PATH: &str = "./public/titles";
// pub const USER_PATH: &str = "./public/users";
pub const COVER_PATH: &str = "./public/covers";

// ALL FILE/DIR MUST BE INTEGERS
pub async fn read_directory_names(path: &str) -> Vec<u32> {
    let mut contents = Vec::new();
    let mut directory = tokio::fs::read_dir(path).await.unwrap();
    
    while let Some(entry) = directory.next_entry().await.unwrap() {
        contents.push(entry.file_name().into_string().unwrap().parse::<u32>().unwrap());
    }

    contents
}

pub async fn open_json(path: &str) -> Res<String> {
    let mut file = File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await.unwrap();
    Ok(content)
}

pub async fn save_json(path: &str, content: &str) {
    let mut file = File::create(path).await.unwrap();
    file.write_all(content.as_bytes()).await.unwrap();
}

pub async fn setup_title(id: &u32) {
    // Create Folder
    if let Err(e) = create_dir(format!("{}/{}", TITLE_PATH, id)).await {
        if e.kind() != ErrorKind::AlreadyExists {
            panic!("storage::setup_title failed. Error: {}", e);
        }
    }
}

pub async fn remove_title(id: &u32) {
    // Remove Folder
    if let Err(e) = remove_dir_all(format!("{}/{}", TITLE_PATH, id)).await {
        if e.kind() == ErrorKind::NotFound {
            println!("Folder id = {id} not found.")
        } else {
            panic!("storage::remove_title failed. Error: {}", e);
        }
    }
}

pub async fn save_cover(id: u32, cover: Bytes) {
    // Save Cover
    let mut cover_file = File::create(format!("{COVER_PATH}/{id}.jpeg")).await.unwrap();
    cover_file.write_all(&cover).await.unwrap();
}

pub async fn clear_title(id: &u32) {
    // Delete all chapters
    match tokio::fs::read_dir(format!("{}/{}", TITLE_PATH, id)).await {
        Ok(mut directory) => {
            while let Some(entry) = directory.next_entry().await.unwrap() {
                if entry.file_type().await.unwrap().is_dir() {
                    remove_dir_all(entry.path()).await.unwrap();
                }
            }
        },
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("Folder id = {id} not found.")
            } else {
                panic!("storage::clear_title failed. Error: {}", e);
            }
        }
    }
}

pub async fn setup_chapter(title_id: &u32, chapter_id: &u32) {
    // Create Folder
    if let Err(e) = create_dir(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await {
        if e.kind() != ErrorKind::AlreadyExists {
            panic!("storage::setup_chapter failed. Error: {}", e);
        }
    }
}

pub async fn delete_chapter(title_id: &u32, chapter_id: &u32) {
    // Remove Folder
    if let Err(e) = remove_dir_all(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await {
        if e.kind() == ErrorKind::NotFound {
            println!("Folder id = {title_id} not found.")
        } else {
            panic!("storage::delete_chapter failed. Error: {}", e);
        }
    }
}

pub async fn get_num_images(title_id: u32, chapter_id: u32) -> u32 {
    let mut num_images = 0;
    match tokio::fs::read_dir(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await {
        Ok(mut directory) => {
            while let Some(entry) = directory.next_entry().await.unwrap() {
                if entry.file_type().await.unwrap().is_file() {
                    num_images += 1;
                }
            }
        },
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("Folder id = {title_id} not found.")
            } else {
                panic!("storage::get_num_images failed. Error: {}", e);
            }
        }
    }
    num_images
}

pub async fn get_chapters(title_id: u32) -> Vec<u32> {
    let mut chapters = Vec::new();
    match tokio::fs::read_dir(format!("{}/{}", TITLE_PATH, title_id)).await {
        Ok(mut directory) => {
            while let Some(entry) = directory.next_entry().await.unwrap() {
                if entry.file_type().await.unwrap().is_dir() {
                    chapters.push(entry.file_name().into_string().unwrap().parse::<u32>().unwrap());
                }
            }
        },
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("Folder id = {title_id} not found.")
            } else {
                panic!("storage::get_chapters failed. Error: {}", e);
            }
        }
    }
    chapters
}