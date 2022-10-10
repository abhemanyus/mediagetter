use once_cell::sync::Lazy;
use reqwest::Client;
use std::path::PathBuf;
use thiserror::Error;

mod danbooru;
mod generic;
mod pixiv;
mod twitter;

pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Debug)]
pub enum Type {
    Image,
    Video,
}

#[derive(Debug)]
pub struct Media {
    location: PathBuf,
    kind: Type,
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
}
