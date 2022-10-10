use std::{net::SocketAddr, path::PathBuf};

use axum::{response::IntoResponse, routing::post, Extension, Json, Router};

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod scraper;

pub struct Config {
    addr: SocketAddr,
    dir: PathBuf,
}
pub async fn application(config: Config) -> ! {
    let app = Router::new()
        .route("/url", post(url_handler))
        .layer(Extension(config.dir));
    axum::Server::bind(&config.addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    todo!()
}

#[axum_macros::debug_handler]
async fn url_handler(
    Json(body): Json<Body>,
    Extension(dir): Extension<PathBuf>,
) -> Result<MediaInfo, AppErr> {
    let media = scraper::scrape(&body.url).await?;
    Ok(MediaInfo::From(media, body.folder).await?)
}

#[derive(Deserialize)]
struct Body {
    url: String,
    folder: Folder,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Folder {
    Safe,
    Unsafe,
}

#[derive(Debug)]
struct MediaInfo {
    size: String,
    kind: scraper::Type,
    folder: Folder,
}

impl IntoResponse for MediaInfo {
    fn into_response(self) -> axum::response::Response {
        format!("{self:?}").into_response()
    }
}

impl MediaInfo {
    async fn From(media: scraper::Media, folder: Folder) -> Result<Self, std::io::Error> {
        let len = tokio::fs::metadata(&media.location).await?.len();
        let unit = byte_unit::Byte::from_bytes(len as u128);
        Ok(Self {
            size: unit.get_appropriate_unit(false).format(2),
            kind: media.kind,
            folder,
        })
    }
}

#[derive(Error, Debug)]
enum AppErr {
    #[error("error during scraping")]
    Scrape(#[from] scraper::ScrapErr),
    #[error("file access error")]
    FileIO(#[from] std::io::Error),
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        format!("{self:?}").into_response()
    }
}
