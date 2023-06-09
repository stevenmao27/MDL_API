use reqwest::Response;
use tokio::{
    fs::{create_dir, remove_dir_all, File},
    io::{AsyncWriteExt, ErrorKind},
};

pub const TITLE_PATH: &str = "./public/titles";

pub async fn setup_title(id: &u32) {
    // Create Folder
    if let Err(e) = create_dir(format!("{}/{}", TITLE_PATH, id)).await {
        if e.kind() == ErrorKind::AlreadyExists {
            println!("Folder id = {id} already exists.")
        } else {
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

pub async fn save_cover(id: &u32, cover: Response) {
    // Save Cover
    let mut cover_file = File::create(format!("{}/{}/cover.jpg", TITLE_PATH, id)).await.unwrap();
    cover_file.write_all(&cover.bytes().await.unwrap()).await.unwrap();
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

pub async fn setup_chapter(title_id: &u32, chapter_id: &u32) -> StorageResult {
    // Create Folder
    if let Err(e) = create_dir(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await {
        if e.kind() == ErrorKind::AlreadyExists {
            println!("Folder id = {title_id} already exists.");
            return StorageResult::AlreadyExists;
        } else {
            panic!("storage::setup_chapter failed. Error: {}", e);
        }
    }

    StorageResult::Done
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

pub enum StorageResult {
    Done,
    AlreadyExists,
    NotFound,
    UnknownError,
}