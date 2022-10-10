use once_cell::sync::Lazy;
use scraper::{Html, Selector};

use super::{generic, Media, ScrapErr, CLIENT};

#[tokio::test]
async fn download_media() {
    scrape("https://danbooru.donmai.us/posts/5634057").await.unwrap();
}

static ORIGINAL_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".image-view-original-link[href]").unwrap());

pub async fn scrape(url: &str) -> Result<Media, ScrapErr> {
    let html = CLIENT.get(url).send().await?;
    let html = html.text().await?;
    let html = Html::parse_document(&html);
    let link = html
        .select(&ORIGINAL_SELECTOR)
        .next()
        .ok_or(ScrapErr::HtmlScraping {
            element: "html.a".to_string(),
            scraper: "danbooru".to_string(),
        })?;
    let media_url = link.value().attr("href").ok_or(ScrapErr::HtmlScraping {
        element: "a.href".to_string(),
        scraper: "danbooru".to_string(),
    })?;
    let media = generic::scrape(media_url).await?;
    Ok(media)
}
