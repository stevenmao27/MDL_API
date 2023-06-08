use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};

pub async fn create_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://mangadex.org/"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
    Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

pub async fn scout_title(url: &str) -> (String, String, Vec<(String, String)>) {
    // Scrape Chapter URLs
    let client = create_client().await;
    let response = client.get(url).send().await.unwrap();

    let body = response.text().await.unwrap();
    let document = Html::parse_document(&body);

    let link_selector = Selector::parse(".row-content-chapter > li > a").unwrap();
    let title_selector = Selector::parse(".story-info-right > h1").unwrap();
    let cover_selector = Selector::parse(".info-image > .img-loading").unwrap();

    let mut links = document
        .select(&link_selector)
        .map(|link| (link.text().collect::<String>(), link.value().attr("href").unwrap().to_string()) )
        .collect::<Vec<(String, String)>>();
        // .collect::<Vec<scraper::ElementRef>>();
    links.reverse();
    
    let title = document
        .select(&title_selector)
        .next()
        .unwrap()
        .text()
        .collect::<String>();

    let cover_url = document
        .select(&cover_selector)
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap();

    (title, cover_url.to_string(), links)
}

pub async fn download_chapter(chapter_dir: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {

    // Simply Multithreads download_image
    let client = create_client().await;
    let response = client.get(url).send().await.unwrap();
    let body = response.text().await.unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse(".container-chapter-reader > img").unwrap();

    // Each thread runs download_image_and_save()
    let mut threads = Vec::new();
    for (i, element) in document.select(&selector).enumerate() {
        let client_clone = client.clone();
        let src = element.value().attr("src").unwrap().to_string();
        let path = format!("{}/{}.png", chapter_dir, i);

        threads.push(tokio::spawn(download_image_and_save(client_clone, src, path)));
    }

    // Wait for all threads to finish
    for thread in threads {
        thread.await?;
    }
    Ok(())
}

// Downloads image and saves it to path
use tokio::{fs::File, io::AsyncWriteExt};
async fn download_image_and_save(client: Client, url: String, path: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = client.get(&url).send().await?;
    let bytes = response.bytes().await?;

    let mut file = File::create(path).await?;
    file.write_all(&bytes).await?;

    Ok(())
}