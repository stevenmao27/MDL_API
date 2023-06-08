use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use crate::{
    storage,
    storage::TITLE_PATH,
    web,
    story::{Title, Chapter},
    timestamp::{get_time},
};

#[derive(Serialize, Deserialize, Debug)]
struct Library {
    titles: Vec<Title>,
}

async fn load_library() -> Library {
    let mut file = File::open(format!("{}/library.json", TITLE_PATH))
        .await
        .expect("Could not open library.json");
    let mut content = String::new();
    file.read_to_string(&mut content).await.unwrap();

    serde_json::from_str(&content).expect("Could not parse library.json")
}

async fn save_library(library: Library) {
    let string = serde_json::to_string(&library).unwrap();
    let mut library_file = File::create(format!("{}/library.json", TITLE_PATH))
        .await
        .unwrap();
    library_file.write_all(string.as_bytes()).await.unwrap();
}

fn get_title_by_id(id: u32, library: &Library) -> Option<Title> {
    library.titles.iter().find(|title| title.id == id).cloned()
}

fn get_title_by_url(url: &str, library: &Library) -> Option<Title> {
    library.titles.iter().find(|title| title.url == url).cloned()
}

fn get_new_id(library: &Library) -> u32 {
    let ids = library.titles.iter().map(|title| title.id).collect::<Vec<u32>>();
    let ids_set: HashSet<u32> = HashSet::from_iter(ids);
    let mut new_id: u32 = 0;
    while ids_set.contains(&new_id) {
        new_id += 1;
    }
    new_id
}

pub async fn add_title(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut library = load_library().await;

    // edge case
    if let Some(title) = get_title_by_url(&url, &library) {
        println!("Title already exists: {}", title.title);
        return Ok(());
    }

    // scrape for details (title, cover, chapters)
    let (title, cover_url, links) = web::scout_title(&url).await;

    let title = Title {
        id: get_new_id(&library),
        title,
        updated: get_time(),
        url: url.clone(),
        chapters: links.into_iter()
            .enumerate()
            .map(|(num, link_element)| Chapter {
                id: (num as u32),
                description: link_element.0,
                url: link_element.1,
            })
            .collect::<Vec<Chapter>>(),
    };

    // Create Title Folder
    storage::setup_title(&title.id).await;

    // Download Cover
    let img_response = web::create_client().await
        .get(cover_url).send().await?;
    storage::save_cover(&title.id, img_response).await;

    // Add Title to Library JSON
    library.titles.push(title);
    save_library(library).await;

    Ok(())
}

pub async fn remove_title(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut library = load_library().await;

    let result = library.titles.iter().position(|title| title.url == url);
    if result.is_none() {
        println!("Title does not exist: {}", url);
        return Ok(());
    }
    let index = result.unwrap();
    let title = &library.titles[index];

    storage::remove_title(&title.id).await;

    // Remove Title from Library JSON
    if index != library.titles.len() - 1 {
        library.titles.swap_remove(index);
    } else {
        library.titles.pop();
    }

    // Write Library to File
    save_library(library).await;

    Ok(())
}

pub async fn add_chapter(title_id: &u32, chapter_id: &u32) -> Result<(), Box<dyn std::error::Error>> {
    storage::setup_chapter(title_id, chapter_id).await;

    let library = load_library().await;
    let title = get_title_by_id(*title_id, &library).unwrap();
    let chapter_url = title.chapters.get(*chapter_id as usize).unwrap().url.clone();
    web::download_chapter(format!("{}/{}", title_id, chapter_id).as_str(), chapter_url.as_str()).await;

    Ok(())
}

pub async fn remove_chapter(title_id: &u32, chapter_id: &u32) -> Result<(), Box<dyn std::error::Error>> {
    storage::delete_chapter(title_id, chapter_id).await;
    Ok(())
}
