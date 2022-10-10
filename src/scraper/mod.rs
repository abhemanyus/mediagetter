use once_cell::sync::Lazy;
use reqwest::{Client, Url};
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

mod danbooru;
mod generic;
mod pixiv;
mod twitter;

#[tokio::test]
async fn download_media() {
    scrape(
        "https://mobile.twitter.com/dog_rates/status/1578145895584911362?s=20&t=7PAtwWBDwS8wjFeisuawMg",
    )
    .await
    .unwrap();
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn scrape(url: &str) -> Result<Media, ScrapErr> {
    let parse_url = Url::parse(url)?;
    match parse_url
        .domain()
        .ok_or(ScrapErr::NoHost(url.to_string()))?
    {
        "danbooru.donmai.com" => danbooru::scrape(url).await,
        "pixiv.net" | "www.pixiv.net" => pixiv::scrape(url).await,
        "mobile.twitter.com" | "www.twitter.com" => twitter::scrape(url).await,
        _ => generic::scrape(url).await,
    }
}

#[derive(Debug)]
pub enum Type {
    Image,
    Video,
}

#[derive(Debug)]
pub struct Media {
    pub location: PathBuf,
    pub kind: Type,
}

#[derive(Error, Debug)]
pub enum ScrapErr {
    #[error("fetch request failed")]
    Request(#[from] reqwest::Error),
    #[error("could not scrape {element:?} for {scraper:?}")]
    HtmlScraping { element: String, scraper: String },
    #[error("unable to parse json")]
    JsonParse(#[from] serde_json::Error),
    #[error("io operation failed")]
    FileSystem(#[from] std::io::Error),
    #[error("unable to parse url")]
    UrlParse(#[from] url::ParseError),
    #[error("unable to guess extensionfor {0}")]
    GuessExt(String),
    #[error("no host found for {0}")]
    NoHost(String),
}
