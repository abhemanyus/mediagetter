use std::env::temp_dir;

use reqwest::header::CONTENT_TYPE;
use uuid::Uuid;

use super::{Media, ScrapErr, Type, CLIENT};

#[tokio::test]
async fn download_media() {
    scrape("https://cdn.discordapp.com/attachments/509982049239564290/1028336757080608830/1665206582990.mp4").await.unwrap();
    scrape("https://cdn.discordapp.com/attachments/509982049239564290/1028212859152379955/32f1a2297b08e31a6d374bd57b0deb92.jpg").await.unwrap();
}

pub async fn scrape(url: &str) -> Result<Media, ScrapErr> {
    let img = CLIENT.get(url).send().await?;
    let is_video = img
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|mime| mime.to_str().ok())
        .map(|mime| mime.starts_with("video/"))
        .ok_or(ScrapErr::GuessExt(url.to_string()))?;
    let img = img.bytes().await?;
    let mut path = temp_dir().join(Uuid::new_v4().to_string());
    if is_video {
        path.set_extension("mp4");
    }
    let _ = tokio::fs::write(&path, &img).await?;
    Ok(Media {
        location: path,
        kind: if is_video { Type::Video } else { Type::Image },
    })
}
