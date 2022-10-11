use std::{
    ffi::OsStr,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::{response::IntoResponse, routing::post, Extension, Json, Router};

use image::GenericImageView;
use image_hasher::{HashAlg, Hasher, HasherConfig};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod scraper;

pub struct Config {
    pub addr: SocketAddr,
    pub dir: PathBuf,
}
pub async fn application(config: Config) -> Result<(), hyper::Error> {
    let app = Router::new()
        .route("/url", post(url_handler))
        .layer(Extension(config.dir));
    axum::Server::bind(&config.addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

static HASHER: Lazy<Hasher> = Lazy::new(|| {
    HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(8, 8)
        .to_hasher()
});
#[axum_macros::debug_handler]
async fn url_handler(
    Json(body): Json<Body>,
    Extension(dir): Extension<PathBuf>,
) -> Result<MediaInfo, AppErr> {
    let media = scraper::scrape(&body.url).await?;
    let path = match media.kind {
        scraper::Type::Image => {
            let img = tokio::fs::read(&media.location).await?;
            tokio::fs::remove_file(&media.location).await.ok();
            let img = image::load_from_memory(&img)?;
            let hash = HASHER.hash_image(&img);
            let hash = hex::encode(hash.as_bytes());
            let mut path = dir.join(&body.folder).join(hash);
            path.set_extension("png");
            let current = img.dimensions();
            if let Ok(exists) = image::image_dimensions(&path) {
                if exists >= current {
                    return Err(AppErr::BetterImage {
                        old: exists,
                        new: current,
                    });
                }
            }
            let _ = img.save_with_format(&path, image::ImageFormat::Png)?;
            path
        }
        scraper::Type::Video => {
            let path = dir.join(&body.folder).join(
                media
                    .location
                    .file_name()
                    .unwrap_or(OsStr::new("default.mp4")),
            );
            tokio::fs::copy(&media.location, &path).await?;
            tokio::fs::remove_file(&media.location).await.ok();
            path
        }
    };
    let media_info = MediaInfo::new(&path, body.folder, media.kind).await?;
    Ok(media_info)
}

#[derive(Deserialize)]
struct Body {
    url: String,
    folder: Folder,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Folder {
    Safe,
    Unsafe,
}

impl AsRef<Path> for Folder {
    fn as_ref(&self) -> &Path {
        let path = match self {
            Folder::Safe => "safe",
            Folder::Unsafe => "unsafe",
        };
        Path::new(path)
    }
}

#[derive(Debug, Serialize)]
struct MediaInfo {
    size: String,
    kind: scraper::Type,
    folder: Folder,
}

impl IntoResponse for MediaInfo {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string_pretty(&self)
            .unwrap_or_default()
            .into_response()
    }
}

impl MediaInfo {
    async fn new(
        location: &Path,
        folder: Folder,
        kind: scraper::Type,
    ) -> Result<Self, std::io::Error> {
        let len = tokio::fs::metadata(location).await?.len();
        let unit = byte_unit::Byte::from_bytes(len as u128);
        Ok(Self {
            size: unit.get_appropriate_unit(false).format(2),
            kind: kind,
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
    #[error("unable to parse image")]
    Image(#[from] image::ImageError),
    #[error("better image exists. {old:?} > {new:?}")]
    BetterImage { old: (u32, u32), new: (u32, u32) },
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        format!("{self:?}").into_response()
    }
}
