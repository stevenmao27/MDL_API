use std::{error::Error, collections::{HashSet, HashMap}, hash::Hash};
use serde::{Deserialize, Serialize};
use tokio::fs;
use crate::{storage, timestamp::get_time};

const USERS_PATH: &str = "./public/users";
type Res<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub password: String, // doubles as error message
    pub titles: Vec<Title>,
    pub tags: HashMap<String, Vec<u32>> // tag_name -> [title ids]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Title {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub chap_prefix: String, // "...com/"
    pub last_chap: u32,
    pub last_updated: String, // Actual Release Date
    pub last_read: String, // User Read Date
    pub last_scanned: String, // When Axum scanned
    pub tags: Vec<String>,
    pub chapters: Vec<Chapter>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chapter {
    pub t: String, // text description
    pub s: String, // suffix "chapter-1"
    pub i: u32, // number of images
}

#[derive(Serialize, Deserialize, Debug)]
struct DB {
    users: HashMap<String, u32>,
    titles: HashMap<String, u32>,
}

impl DB {
    async fn new() -> DB {
        let json_str = storage::open_json("./public/db.json").await.unwrap();
        let db = serde_json::from_str::<DB>(&json_str).unwrap();
        db
    }
    async fn save(&self) {
        let json_str = serde_json::to_string(self).unwrap();
        storage::save_json("./public/db.json", &json_str).await;
    }
}

impl User {
    pub fn empty_with_message(message: String) -> User {
        User {
            id: 0,
            username: String::new(),
            password: message,
            titles: Vec::new(),
            tags: HashMap::new(),
        }
    }

    // register new user instance
    pub async fn new(username: String, password: String) -> Option<User> {
        // check if username already exists
        let mut db = DB::new().await;
        if db.users.contains_key(&username) {
            println!("Username already exists: {}", username);
            return None;
        }

        // find suitable ID
        let user_ids = storage::read_directory_names(USERS_PATH).await;
        let set: HashSet<u32> = user_ids.into_iter().collect();
        let id = (0..).find(|i| !set.contains(i)).unwrap();

        // add to db
        db.users.insert(username.clone(), id);
        db.save().await;

        Some(User {
            id,
            username,
            password,
            titles: Vec::new(),
            tags: HashMap::new(),
        })
    }

    // load existing user from disk
    pub async fn from(name: &str) -> Option<User> {
        // check db
        let db = DB::new().await;
        let Some(id) = db.users.get(name) else { return None; };

        let content = storage::open_json(&format!("{}/{}.json", USERS_PATH, id)).await.unwrap();
        let user: User = serde_json::from_str(&content).unwrap();
        Some(user)
    }

    pub async fn save_to_disk(&self) -> Res<()> {
        let string = serde_json::to_string(self)?;
        storage::save_json(&format!("{USERS_PATH}/{}.json", self.id), &string).await;
        Ok(())
    }

    pub async fn delete_from_disk(&self) -> Res<()> {
        // Check if user exists in DB and remove
        let mut db = DB::new().await;
        let Some(user_id) = db.users.remove(&self.username) else {return Ok(());};
        db.save().await;

        // Delete user.json if possible
        fs::remove_file(&format!("{USERS_PATH}/{user_id}.json")).await?;

        Ok(())
    }

    pub fn add_title(&mut self, name: String, url: String, chap_prefix: String, last_updated: String, chapters: Vec<Chapter>) -> Res<u32> {

        // generate new ID
        let title_ids = self.titles.iter().map(|title| title.id).collect::<HashSet<u32>>();
        let new_title_id = (0..).find(|i| !title_ids.contains(i)).unwrap();

        self.titles.push(Title {
            id: new_title_id,
            name,
            url,
            chap_prefix,
            last_chap: 0,
            last_updated,
            last_read: get_time(),
            last_scanned: get_time(),
            tags: Vec::new(),
            chapters,
        });

        Ok(new_title_id)
    }

    pub fn has_title_url(&mut self, url: &str) -> bool {
        self.titles.iter().any(|title| url == title.url)
    }

    pub fn remove_title(&mut self, id: u32) {
        self.titles.retain(|title| title.id != id);
    }
}