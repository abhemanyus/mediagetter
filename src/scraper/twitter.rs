use std::{env::temp_dir, str::FromStr};

use reqwest::Url;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use super::{Media, ScrapErr, Type, CLIENT};

#[tokio::test]
async fn download_media() {
    // test image
    scrape(
        "https://twitter.com/dog_rates/status/1578145895584911362?s=20&t=7PAtwWBDwS8wjFeisuawMg",
    )
    .await
    .unwrap();

    // test video
    scrape(
        "https://twitter.com/furyofthegodz/status/1578910543439486976?s=20&t=mEdD0Y064M9SDLZHUwJnvw",
    )
    .await
    .unwrap();
}

const TOKEN: &str = include_str!("../../twitter.token");

pub async fn scrape(url: &str) -> Result<Media, ScrapErr> {
    let url = Url::parse(url)?;
    let tweet_id =
        url.path_segments()
            .and_then(|seg| seg.last())
            .ok_or(ScrapErr::HtmlScraping {
                element: "tweet id".to_string(),
                scraper: "twitter url".to_string(),
            })?;
    let mut api_url = Url::from_str("https://api.twitter.com/2/tweets/")?.join(tweet_id)?;
    api_url.set_query(Some(
        "expansions=attachments.media_keys&media.fields=url,variants",
    ));
    let tweet = CLIENT
        .get(api_url)
        .bearer_auth(TOKEN)
        .send()
        .await?
        .text()
        .await?;
    let tweet: Tweet = serde_json::from_str(&tweet)?;
    let media = tweet.includes.media.first().ok_or(ScrapErr::HtmlScraping {
        element: "tweet.media".to_string(),
        scraper: "twitter".to_string(),
    })?;
    let (media_url, media_type) = match media {
        MediaType::Photo { url } => Some((url, Type::Image)),
        MediaType::Video { variants } => variants
            .into_iter()
            .max_by_key(|var| var.bit_rate)
            .map(|var| (&var.url, Type::Video)),
    }
    .ok_or(ScrapErr::HtmlScraping {
        element: "media.url".to_string(),
        scraper: "twitter.json".to_string(),
    })?;
    let mut media_data = CLIENT.get(media_url).send().await?;
    let path = temp_dir().join(Uuid::new_v4().to_string());
    let mut file = tokio::fs::File::create(&path).await?;
    while let Some(chunk) = media_data.chunk().await? {
        file.write(&chunk).await?;
    }
    file.shutdown().await?;
    Ok(Media {
        location: path,
        kind: media_type,
    })
}

#[derive(Deserialize, Debug)]
struct Tweet {
    includes: Includes,
}

#[derive(Deserialize, Debug)]
struct Includes {
    media: Vec<MediaType>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type")]
enum MediaType {
    Photo { url: String },
    Video { variants: Vec<Variant> },
}

#[derive(Deserialize, Debug)]
struct Variant {
    #[serde(default)]
    bit_rate: u32,
    url: String,
}

#[derive(Deserialize, Debug)]
enum ContentType {
    #[serde(rename = "video/mp4")]
    Video,
    #[serde(rename = "application/x-mpegURL")]
    Stream,
}
