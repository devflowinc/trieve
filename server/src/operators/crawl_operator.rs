use crate::data::models::CrawlOptions;
use crate::data::models::CrawlStatus;
use crate::data::models::FirecrawlCrawlRequest;
use crate::data::models::RedisPool;
use crate::handlers::chunk_handler::CrawlInterval;
use crate::{
    data::models::{CrawlRequest, CrawlRequestPG, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestResult {
    pub status: Status,
    pub completed: u32,
    pub total: u32,
    #[serde(rename = "creditsUsed")]
    pub credits_used: u32,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
    pub next: Option<String>,
    pub data: Option<Vec<Option<Document>>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Scraping,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub markdown: Option<String>,
    pub extract: Option<String>,
    pub html: Option<String>,
    #[serde(rename = "rawHtml")]
    pub raw_html: Option<String>,
    pub links: Option<Vec<String>>,
    pub screenshot: Option<String>,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub keywords: Option<String>,
    pub robots: Option<String>,
    #[serde(rename = "ogTitle")]
    pub og_title: Option<String>,
    #[serde(rename = "ogDescription")]
    pub og_description: Option<String>,
    #[serde(rename = "ogUrl")]
    pub og_url: Option<String>,
    #[serde(rename = "ogImage")]
    pub og_image: Option<String>,
    #[serde(rename = "ogAudio")]
    pub og_audio: Option<String>,
    #[serde(rename = "ogDeterminer")]
    pub og_determiner: Option<String>,
    #[serde(rename = "ogLocale")]
    pub og_locale: Option<String>,
    #[serde(rename = "ogLocaleAlternate")]
    pub og_locale_alternate: Option<Vec<String>>,
    #[serde(rename = "ogSiteName")]
    pub og_site_name: Option<String>,
    #[serde(rename = "ogVideo")]
    pub og_video: Option<String>,
    #[serde(rename = "dcTermsCreated")]
    pub dc_terms_created: Option<String>,
    #[serde(rename = "dcDateCreated")]
    pub dc_date_created: Option<String>,
    #[serde(rename = "dcDate")]
    pub dc_date: Option<String>,
    #[serde(rename = "dcTermsType")]
    pub dc_terms_type: Option<String>,
    #[serde(rename = "dcType")]
    pub dc_type: Option<String>,
    #[serde(rename = "dcTermsAudience")]
    pub dc_terms_audience: Option<String>,
    #[serde(rename = "dcTermsSubject")]
    pub dc_terms_subject: Option<String>,
    #[serde(rename = "dcSubject")]
    pub dc_subject: Option<String>,
    #[serde(rename = "dcDescription")]
    pub dc_description: Option<String>,
    #[serde(rename = "dcTermsKeywords")]
    pub dc_terms_keywords: Option<String>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
    #[serde(rename = "publishedTime")]
    pub published_time: Option<String>,
    #[serde(rename = "articleTag")]
    pub article_tag: Option<String>,
    #[serde(rename = "articleSection")]
    pub article_section: Option<String>,
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    #[serde(rename = "statusCode")]
    pub status_code: Option<u32>,
    pub error: Option<String>,
    pub site_map: Option<Sitemap>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sitemap {
    pub changefreq: String,
}

pub async fn crawl(
    crawl_options: CrawlOptions,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_id: uuid::Uuid,
) -> Result<uuid::Uuid, ServiceError> {
    let scrape_id = crawl_site(crawl_options.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not crawl site: {}", err)))?;

    create_crawl_request(crawl_options, dataset_id, scrape_id, pool, redis_pool).await?;

    Ok(scrape_id)
}

pub async fn get_crawl_request(
    crawl_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CrawlRequest, ServiceError> {
    use crate::data::schema::crawl_requests::dsl::*;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    let request = crawl_requests
        .select((
            id,
            url,
            status,
            next_crawl_at,
            interval,
            crawl_options,
            scrape_id,
            dataset_id,
            created_at,
        ))
        .filter(scrape_id.eq(crawl_id))
        .first::<CrawlRequestPG>(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(request.into())
}

pub async fn get_crawl_requests_to_rerun(
    pool: web::Data<Pool>,
) -> Result<Vec<CrawlRequest>, ServiceError> {
    use crate::data::schema::crawl_requests::dsl::*;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    let requests = crawl_requests
        .select((
            id,
            url,
            status,
            next_crawl_at,
            interval,
            crawl_options,
            scrape_id,
            dataset_id,
            created_at,
        ))
        .filter(next_crawl_at.le(chrono::Utc::now().naive_utc()))
        .load::<CrawlRequestPG>(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(requests.into_iter().map(|r| r.into()).collect())
}

pub async fn create_crawl_request(
    crawl_options: CrawlOptions,
    dataset_id: uuid::Uuid,
    scrape_id: uuid::Uuid,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;

    let interval = match crawl_options.interval {
        Some(CrawlInterval::Daily) => std::time::Duration::from_secs(60 * 60 * 24),
        Some(CrawlInterval::Weekly) => std::time::Duration::from_secs(60 * 60 * 24 * 7),
        Some(CrawlInterval::Monthly) => std::time::Duration::from_secs(60 * 60 * 24 * 30),
        None => std::time::Duration::from_secs(60 * 60 * 24),
    };

    let new_crawl_request: CrawlRequestPG = CrawlRequest {
        id: uuid::Uuid::new_v4(),
        url: crawl_options.site_url.clone().unwrap_or_default(),
        status: CrawlStatus::Pending,
        interval,
        next_crawl_at: chrono::Utc::now().naive_utc(),
        crawl_options,
        scrape_id,
        dataset_id,
        created_at: chrono::Utc::now().naive_utc(),
        attempt_number: 0,
    }
    .into();

    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    diesel::insert_into(crawl_requests_table::crawl_requests)
        .values(&new_crawl_request)
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let serialized_message =
        serde_json::to_string(&CrawlRequest::from(new_crawl_request.clone())).unwrap();
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    redis::cmd("lpush")
        .arg("scrape_queue")
        .arg(&serialized_message)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    Ok(new_crawl_request.scrape_id)
}

pub async fn update_crawl_status(
    crawl_id: uuid::Uuid,
    status: CrawlStatus,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    diesel::update(
        crawl_requests_table::crawl_requests.filter(crawl_requests_table::scrape_id.eq(crawl_id)),
    )
    .set(crawl_requests_table::status.eq(status.to_string()))
    .execute(&mut conn)
    .await
    .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(())
}

pub async fn update_next_crawl_at(
    crawl_id: uuid::Uuid,
    next_crawl_at: chrono::NaiveDateTime,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    diesel::update(
        crawl_requests_table::crawl_requests.filter(crawl_requests_table::scrape_id.eq(crawl_id)),
    )
    .set(crawl_requests_table::next_crawl_at.eq(next_crawl_at))
    .execute(&mut conn)
    .await
    .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(())
}

pub async fn update_crawl_settings_for_dataset(
    crawl_options: CrawlOptions,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let crawl_req = crawl_requests_table::crawl_requests
        .select((
            crawl_requests_table::id,
            crawl_requests_table::url,
            crawl_requests_table::status,
            crawl_requests_table::next_crawl_at,
            crawl_requests_table::interval,
            crawl_requests_table::crawl_options,
            crawl_requests_table::scrape_id,
            crawl_requests_table::dataset_id,
            crawl_requests_table::created_at,
        ))
        .filter(crawl_requests_table::dataset_id.eq(dataset_id))
        .first::<CrawlRequestPG>(&mut conn)
        .await;

    if let Some(ref url) = crawl_options.site_url {
        if crawl_req.is_err() {
            crawl(
                crawl_options.clone(),
                pool.clone(),
                redis_pool.clone(),
                dataset_id,
            )
            .await?;
        }

        diesel::update(
            crawl_requests_table::crawl_requests
                .filter(crawl_requests_table::dataset_id.eq(dataset_id)),
        )
        .set(crawl_requests_table::url.eq(url))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    }

    if let Some(interval) = crawl_options.interval {
        let interval = match interval {
            CrawlInterval::Daily => std::time::Duration::from_secs(60 * 60 * 24),
            CrawlInterval::Weekly => std::time::Duration::from_secs(60 * 60 * 24 * 7),
            CrawlInterval::Monthly => std::time::Duration::from_secs(60 * 60 * 24 * 30),
        };
        diesel::update(
            crawl_requests_table::crawl_requests
                .filter(crawl_requests_table::dataset_id.eq(dataset_id)),
        )
        .set(crawl_requests_table::interval.eq(interval.as_secs() as i32))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    }

    Ok(())
}

pub async fn update_scrape_id(
    scrape_id: uuid::Uuid,
    new_scrape_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CrawlRequest, ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    let updated_request = diesel::update(
        crawl_requests_table::crawl_requests.filter(crawl_requests_table::scrape_id.eq(scrape_id)),
    )
    .set(crawl_requests_table::scrape_id.eq(new_scrape_id))
    .returning(CrawlRequestPG::as_returning())
    .get_result(&mut conn)
    .await
    .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(updated_request.into())
}

pub async fn get_crawl_from_firecrawl(scrape_id: uuid::Uuid) -> Result<IngestResult, ServiceError> {
    log::info!("Getting crawl from firecrawl");

    let firecrawl_url =
        std::env::var("FIRECRAWL_URL").unwrap_or_else(|_| "https://api.firecrawl.dev".to_string());
    let firecrawl_api_key = std::env::var("FIRECRAWL_API_KEY").unwrap_or_else(|_| "".to_string());
    let mut firecrawl_url = format!("{}/v1/crawl/{}", firecrawl_url, scrape_id);

    let mut collected_docs: Vec<Option<Document>> = vec![];
    let mut resp = None;

    let client = reqwest::Client::new();

    while resp.is_none() {
        let response = client
            .get(&firecrawl_url)
            .header("Authorization", format!("Bearer {}", firecrawl_api_key))
            .send()
            .await
            .map_err(|e| {
                log::error!("Error sending request to firecrawl: {:?}", e);
                ServiceError::InternalServerError("Error sending request to firecrawl".to_string())
            })?;

        if !response.status().is_success() {
            log::error!(
                "Error getting response from firecrawl: {:?}",
                response.text().await
            );
            return Err(ServiceError::InternalServerError(
                "Error getting response from firecrawl".to_string(),
            ));
        };

        let ingest_result = response.json::<IngestResult>().await.map_err(|e| {
            log::error!("Error parsing response from firecrawl: {:?}", e);
            ServiceError::InternalServerError("Error parsing response from firecrawl".to_string())
        })?;
        if ingest_result.status != Status::Completed {
            log::info!("Crawl status: {:?}", ingest_result.status);
            return Ok(ingest_result);
        }

        let cur_docs = ingest_result.clone().data.unwrap_or_default();
        collected_docs.extend(cur_docs);

        if let Some(ref next_ingest_result) = ingest_result.next {
            let next_ingest_result =
                next_ingest_result.replace("https://localhost", "http://localhost");

            log::info!(
                "Next ingest url: {} | prev {}",
                next_ingest_result,
                firecrawl_url
            );
            if next_ingest_result == firecrawl_url {
                log::info!("Breaking loop");
                resp = Some(ingest_result.clone());
                break;
            }

            firecrawl_url = next_ingest_result;
        } else {
            resp = Some(ingest_result.clone());
        }
    }

    match resp {
        Some(resp) => Ok(IngestResult {
            status: resp.status,
            completed: resp.completed,
            total: resp.total,
            credits_used: resp.credits_used,
            expires_at: resp.expires_at,
            next: None,
            data: Some(collected_docs),
        }),
        None => Err(ServiceError::InternalServerError(
            "Error getting response from firecrawl".to_string(),
        )),
    }
}

pub async fn crawl_site(crawl_options: CrawlOptions) -> Result<uuid::Uuid, ServiceError> {
    let firecrawl_url =
        std::env::var("FIRECRAWL_URL").unwrap_or_else(|_| "https://api.firecrawl.dev".to_string());
    let firecrawl_api_key = std::env::var("FIRECRAWL_API_KEY").unwrap_or_else(|_| "".to_string());
    let firecrawl_url = format!("{}/v1/crawl", firecrawl_url);
    let client = reqwest::Client::new();
    let response = client
        .post(&firecrawl_url)
        .json(&FirecrawlCrawlRequest::from(crawl_options))
        .header("Authorization", format!("Bearer {}", firecrawl_api_key))
        .send()
        .await
        .map_err(|e| {
            log::error!("Error sending request to firecrawl: {:?}", e);
            ServiceError::InternalServerError("Error sending request to firecrawl".to_string())
        })?;

    if response.status().is_success() {
        let json = response.json::<serde_json::Value>().await.map_err(|e| {
            log::error!("Error parsing response from firecrawl: {:?}", e);
            ServiceError::InternalServerError("Error parsing response from firecrawl".to_string())
        })?;

        Ok(json.get("id").unwrap().as_str().unwrap().parse().unwrap())
    } else {
        log::error!(
            "Error getting response from firecrawl: {:?}",
            response.text().await
        );
        Err(ServiceError::InternalServerError(
            "Error getting response from firecrawl".to_string(),
        ))
    }
}

pub fn get_tags(url: String) -> Vec<String> {
    if let Ok(parsed_url) = Url::parse(&url) {
        let path_parts: Vec<&str> = parsed_url.path().split('/').collect();
        return path_parts
            .iter()
            .filter_map(|part| {
                if !part.is_empty() {
                    Some(part.to_string())
                } else {
                    None
                }
            })
            .collect();
    }
    Vec::new()
}

pub fn get_chunk_html(
    content: String,
    page_title: String,
    heading_text: String,
    start_index: usize,
    chunk_end: Option<usize>,
) -> String {
    let chunk_content = match chunk_end {
        Some(end) => &content[start_index..end],
        None => &content[start_index..],
    };

    let mut chunk_markdown = chunk_content
        .split('\n')
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .replace('-', "");

    chunk_markdown = Cleaners::clean_multi_column_links(chunk_markdown);
    chunk_markdown = Cleaners::clean_anchortag_headings(chunk_markdown);
    chunk_markdown = Cleaners::clean_extra_newlines_after_links(chunk_markdown);
    chunk_markdown = Cleaners::clean_double_asterisk_whitespace_gaps(chunk_markdown);
    chunk_markdown = Cleaners::clean_newline_and_spaces_after_links(chunk_markdown);

    // Skip heading-only chunks
    if chunk_markdown.trim().split('\n').count() <= 1 {
        return format!("{{\"HEADING_ONLY\": \"{}\"}}", chunk_markdown.trim());
    }

    if heading_text.is_empty() {
        chunk_markdown = chunk_markdown.trim().replace('-', "");
    } else {
        let heading_line = format!("{}: {}", page_title, heading_text);
        let mut lines: Vec<&str> = chunk_markdown.split('\n').collect();
        lines[0] = heading_line.as_str();
        chunk_markdown = lines.join("\n");
    }

    chunk_markdown
}

pub struct Cleaners;

impl Cleaners {
    pub fn clean_double_newline_markdown_links(text: String) -> String {
        let re = Regex::new(r"\[(.*?\\\s*\n\s*\\\s*\n\s*.*?)\]\((.*?)\)").unwrap();
        re.replace_all(&text, |caps: &regex::Captures| {
            let full_content = &caps[1];
            let url = &caps[2];
            let cleaned_content = Regex::new(r"\\\s*\n\s*\\\s*\n\s*")
                .unwrap()
                .replace_all(full_content, " ");
            format!("[{}]({})", cleaned_content, url)
        })
        .to_string()
    }

    pub fn clean_anchortag_headings(text: String) -> String {
        let re = Regex::new(r"\[\]\((#.*?)\)\n(.*?)($|\n)").unwrap();
        re.replace_all(&text, "## $2").to_string()
    }

    pub fn clean_double_asterisk_whitespace_gaps(text: String) -> String {
        let re = Regex::new(r"\*\*(\[.*?\]\(.*?\))\n\s*\*\*").unwrap();
        re.replace_all(&text, "**$1**").to_string()
    }

    pub fn clean_newline_and_spaces_after_links(text: String) -> String {
        let re = Regex::new(r"(\[.*?\]\(.*?\))\n\s*([a-z].*)").unwrap();
        re.replace_all(&text, "$1 $2").to_string()
    }

    pub fn clean_multi_column_links(markdown_text: String) -> String {
        let link_pattern = Regex::new(r"(\n\n)(\[(?:[^\]]+\\\s*)+[^\]]+\]\([^\)]+\)(?:\s*\[(?:[^\]]+\\\s*)+[^\]]+\]\([^\)]+\))*)\s*").unwrap();
        let link_re = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();

        link_pattern
            .replace_all(&markdown_text, |caps: &regex::Captures| {
                let newlines = &caps[1];
                let links = &caps[2];
                let cleaned_links: Vec<String> = link_re
                    .captures_iter(links)
                    .map(|link_cap| {
                        let link_text = &link_cap[1];
                        let link_url = &link_cap[2];
                        let clean_text = Regex::new(r"\\\s*\n\s*\\\s*\n\s*")
                            .unwrap()
                            .replace_all(link_text, ": ");
                        let clean_text = Regex::new(r"\s*\\\s*\n\s*")
                            .unwrap()
                            .replace_all(&clean_text, " ");
                        let clean_text = Regex::new(r"\\ \\ ")
                            .unwrap()
                            .replace_all(&clean_text, ": ");
                        format!("- [{}]({})", clean_text.trim(), link_url)
                    })
                    .collect();
                format!("{}{}", newlines, cleaned_links.join("\n"))
            })
            .trim()
            .to_string()
    }

    pub fn clean_extra_newlines_after_links(text: String) -> String {
        let re1 = Regex::new(r"(\[.*?\]\(.*?\))\n\.").unwrap();
        let re2 = Regex::new(r"(\[.*?\]\(.*?\))\n").unwrap();
        let text = re1.replace_all(&text, "$1.");
        re2.replace_all(&text, "$1 ").to_string()
    }
}

pub fn get_images(markdown_content: &str) -> Vec<String> {
    let image_pattern = Regex::new(r"\((https?://.*?\.(?:png|jpg|jpeg|gif|bmp|webp))\)").unwrap();
    image_pattern
        .captures_iter(markdown_content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn chunk_markdown(markdown: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"(?m)^(#{1,6}\s.+)$").unwrap();
    let mut chunks = Vec::new();
    let mut current_chunk = (String::new(), String::new());

    for line in markdown.lines() {
        if re.is_match(line) {
            if !current_chunk.1.is_empty() {
                current_chunk.1 = current_chunk.1.trim().to_string();
                chunks.push(current_chunk);
                current_chunk = (String::new(), String::new());
            }
            current_chunk.0.push_str(line);
            current_chunk.1.push_str(line);
            current_chunk.1.push('\n');
        } else {
            current_chunk.1.push_str(line);
            current_chunk.1.push('\n');
        }
    }

    if !current_chunk.1.is_empty() {
        current_chunk.1 = current_chunk.1.trim().to_string();
        chunks.push(current_chunk);
    }

    chunks
}
