use std::error::Error;
use axum::body::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};
use futures::future::join_all;
use crate::{latency::{Latency, self}, user::{Chapter, Title}, timestamp};

type Res<T> = Result<T, Box<dyn Error + Send + Sync>>;


pub async fn create_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://manganato.com/"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
    Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

pub struct WebResult {
    pub title: String,
    pub chap_prefix: String,
    pub last_updated: String,
    pub chapters: Vec<Chapter>,
    pub cover: Bytes,
}
/// Heavy and Expensive function. Scrapes:
/// - Basic Details and URLs
/// - Number of images per chapter
/// - Cover Image Data
pub async fn extract_title(url: &str) -> WebResult {
    // Persistent Variables
    let mut timer = Latency::new("extract_title");
    let title: String;
    let chap_prefix: String;
    let last_updated: String;

    let cover_url: String;
    let client = create_client().await;
    let mut links: Vec<(String, String)>;
    let handles: Vec<tokio::task::JoinHandle<u32>>;
    
    // Contain !Send Types (Html, Selector)
    {
        let response = client.get(url).send().await.unwrap();
        timer.tick("got page HTML");

        let body = response.text().await.unwrap();
        let document = Html::parse_document(&body);

        let title_selector = Selector::parse(".story-info-right > h1").unwrap();
        let cover_selector = Selector::parse(".info-image > .img-loading").unwrap();
        let link_selector = Selector::parse(".row-content-chapter > li > a").unwrap();
        let date_released_selector = Selector::parse(".row-content-chapter > li > span").unwrap();

        title = document.select(&title_selector).next().unwrap()
            .text().collect::<String>();

        cover_url = document.select(&cover_selector).next().unwrap()
            .value().attr("src").unwrap().to_string();

        let most_recent_date = document.select(&date_released_selector).nth(1).unwrap()
            .text().collect::<String>();
        last_updated = timestamp::get_nelo_time(&most_recent_date);

        // Get Chapter URLs and Description --- Extract Prefix/Suffix
        // Ex. https://manganato.com/manga-ai118410/chapter-1 
        // --> chap_prefix = "https://manganato.com/manga-ai118410/"
        // --> s (or suffix) = "chapter-1"
        links = document.select(&link_selector)
            .map(|link| 
                (link.text().collect::<String>(), link.value().attr("href").unwrap().to_string())
            ).collect();
        links.reverse(); // 3,2,1 -> 1,2,3

        chap_prefix = links.get(0).unwrap().1.rsplit_once("/").unwrap().0.to_string() + "/";

        // Get Num Images per Chapter
        handles = links.iter().map(|tuple|
            tokio::spawn(get_num_images(client.clone(), tuple.1.clone()))
        ).collect();
        timer.tick("done scraping HTML");
    }

    // Download Cover
    let cover_bytes: Bytes = client.get(cover_url).send().await.unwrap()
        .bytes().await.unwrap();
    timer.tick("done downloading cover image");

    // Multithread Scout Chapter Img Count
    let results: Vec<Result<u32, JoinError>> = join_all(handles).await;
    timer.tick("all threads finished scouting chapter image count");
    let mut chapters: Vec<Chapter> = Vec::new();
    for (i, (text, url)) in links.into_iter().enumerate() {
        chapters.push(Chapter {
            t: text,
            s: url.rsplit_once("/").unwrap().1.to_string(),
            i: results.get(i).unwrap().as_ref().unwrap().clone(),
        });
    }

    WebResult {
        title,
        chap_prefix,
        last_updated,
        chapters,
        cover: cover_bytes,
    }
}

// Updates title directly and returns None if no new chapters
pub async fn update_title(title: &mut Title) -> Option<()> {
    let mut latency = Latency::new("update_title");
    let client = create_client().await;
    let response = client.get(&title.url).send().await.unwrap();
    latency.tick("got page HTML");

    let mut links: Vec<(String, String)>;
    let most_recent_date: String;
    // get new data
    {
        let document = Html::parse_document(&response.text().await.unwrap());
        let link_selector = Selector::parse(".row-content-chapter > li > a").unwrap();
        let date_released_selector = Selector::parse(".row-content-chapter > li > span").unwrap();

        links = document.select(&link_selector)
            .map(|link| 
                (link.text().collect::<String>(), link.value().attr("href").unwrap().to_string())
            ).collect();
        links.reverse();

        most_recent_date = document.select(&date_released_selector).nth(1).unwrap()
            .text().collect::<String>();
    }

    // update title
    title.last_scanned = timestamp::get_nelo_time(&most_recent_date);

    if links.len() == title.chapters.len() {
        return None;
    }

    title.last_updated = title.last_scanned.clone();

    for (i, (text, url)) in links.into_iter().enumerate() {
        if i >= title.chapters.len() {
            title.chapters.push(Chapter {
                t: text,
                s: url.rsplit_once("/").unwrap().1.to_string(),
                i: get_num_images(client.clone(), url).await,
            });
        }
    }

    Some(())
}

async fn get_num_images(client: Client, url: String) -> u32 {
    let response = client.get(url).send().await.unwrap();
    let body = response.text().await.unwrap();
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".container-chapter-reader > img").unwrap();
    document.select(&selector).count() as u32
}

pub async fn get_images_src(chapter_url: &str) -> Res<Vec<String>> {
    let client = create_client().await;
    let response = client.get(chapter_url).send().await?;
    let document = Html::parse_document(&response.text().await?);
    let css_selector = Selector::parse(".container-chapter-reader > img").unwrap();
    let images_src = document.select(&css_selector)
            .map(|element| element.value().attr("src").unwrap().to_string())
            .collect();

    Ok(images_src)
}

pub async fn download_chapter(chapter_dir: &str, url: &str) -> Res<()> {
    
    let mut threads = Vec::new();
    let mut timer = Latency::new("download_chapter");
    {
        // Multithreads download_image
        let client = create_client().await;
        let response = client.get(url).send().await?;
        let body = response.text().await.unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse(".container-chapter-reader > img").unwrap();

        // Each thread runs download_image_and_save()
        for (i, element) in document.select(&selector).enumerate() {
            let client_clone = client.clone();
            let src = element.value().attr("src").unwrap().to_string();
            let path = format!("{}/{}.jpeg", chapter_dir, i);

            threads.push(tokio::spawn(download_image_and_save(client_clone, src, path)));
        }
    }


    // Wait for all threads to finish
    for thread in threads {
        thread.await.unwrap().unwrap();
    }
    timer.tick("done downloading + saving all images");
    Ok(())
}

// Downloads image and saves it to path
use tokio::{fs::File, io::AsyncWriteExt, task::JoinError};
async fn download_image_and_save(client: Client, url: String, path: String) -> Res<()> {
    let response = client.get(&url).send().await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(path.clone()).await?;
    file.write_all(&bytes).await?;

    Ok(())
}