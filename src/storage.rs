use reqwest::Response;
use tokio::{
    fs::{create_dir, remove_dir_all, File},
    io::{AsyncWriteExt, ErrorKind, Error},
};

pub const TITLE_PATH: &str = "./public/titles";

pub async fn setup_title(id: &u32) {
    // Create Folder
    create_dir(format!("{}/{}", TITLE_PATH, id))
        .await
        .unwrap();
}

pub async fn remove_title(id: &u32) -> Result<(), String> {
    // Remove Folder
    if let Err(_) = remove_dir_all(format!("{}/{}", TITLE_PATH, id)).await {
        println!("Title id = {} not found", id);
        return Err(format!("Title id = {} not found", id));
    }

    Ok(())
}

pub async fn save_cover(id: &u32, cover: Response) {
    // Save Cover
    let mut cover_file = File::create(format!("{}/{}/cover.jpg", TITLE_PATH, id))
        .await
        .unwrap();
    cover_file.write_all(&cover.bytes().await.unwrap()).await.unwrap();
}

pub async fn clear_title(id: &u32) {
    // Delete all chapters
    let mut directory = tokio::fs::read_dir(format!("{}/{}", TITLE_PATH, id))
        .await
        .unwrap();

    while let Some(entry) = directory.next_entry().await.unwrap() {
        if entry.file_type().await.unwrap().is_dir() {
            remove_dir_all(entry.path()).await.unwrap();
        }
    }
}

pub async fn setup_chapter(title_id: &u32, chapter_id: &u32) {
    // Create Folder
    let _ = create_dir(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await;
}

pub async fn delete_chapter(title_id: &u32, chapter_id: &u32) {
    // Remove Folder
    let _ = remove_dir_all(format!("{}/{}/{}", TITLE_PATH, title_id, chapter_id)).await;
}