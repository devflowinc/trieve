use std::error::Error;

use actix_web::web;
use broccoli_queue::{error::BroccoliError, queue::BroccoliQueue};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use trieve_server::{
    data::models::{
        ChunkGroup, DatasetAndOrgWithSubAndPlan, EventType, Pool, RedisPool, UnifiedId,
        VideoCrawlMessage, WorkerEvent,
    },
    establish_connection, get_env,
    handlers::chunk_handler::ChunkReqPayload,
    operators::{
        chunk_operator::{create_chunk_metadata, get_row_count_for_organization_id_query},
        clickhouse_operator::{ClickHouseEvent, EventQueue},
        dataset_operator::get_dataset_and_organization_from_dataset_id_query,
        group_operator::create_groups_query,
    },
};
use ureq::json;
use youtube_transcript::{Transcript, TranscriptCore, Youtube, YoutubeBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(3)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    let event_queue = if std::env::var("USE_ANALYTICS")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false)
    {
        log::info!("Analytics enabled");

        let clickhouse_client = clickhouse::Client::default()
            .with_url(
                std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string()),
            )
            .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
            .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
            .with_database(std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()))
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0");

        let mut event_queue = EventQueue::new(clickhouse_client.clone());
        event_queue.start_service();
        event_queue
    } else {
        log::info!("Analytics disabled");
        EventQueue::default()
    };

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");

    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2);

    let redis_manager =
        bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(redis_connections)
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(redis_manager)
        .await
        .expect("Failed to create redis pool");

    let web_redis_pool = actix_web::web::Data::new(redis_pool);

    let queue = BroccoliQueue::builder(redis_url)
        .pool_connections(redis_connections.try_into().unwrap())
        .failed_message_retry_strategy(Default::default())
        .build()
        .await?;

    queue
        .process_messages("video_queue", None, move |msg| {
            let pool = web_pool.clone();
            let redis_pool = web_redis_pool.clone();
            let event_queue = event_queue.clone();
            async move {
                video_worker(
                    msg.payload,
                    pool.clone(),
                    redis_pool.clone(),
                    event_queue.clone(),
                )
                .await
                .map_err(|e| {
                    log::error!("Error processing video worker: {:?}", e);
                    e
                })
            }
        })
        .await?;

    Ok(())
}

