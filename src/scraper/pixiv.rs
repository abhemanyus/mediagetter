use std::{collections::HashMap, env::temp_dir};

use once_cell::sync::Lazy;
use reqwest::header::REFERER;
use scraper::{Html, Selector};
use serde::Deserialize;
use uuid::Uuid;

use super::{Media, ScrapErr, Type, CLIENT};

#[tokio::test]
async fn download_media() {
    let url = "https://www.pixiv.net/en/artworks/101779848";
    scrape(url).await.unwrap();
}

static CONTENT_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#meta-preload-data[content]").unwrap());

pub async fn scrape(url: &str) -> Result<Media, ScrapErr> {
    let content = {
        let html = CLIENT.get(url).send().await?.text().await?;
        let html = Html::parse_document(&html);
        let content = html
            .select(&CONTENT_SELECTOR)
            .next()
            .ok_or(ScrapErr::HtmlScraping {
                element: "metadata".to_string(),
                scraper: "pixiv".to_string(),
            })?;
        let content = content
            .value()
            .attr("content")
            .ok_or(ScrapErr::HtmlScraping {
                element: "metadata.content".to_string(),
                scraper: "pixiv".to_string(),
            })?;
        let content: Content = serde_json::from_str(content)?;
        content
    };
    let illust = content
        .illust
        .values()
        .next()
        .ok_or(ScrapErr::HtmlScraping {
            element: "metadata.content.illust".to_string(),
            scraper: "pixiv.content".to_string(),
        })?;
    let url = &illust.urls.original;
    let img = CLIENT
        .get(url)
        .header(REFERER, "https://www.pixiv.net")
        .send()
        .await?;
    let img = img.bytes().await?;
    let path = temp_dir().join(Uuid::new_v4().to_string());
    let _ = tokio::fs::write(&path, img).await?;
    Ok(Media {
        kind: Type::Image,
        location: path,
    })
}

#[derive(Deserialize)]
struct Content {
    illust: HashMap<String, Illust>,
}

#[derive(Deserialize)]
struct Illust {
    urls: Urls,
}

#[derive(Deserialize)]
struct Urls {
    original: String,
}
