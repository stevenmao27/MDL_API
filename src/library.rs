use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashSet;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use crate::{
    storage,
    storage::*,
    web,
    story::{SystemTitle, Chapter},
    timestamp::{get_time, self},
    latency::Latency,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    titles: Vec<SystemTitle>,
}

impl Library {
    pub async fn new() -> Result<Library, Box<dyn Error + Send + Sync>> {
        let mut file = File::open(format!("{}/library.json", TITLE_PATH)).await?;

        let mut content = String::new();
        file.read_to_string(&mut content).await.unwrap();

        let library: Library = serde_json::from_str(&content)?;
        Ok(library)
    }

    pub async fn save(&self) {
        let string = serde_json::to_string(&self).unwrap();
        let mut library_file = File::create(format!("{}/library.json", TITLE_PATH))
            .await
            .unwrap();
        library_file.write_all(string.as_bytes()).await.unwrap();
    }

    pub async fn add_title(&mut self, url: &str) -> Result<SystemTitle, Box<dyn Error>> {

        // check if title already exists
        if let Some(title) = self.get_title_by_url(url) {
            println!("Title already exists: {}", title.title);
            return Ok(title.clone());
        }

        // scrape basic details
        let (title_name, cover_url, links) = web::scout_title(url).await?;
        let title = SystemTitle {
            id: self.get_new_id(),
            title: title_name,
            last_updated: get_time(),
            url: url.to_string(),
            chapters: links.into_iter()
                .map(|element| Chapter {
                    desc: element.0,
                    url: element.1,
                    // sufx: element.1.split('/').last().unwrap().to_string(),
                    })
                .collect::<Vec<Chapter>>(),
        };

        // configure storage
        // - create title folder
        // - add to library.JSON
        // - download cover image
        
        storage::setup_title(&title.id).await;

        let img_response = web::create_client().await.get(cover_url).send().await?;
        storage::save_cover(&title.id, img_response).await;

        self.titles.push(title.clone());
        self.save().await;

        Ok(title)
    }

    pub async fn remove_title(&mut self, id: u32) -> Result<(), Box<dyn Error>> {
        match self.get_title_by_id(id) {
            Some(title) => {
                storage::remove_title(&title.id).await;
                self.titles.retain(|title| title.id != id);
                self.save().await;
            }
            None => { println!("Title does not exist: {}", id); }
        };
        Ok(())
    }

    pub async fn update_title(&mut self, id: u32) -> Result<(), Box<dyn Error>> {
        match self.titles.iter_mut().find(|title| title.id == id) {
            Some(mut title) => {
                let links = web::get_chapters(&title.url).await?;
                title.last_updated = get_time();
                title.chapters = links.into_iter()
                    .map(|element| Chapter {
                        desc: element.0,
                        url: element.1,
                        // sufx: element.1.split('/').last().unwrap().to_string(),
                        })
                    .collect::<Vec<Chapter>>();
                self.save().await;
                
                Ok(())
            }
            None => {
                println!("Title does not exist: {}", id);
                Err("Title does not exist".into())
            }
        }
    }

    pub async fn add_chapter(&self, title_id: &u32, chapter_id: &u32) -> Result<u32, Box<dyn Error>> {

        // what if title doesn't exist?
        // note: we assume chapter id exists
        match self.get_title_by_id(*title_id) {
            Some(title) => {
                match storage::setup_chapter(title_id, chapter_id).await {
                    StorageResult::Success => {
                        let chapter_url = title.chapters.get(*chapter_id as usize).unwrap().url.as_str();
                        web::download_chapter(&format!("{TITLE_PATH}/{title_id}/{chapter_id}"), chapter_url).await.unwrap();
                    }
                    StorageResult::AlreadyExists => {
                        println!("Chapter already downloaded: {}/{}", title_id, chapter_id);
                    }
                }
                Ok(storage::get_num_images(*title_id, *chapter_id).await)
            }
            None => {
                println!("Title does not exist: {}", title_id);
                Err("Title does not exist".into())
            }
        }
    }

    pub async fn remove_chapter(&self, title_id: &u32, chapter_id: &u32) -> Result<(), Box<dyn Error>> {
        match self.get_title_by_id(*title_id) {
            Some(_title) => {
                storage::delete_chapter(title_id, chapter_id).await;
                Ok(())
            }
            None => {
                println!("Title does not exist: {}", title_id);
                Err("Title does not exist".into())
            }
        }
    }

    pub async fn cleanup(&mut self) {
        // TODO: delete old titles (assume multiple users)
        // delete old chapters
        let MAX_CHAPTERS: usize = 5;
        
        for title in &self.titles {
            let days_since_updated = timestamp::get_duration(title.last_updated.clone(), timestamp::get_time());
            let mut chapters = storage::get_chapters(title.id).await;

            if days_since_updated >= 3 {
                storage::clear_title(&title.id).await;
            }
            else if chapters.len() > MAX_CHAPTERS {
                chapters.sort();
                for i in 0..(chapters.len() - MAX_CHAPTERS) {
                    storage::delete_chapter(&title.id, &chapters[i]).await;
                }
            }
        }
    }

    fn get_title_by_id(&self, id: u32) -> Option<&SystemTitle> {
        self.titles.iter().find(|title| title.id == id)
    }
    fn get_title_by_url(&self, url: &str) -> Option<&SystemTitle> {
        self.titles.iter().find(|title| title.url == url)
    }
    fn get_new_id(&self) -> u32 {
        let ids = self.titles.iter().map(|title| title.id).collect::<Vec<u32>>();
        let ids_set: HashSet<u32> = HashSet::from_iter(ids);
        let mut new_id: u32 = 0;
        while ids_set.contains(&new_id) {
            new_id += 1;
        }
        new_id
    }
}
