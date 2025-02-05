use std::error::Error;

use serde::{Deserialize, Serialize};
use youtube_transcript::{Transcript, TranscriptCore, Youtube, YoutubeBuilder};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoItem {
    pub id: VideoId,
    pub snippet: VideoSnippet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoId {
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoSnippet {
    pub title: String,
    pub description: String,
    pub thumbnails: Thumbnails,
    #[serde(rename = "publishTime")]
    pub publish_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnails {
    pub high: Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub items: Vec<VideoItem>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub items: Vec<ChannelItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelItem {
    pub id: String,
}

pub async fn get_channel_id(api_key: &str, channel_url: String) -> Result<String, Box<dyn Error>> {
    if channel_url.contains("channel") {
        return Ok(channel_url.split("/").last().unwrap().to_string());
    }
    let handle_name = channel_url.split("/").last().unwrap();
    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?key={}&forHandle={}&part=id",
        api_key, handle_name
    );

    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json::<ChannelResponse>()
        .await?;

    Ok(response.items[0].id.clone())
}

pub async fn get_channel_video_ids(
    api_key: &str,
    channel_id: &str,
) -> Result<Vec<VideoItem>, Box<dyn Error>> {
    let mut videos = Vec::new();
    let mut page_token = None;

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=id,snippet&order=viewCount&type=video&maxResults=50",
            api_key,
            channel_id
        );

        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = reqwest::Client::new()
            .get(&url)
            .send()
            .await?
            .json::<PlaylistResponse>()
            .await?;

        videos.extend(response.items.into_iter());

        match response.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(videos)
}

pub async fn get_transcript(video_id: &str) -> Result<Vec<TranscriptCore>, Box<dyn Error>> {
    let youtube_builder = YoutubeBuilder::default();
    let youtube_loader: Youtube = youtube_builder.build();
    let link = format!("https://www.youtube.com/watch?v={}", video_id);
    let transcript: Transcript = youtube_loader.transcript(&link).await?;
    let mut chunks = Vec::new();
    let mut current_chunk = TranscriptCore {
        start: std::time::Duration::from_secs_f64(0.0),
        duration: std::time::Duration::from_secs_f64(0.0),
        text: String::new(),
    };

    for t in transcript.transcripts {
        if current_chunk.text.split_whitespace().count() + t.text.split_whitespace().count() <= 200
        {
            if current_chunk.text.is_empty() {
                current_chunk.start = t.start;
            }
            current_chunk
                .text
                .push_str(format!("{} \n", t.text).as_str());
            current_chunk.duration = t.start + t.duration - current_chunk.start;
        } else {
            chunks.push(current_chunk);
            current_chunk = TranscriptCore {
                start: t.start,
                duration: t.duration,
                text: format!("{} \n", t.text),
            };
        }
    }

    if !current_chunk.text.is_empty() {
        chunks.push(current_chunk);
    }

    Ok(chunks)
}