async fn video_worker(
    message: VideoCrawlMessage,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    event_queue: EventQueue,
) -> Result<(), BroccoliError> {
    let youtube_api_key = get_env!("YOUTUBE_API_KEY", "YOUTUBE_API_KEY is not set");

    let redis_conn = redis_pool.get().await.map_err(|e| {
        log::error!("Could not get redis connection {:?}", e);
        BroccoliError::Job("Could not get redis connection".to_string())
    })?;

    log::info!("Processing video worker for {}", message.channel_url);

    let channel_id = get_channel_id(youtube_api_key, message.channel_url)
        .await
        .map_err(|e| {
            log::error!("Could not get channel id {:?}", e);
            BroccoliError::Job("Could not get channel id".to_string())
        })?;

    log::info!("Got Channel ID: {}", channel_id);
    let dataset_org_plan_sub = get_dataset_and_organization_from_dataset_id_query(
        UnifiedId::TrieveUuid(message.dataset_id),
        None,
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Could not get dataset and organization {:?}", e);
        BroccoliError::Job("Could not get dataset and organization".to_string())
    })?;

    let videos = get_channel_video_ids(youtube_api_key, &channel_id)
        .await
        .unwrap();

    log::info!("Got {} videos", videos.len());

    for video in videos {
        let mut chunks = Vec::new();
        let chunk_group = ChunkGroup::from_details(
            Some(video.snippet.title.clone()),
            Some(video.snippet.description.clone()),
            dataset_org_plan_sub.dataset.id,
            None,
            None,
            None,
        );

        let chunk_group_option = create_groups_query(vec![chunk_group], true, pool.clone())
            .await
            .map_err(|e| {
                log::error!("Could not create group {:?}", e);
                BroccoliError::Job("Could not create group".to_string())
            })?
            .pop();

        let chunk_group = match chunk_group_option {
            Some(group) => group,
            None => {
                return Err(BroccoliError::Job("Could not create group".to_string()));
            }
        };

        log::info!("Getting transcripts for video_id {}", video.id.video_id);
        let transcripts = get_transcript(&video.id.video_id).await.map_err(|e| {
            BroccoliError::Job(format!(
                "Failed to get transcript for video_id {}: {}",
                video.id.video_id, e
            ))
        });

        match transcripts {
            Ok(transcripts) => {
                for transcript in transcripts {
                    let create_chunk_data = ChunkReqPayload {
                        chunk_html: Some(transcript.text),
                        semantic_content: None,
                        link: Some(format!(
                            "https://www.youtube.com/watch?v={}&t={}",
                            video.id.video_id,
                            transcript.start.as_secs()
                        )),
                        tag_set: None,
                        metadata: Some(json!({
                            "heading": video.snippet.title.clone(),
                            "title": video.snippet.title.clone(),
                            "url": format!("https://www.youtube.com/watch?v={}", video.id.video_id),
                            "hierarchy": video.snippet.title.clone(),
                            "description": video.snippet.description.clone(),
                            "yt_preview_src": video.snippet.thumbnails.high.url.clone(),
                        })),
                        group_ids: Some(vec![chunk_group.id]),
                        group_tracking_ids: None,
                        location: None,
                        tracking_id: None,
                        upsert_by_tracking_id: None,
                        time_stamp: Some(video.snippet.publish_time.clone()),
                        weight: None,
                        split_avg: None,
                        convert_html_to_text: None,
                        image_urls: Some(vec![video.snippet.thumbnails.high.url.clone()]),
                        num_value: None,
                        fulltext_boost: None,
                        semantic_boost: None,
                    };

                    chunks.push(create_chunk_data);
                }

                log::info!(
                    "Sending {} chunks from transcript of video {}",
                    chunks.len(),
                    video.id.video_id
                );

                send_chunks(
                    dataset_org_plan_sub.clone(),
                    chunks,
                    video.id.video_id.clone(),
                    pool.clone(),
                    redis_conn.clone(),
                    event_queue.clone(),
                )
                .await?;
            }
            Err(e) => {
                log::error!("Failed to get transcript for video {}", e);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoItem {
    id: VideoId,
    snippet: VideoSnippet,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoSnippet {
    title: String,
    description: String,
    thumbnails: Thumbnails,
    #[serde(rename = "publishTime")]
    publish_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Thumbnails {
    high: Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistResponse {
    items: Vec<VideoItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelResponse {
    items: Vec<ChannelItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelItem {
    id: String,
}

async fn get_channel_id(api_key: &str, channel_url: String) -> Result<String, Box<dyn Error>> {
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

async fn get_channel_video_ids(
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

async fn get_transcript(video_id: &str) -> Result<Vec<TranscriptCore>, Box<dyn Error>> {
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

async fn send_chunks(
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    chunks: Vec<ChunkReqPayload>,
    video_id: String,
    pool: web::Data<Pool>,
    mut redis_conn: MultiplexedConnection,
    event_queue: EventQueue,
) -> Result<(), BroccoliError> {
    let chunk_count = get_row_count_for_organization_id_query(
        dataset_org_plan_sub.organization.organization.id,
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Could not get row count {:?}", e);
        BroccoliError::Job("Could not get row count".to_string())
    })?;

    if chunk_count + chunks.len()
        > dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .chunk_count as usize
    {
        return Err(BroccoliError::Job(
            "Chunk count exceeds plan limit".to_string(),
        ));
    }

    let chunk_segments = chunks
        .chunks(120)
        .map(|chunk_segment| chunk_segment.to_vec())
        .collect::<Vec<Vec<ChunkReqPayload>>>();

    let mut serialized_messages: Vec<String> = vec![];

    for chunk_segment in chunk_segments {
        let (ingestion_message, _) =
            create_chunk_metadata(chunk_segment, dataset_org_plan_sub.dataset.id)
                .await
                .map_err(|e| {
                    log::error!("Could not create chunk metadata {:?}", e);
                    BroccoliError::Job("Could not create chunk metadata".to_string())
                })?;

        let serialized_message: String = serde_json::to_string(&ingestion_message)
            .map_err(|_| BroccoliError::Job("Failed to serialize message".to_string()))?;

        if serialized_message.is_empty() {
            continue;
        }

        serialized_messages.push(serialized_message);
    }
    // in the future, once all workers use broccoli_queue, we will not need to use redis directly
    for serialized_message in serialized_messages {
        redis::cmd("lpush")
            .arg("ingestion")
            .arg(&serialized_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut redis_conn)
            .await
            .map_err(|err| BroccoliError::Job(err.to_string()))?;
    }

    event_queue
        .send(ClickHouseEvent::WorkerEvent(
            WorkerEvent::from_details(
                dataset_org_plan_sub.dataset.id,
                EventType::VideoUploaded {
                    video_id,
                    chunks_created: chunks.len(),
                },
            )
            .into(),
        ))
        .await;

    Ok(())
}
