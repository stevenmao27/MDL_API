use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use crate::story::{Title};

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

pub async fn save_user(user: User) {
    let string = serde_json::to_string(&user).unwrap();
    let mut user_file = File::create(format!("{}/{}.json", USER_PATH, user.name))
        .await
        .unwrap();
    user_file.write_all(string.as_bytes()).await.unwrap();
}
