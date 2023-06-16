use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use crate::{story::Title, timestamp};

const USER_PATH: &str = "./public/users";

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub titles: Vec<(TitleHistory, Title)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TitleHistory {
    pub id: u32,
    pub last_read: String,
}

pub async fn load_user(name: &str) -> User {
    let mut file = File::open(format!("{}/{}.json", USER_PATH, name))
        .await
        .expect(format!("Could not open {}.json", name).as_str());
    let mut content = String::new();
    file.read_to_string(&mut content).await.unwrap();

    serde_json::from_str(&content).expect(format!("Could not parse {}.json", name).as_str())
}

// pub async fn save_user(user: User) {
//     let string = serde_json::to_string(&user).unwrap();
//     let mut user_file = File::create(format!("{}/{}.json", USER_PATH, user.name))
//         .await
//         .unwrap();
//     user_file.write_all(string.as_bytes()).await.unwrap();
// }

impl User {
    pub async fn save_user(&self) -> Result<(), Box<dyn std::error::Error>> {
        let string = serde_json::to_string(self).unwrap();
        let mut user_file = File::create(format!("{}/{}.json", USER_PATH, self.name))
            .await
            .unwrap();
        user_file.write_all(string.as_bytes()).await.unwrap();

        Ok(())
    }

    pub async fn title_exists(&self, url: &str) -> bool {
        for (_, title) in &self.titles {
            if title.url == url {
                return true;
            }
        }

        false
    }

    pub async fn add_title(&mut self, title: Title) {
        self.titles.push((TitleHistory { id: 1, last_read: timestamp::get_time() }, title));
    }

    pub async fn remove_title(&mut self, title_id: u32) {
        self.titles.retain(|title| title.1.id != title_id);
    }
}