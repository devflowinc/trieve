use actix_web::web;
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use ureq::json;

use crate::data::models::CrawlStatus;
use crate::{
    data::models::{CrawlRequest, CrawlRequestPG, Pool},
    errors::ServiceError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestResult {
    pub success: bool,
    pub r#type: String,
    pub error: Option<String>,
    pub id: uuid::Uuid,
    pub data: Vec<Document>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rawHtml")]
    pub raw_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robots: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogTitle")]
    pub og_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogDescription")]
    pub og_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogUrl")]
    pub og_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogImage")]
    pub og_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogAudio")]
    pub og_audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogDeterminer")]
    pub og_determiner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogLocale")]
    pub og_locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogLocaleAlternate")]
    pub og_locale_alternate: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogSiteName")]
    pub og_site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ogVideo")]
    pub og_video: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcTermsCreated")]
    pub dc_terms_created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcDateCreated")]
    pub dc_date_created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcDate")]
    pub dc_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcTermsType")]
    pub dc_terms_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcType")]
    pub dc_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcTermsAudience")]
    pub dc_terms_audience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcTermsSubject")]
    pub dc_terms_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcSubject")]
    pub dc_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcDescription")]
    pub dc_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dcTermsKeywords")]
    pub dc_terms_keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "publishedTime")]
    pub published_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "articleTag")]
    pub article_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "articleSection")]
    pub article_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statusCode")]
    pub status_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
        .select((id, url, status, scrape_id, dataset_id, created_at))
        .filter(scrape_id.eq(crawl_id))
        .first::<CrawlRequestPG>(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(request.into())
}

pub async fn create_crawl_request(
    url: String,
    dataset_id: uuid::Uuid,
    scrape_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::crawl_requests::dsl as crawl_requests_table;
    let new_crawl_request = CrawlRequestPG {
        id: uuid::Uuid::new_v4(),
        url,
        status: CrawlStatus::Pending.to_string(),
        scrape_id,
        dataset_id,
        created_at: chrono::Utc::now().naive_utc(),
    };
    let mut conn = pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    diesel::insert_into(crawl_requests_table::crawl_requests)
        .values(&new_crawl_request)
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
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

pub async fn crawl_site(
    url: String,
) -> Result<uuid::Uuid, ServiceError> {
    let firecrawl_url = std::env::var("FIRECRAWL_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
    let firecrawl_webhook = std::env::var("FIRECRAWL_WEBHOOK").unwrap_or_else(|_| "http://localhost:54324".to_string());
    let firecrawl_url = format!("{}/v1/crawl", firecrawl_url);
    let client = reqwest::Client::new();
    let response = client
        .post(&firecrawl_url)
        .json(&json!({ "url": url, "webhook":  firecrawl_webhook}))
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
        log::error!("Error getting response from firecrawl: {:?}", response);
        Err(ServiceError::InternalServerError("Error getting response from firecrawl".to_string()))
    }
}


pub fn get_tags(url: String) -> Vec<String> {
    if let Ok(parsed_url) = Url::parse(&url) {
        let path_parts: Vec<&str> = parsed_url.path().split('/').collect();
        if let Some(docs_index) = path_parts.iter().position(|&part| part == "docs") {
            return path_parts
                .iter()
                .skip(docs_index + 1)
                .take(path_parts.len() - docs_index - 2)
                .filter(|&&part| !part.is_empty())
                .map(|&part| part.to_string())
                .collect();
        }
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

    let mut chunk_html = chunk_content
        .split('\n')
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .replace(|c| c == '-', "");

    chunk_html = Cleaners::clean_multi_column_links(chunk_html);
    chunk_html = Cleaners::clean_anchortag_headings(chunk_html);
    chunk_html = Cleaners::clean_extra_newlines_after_links(chunk_html);
    chunk_html = Cleaners::clean_double_asterisk_whitespace_gaps(chunk_html);
    chunk_html = Cleaners::clean_newline_and_spaces_after_links(chunk_html);

    // Skip heading-only chunks
    if chunk_html.trim().split('\n').count() <= 1 {
        return format!("{{\"HEADING_ONLY\": \"{}\"}}", chunk_html.trim());
    }

    if heading_text.is_empty() {
        chunk_html = chunk_html.trim().replace(|c| c == '-', "");
    } else {
        let heading_line = format!("{}: {}", page_title, heading_text);
        let mut lines: Vec<&str> = chunk_html.split('\n').collect();
        lines[0] = heading_line.as_str();
        chunk_html = lines.join("\n");
    }

    chunk_html
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

    pub fn remove_end_matter(text: String) -> String {
        let patterns = [
            r"\[]\(#get-help\)",
            r"\[Prev",
            r"If you have any questions or need any help in setting things up, join our slack community and ping us in `#help` channel.",
        ];

        let indices: Vec<_> = patterns
            .iter()
            .filter_map(|&pattern| Regex::new(pattern).ok()?.find(&text).map(|m| m.start()))
            .collect();

        if let Some(&remove_index) = indices.iter().min() {
            text[..remove_index].trim().to_string()
        } else {
            text.to_string()
        }
    }
}

pub fn get_images(markdown_content: &str) -> Vec<String> {
    let image_pattern = Regex::new(r"!\[.*?\]\((.*?\.(?:png|webp))\)").unwrap();
    image_pattern
        .captures_iter(markdown_content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}
