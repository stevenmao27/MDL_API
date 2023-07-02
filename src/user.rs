use std::error::Error;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use crate::{story::{SystemTitle, Chapter}, timestamp};

const USERS_PATH: &str = "./public/users";

#[derive(Serialize, Deserialize, Debug)]
pub struct UserTitle {
    pub id: u32, // consistent with library id
    pub title: String,
    pub last_read: String, // YYYY-MM-DD
    pub last_chap: u32, // chapter's ID
    pub url: String,
    pub chapters: Vec<Chapter>, // chapter's ID == index
}

impl UserTitle {
    pub fn into_systemtitle(self) -> SystemTitle {
        SystemTitle {
            id: self.id,
            title: self.title,
            last_updated: String::new(),
            url: self.url,
            chapters: self.chapters,
        }
    }

    pub fn set_timestamp(&mut self) {
        self.last_read = timestamp::get_time();
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String, // identification for now
    pub titles: Vec<UserTitle>,
}

impl User {
    pub async fn new(name: &str) -> Result<User, Box<dyn Error>> {
        let mut file = File::open(format!("{}/{}.json", USERS_PATH, name)).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await.unwrap();

        let user: User = serde_json::from_str(&content).expect("Failed to deserialize user. Has the user been tampered with?");
        Ok(user)
    }

    pub async fn create_user(name: &str) -> User {
        User {
            name: name.to_string(),
            titles: Vec::new(),
        }
    }

    pub async fn save_user(&self) -> Result<(), Box<dyn Error>> {
        let string = serde_json::to_string(self)?;
        let mut user_file = File::create(format!("{}/{}.json", USERS_PATH, self.name)).await?;
        user_file.write_all(string.as_bytes()).await?;

        Ok(())
    }

    pub fn has_title(&self, url: &str) -> bool {
        self.titles.iter().any(|title| title.url == url)
    }

    pub fn add_title(&mut self, title: &SystemTitle) {
        if !self.has_title(&title.url) {
            let new_title = title.clone().into_usertitle();
            self.titles.push(new_title);
        } else {
            println!("User {} already has title {}", self.name, title.title);
        }
    }

    pub fn remove_title(&mut self, title_id: u32) {
        self.titles.retain(|title| title.id != title_id);
    }
}