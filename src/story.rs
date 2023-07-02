use serde::{Deserialize, Serialize};
use crate::user::UserTitle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemTitle {
    pub id: u32,
    pub title: String,
    pub last_updated: String,
    pub url: String,
    pub chapters: Vec<Chapter>,
}

impl SystemTitle {
    pub fn into_usertitle(self) -> UserTitle {
        UserTitle {
            id: self.id,
            title: self.title,
            last_read: String::new(),
            last_chap: 0,
            url: self.url,
            chapters: self.chapters,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    pub desc: String,
    pub url: String,
}