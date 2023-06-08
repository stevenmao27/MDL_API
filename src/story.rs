use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub id: u32,
    pub title: String,
    pub updated: String,
    pub url: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    pub id: u32,
    pub description: String,
    pub url: String,
}

impl Title {
    pub fn new(id: u32, title: String, updated: String, url: String, chapters: Vec<Chapter>) -> Title {
        Title {
            id,
            title,
            updated,
            url,
            chapters,
        }
    }
}

impl Chapter {
    pub fn new(id: u32, description: String, url: String) -> Chapter {
        Chapter {
            id,
            description,
            url,
        }
    }
}