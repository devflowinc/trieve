use actix_web::web;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use serde::{Deserialize, Serialize};
use signal_hook::consts::SIGTERM;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use trieve_server::{
    data::models::{self, WorkerEvent},
    operators::{
        clickhouse_operator::{ClickHouseEvent, EventQueue},
        organization_operator::hash_function,
    },
};
use trieve_server::{
    data::models::{CrawlRequest, CrawlShopifyOptions, RedisPool, ScrapeOptions},
    operators::crawl_operator::{get_crawl_from_firecrawl, Status},
};
use trieve_server::{
    data::models::{CrawlStatus, Pool},
    errors::ServiceError,
    establish_connection, get_env,
    operators::crawl_operator::{get_tags, update_crawl_status},
};
use trieve_server::{
    handlers::chunk_handler::ChunkReqPayload, operators::crawl_operator::chunk_html,
};
use trieve_server::{
    handlers::chunk_handler::{FullTextBoost, SemanticBoost},
    operators::{chunk_operator::create_chunk_metadata, crawl_operator::update_next_crawl_at},
};
use ureq::json;

#[derive(Debug)]
struct ScrapeReport {
    request_id: uuid::Uuid,
    pages_scraped: usize,
    chunks_created: usize,
}

#[derive(Debug, Deserialize)]
struct ShopifyResponse {
    products: Vec<ShopifyProduct>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ShopifyVariant {
    id: u64,
    product_id: u64,
    title: Option<String>,
    price: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ShopifyProduct {
    id: u64,
    title: Option<String>,
    body_html: Option<String>,
    handle: Option<String>,
    tags: Vec<String>,
    variants: Vec<ShopifyVariant>,
    images: Vec<ShopifyImage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ShopifyImage {
    src: String,
}

fn create_chunk_req_payload(
    product: &ShopifyProduct,
    variant: &ShopifyVariant,
    base_url: &str,
    scrape_request: &CrawlRequest,
) -> Result<ChunkReqPayload, ServiceError> {
    let image_urls: Vec<String> = product.images.iter().map(|img| img.src.clone()).collect();

    let mut product_title = product.title.clone().unwrap_or_default();
    let mut variant_title = variant.title.clone().unwrap_or_default();
    let mut product_body_html = product.body_html.clone().unwrap_or_default();

    if let Some(heading_remove_strings) = &scrape_request.crawl_options.heading_remove_strings {
        heading_remove_strings.iter().for_each(|remove_string| {
            product_title = product_title.replace(remove_string, "");
            variant_title = variant_title.replace(remove_string, "");
        });
    }
    if let Some(body_remove_strings) = &scrape_request.crawl_options.body_remove_strings {
        body_remove_strings.iter().for_each(|remove_string| {
            product_body_html = product_body_html.replace(remove_string, "");
        });
    }

    let link = format!(
        "{}/products/{}?variant={}",
        base_url,
        product.handle.clone().unwrap_or_default(),
        variant.id
    );

    let mut chunk_html = if variant.title == Some("Default Title".to_string()) {
        format!(
            "<h1>{}</h1>{}",
            product.title.clone().unwrap_or_default(),
            product.body_html.clone().unwrap_or_default()
        )
    } else {
        format!(
            "<h1>{} - {}</h1>{}",
            product.title.clone().unwrap_or_default(),
            variant.title.clone().unwrap_or_default(),
            product.body_html.clone().unwrap_or_default()
        )
    };

    if let Some(ScrapeOptions::Shopify(CrawlShopifyOptions {
        tag_regexes: Some(tag_regexes),
        ..
    })) = scrape_request.crawl_options.scrape_options.clone()
    {
        // Create al regexes, if the regex is invalid, skip it
        let regexes: Vec<(regex::Regex, String)> = tag_regexes
            .iter()
            .filter_map(|pattern| {
                regex::Regex::new(pattern)
                    .ok()
                    .map(|regex| (regex, pattern.to_string()))
            })
            .collect();

        // Go through all the tags, and find the ones that match the regexes
        let tags_string: String = product
            .tags
            .iter()
            .filter_map(|tag| {
                // Add the pattern if the tag matches the regex
                regexes
                    .iter()
                    .find(|(regex, _)| regex.is_match(tag))
                    .map(|(_, pattern)| format!("<span>{}</span>", pattern.clone()))
            })
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>()
            .join("");

        chunk_html = format!("<div>{}</div>\n\n<div>{}</div>", chunk_html, tags_string);
    }

    let group_variants = if let Some(ScrapeOptions::Shopify(CrawlShopifyOptions {
        group_variants: Some(group_variants),
        ..
    })) = scrape_request.crawl_options.scrape_options
    {
        group_variants
    } else {
        true
    };

    let semantic_boost_phrase = if group_variants {
        variant.title.clone().unwrap_or_default()
    } else {
        product.title.clone().unwrap_or_default()
    };

    let fulltext_boost_phrase = if group_variants {
        variant.title.clone().unwrap_or_default()
    } else {
        product.title.clone().unwrap_or_default()
    };

    Ok(ChunkReqPayload {
        chunk_html: Some(chunk_html),
        link: Some(link),
        tag_set: Some(product.tags.clone()),
        num_value: variant.price.clone().unwrap_or_default().parse().ok(),
        metadata: serde_json::to_value(product.clone()).ok(),
        tracking_id: if group_variants {
            Some(variant.id.to_string())
        } else {
            Some(product.id.to_string())
        },
        group_tracking_ids: if group_variants {
            Some(vec![product.id.to_string()])
        } else {
            None
        },
        image_urls: Some(image_urls),
        fulltext_boost: if scrape_request.crawl_options.boost_titles.unwrap_or(true) {
            Some(FullTextBoost {
                phrase: fulltext_boost_phrase,
                boost_factor: 1.3,
            })
        } else {
            None
        },
        semantic_boost: if scrape_request.crawl_options.boost_titles.unwrap_or(true) {
            Some(SemanticBoost {
                phrase: semantic_boost_phrase,
                distance_factor: 0.3,
            })
        } else {
            None
        },
        convert_html_to_text: Some(true),
        ..Default::default()
    })
}

#[allow(clippy::print_stdout)]
async fn get_chunks_with_firecrawl(
    scrape_request: CrawlRequest,
    pool: web::Data<Pool>,
) -> Result<(Vec<ChunkReqPayload>, usize), ServiceError> {
    let mut chunks = vec![];
    let mut spec = None;

    if let Some(ScrapeOptions::OpenApi(openapi_options)) =
        scrape_request.crawl_options.scrape_options.clone()
    {
        let client = reqwest::Client::new();

        let schema = match client
            .get(openapi_options.openapi_schema_url.clone())
            .send()
            .await
        {
            Ok(response) => {
                let schema = response.text().await.map_err(|e| {
                    log::error!("Error getting schema: {:?}", e);
                    ServiceError::InternalServerError("Error getting schema".to_string())
                })?;

                Some(schema.replace("18446744073709552000", "2147483647"))
            }
            Err(e) => {
                log::error!("Error getting schema: {:?}", e);
                None
            }
        };

        if let Some(schema) = schema {
            spec = match oas3::from_str(&schema) {
                Ok(schema) => Some(schema),
                Err(e) => {
                    log::error!("Error deserializing schema: {:?}", e);
                    None
                }
            };
        }
    }

    let ingest_result;
    loop {
        let temp_result = get_crawl_from_firecrawl(scrape_request.scrape_id)
            .await
            .map_err(|e| {
                log::error!("Error getting scrape request: {:?}", e);
                ServiceError::InternalServerError("Error getting scrape request".to_string())
            })?;
        if temp_result.status == Status::Completed {
            ingest_result = temp_result;
            break;
        } else if temp_result.status == Status::Scraping {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else if temp_result.status == Status::Failed {
            update_crawl_status(scrape_request.id, CrawlStatus::Failed, pool.clone())
                .await
                .map_err(|e| {
                    log::error!("Error updating crawl status: {:?}", e);
                    ServiceError::InternalServerError("Error updating crawl status".to_string())
                })?;

            return Err(ServiceError::InternalServerError(
                "Scrape failed".to_string(),
            ));
        }
    }

    update_crawl_status(
        scrape_request.id,
        CrawlStatus::GotResponseBackFromFirecrawl,
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Error updating crawl status: {:?}", e);
        ServiceError::InternalServerError("Error updating crawl status".to_string())
    })?;

    log::info!(
        "Got response back from firecrawl for scrape_id: {}",
        scrape_request.id
    );

    let data = ingest_result.data.unwrap_or_default();

    log::info!("Processing {} documents from scrape", data.len());

    let page_count = data.len();

    for page in data {
        let crawl_doc = match page {
            Some(page) => page,
            None => continue,
        };

        if crawl_doc.metadata.status_code != Some(200) {
            log::error!("Error getting metadata for page: {:?}", crawl_doc.metadata);
            continue;
        }

        let page_link = crawl_doc
            .metadata
            .source_url
            .clone()
            .unwrap_or_default()
            .trim_end_matches("/")
            .to_string();
        if page_link.is_empty() {
            println!(
                "Error page source_url is not present for page_metadata: {:?}",
                crawl_doc.metadata
            );
            continue;
        }

        let page_title = crawl_doc.metadata.og_title.clone().unwrap_or_default();
        let page_description = crawl_doc
            .metadata
            .og_description
            .clone()
            .unwrap_or(crawl_doc.metadata.description.unwrap_or_default().clone());
        let page_html = crawl_doc.html.clone().unwrap_or_default();
        let page_tags = get_tags(page_link.clone());

        if let Some(spec) = &spec {
            if let Some(ScrapeOptions::OpenApi(ref openapi_options)) =
                scrape_request.crawl_options.scrape_options
            {
                if page_tags.contains(&openapi_options.openapi_tag) {
                    if let Some(last_tag) = page_tags.last() {
                        // try to find a operation in the spec with an operation_id that matches the last tag directly, with - replaced by _ or vice versa
                        let operation = spec.operations().find(|(_, _, operation)| {
                            let operation_id_find =
                                if let Some(operation_id) = operation.operation_id.clone() {
                                    operation_id == *last_tag
                                        || operation_id.to_lowercase().replace("_", "-")
                                            == *last_tag.to_lowercase()
                                        || operation_id.to_lowercase().replace("-", "_")
                                            == *last_tag.to_lowercase()
                                } else {
                                    false
                                };
                            if operation_id_find {
                                return true;
                            }

                            let summary_match = if let Some(summary) = operation.summary.clone() {
                                summary == *last_tag
                                    || summary.to_lowercase().replace(" ", "-")
                                        == *last_tag.to_lowercase()
                                    || summary.to_lowercase().replace(" ", "_")
                                        == *last_tag.to_lowercase()
                            } else {
                                false
                            };
                            if summary_match {
                                return true;
                            }

                            if page_tags.len() < 2 {
                                return false;
                            }

                            if let Some(second_to_last_tag) = page_tags.get(page_tags.len() - 2) {
                                let combined_tag =
                                    format!("{}-{}", second_to_last_tag, last_tag).to_lowercase();
                                if let Some(operation_id) = operation.operation_id.clone() {
                                    operation_id == combined_tag
                                        || operation_id.to_lowercase().replace("_", "-")
                                            == combined_tag.to_lowercase()
                                        || operation_id.to_lowercase().replace("-", "_")
                                            == combined_tag.to_lowercase()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });

                        if operation.is_none() {
                            println!(
                                "No operation found for tag: {} in spec: {}",
                                last_tag, page_link
                            );
                        }

                        if let Some((path, method, operation)) = operation {
                            let mut metadata = json!({
                                "url": page_link.clone(),
                                "operation_id": operation.operation_id.clone(),
                            });

                            let heading = format!("{} {}", method.to_string().to_uppercase(), path);
                            let mut semantic_boost_phrase = heading.clone();
                            let mut fulltext_boost_phrase = heading.clone();
                            metadata["heading"] = json!(heading.clone());

                            let mut chunk_html = format!(
                                "<h2><span class=\"openapi-method\">{}</span> ",
                                method.to_string().to_uppercase(),
                            )
                            .replace("\\", "");

                            if let Some(summary) = operation.summary.clone() {
                                if !summary.is_empty() {
                                    fulltext_boost_phrase
                                        .push_str(format!("\n\n{}", summary).as_str());
                                    semantic_boost_phrase
                                        .push_str(format!("\n\n{}", summary).as_str());

                                    metadata["summary"] = json!(summary.clone());

                                    chunk_html.push_str(format!("{}</h2>", summary).as_str());
                                    chunk_html.push_str(format!("\n\n<p>{}</p>", path).as_str());
                                } else {
                                    chunk_html.push_str(format!("{}</h2>", path).as_str());
                                }
                            } else {
                                chunk_html.push_str(format!("{}</h2>", path).as_str());
                            }
                            if let Some(description) = operation.description.clone() {
                                if !description.is_empty() {
                                    semantic_boost_phrase
                                        .push_str(format!("\n\n{}", description).as_str());

                                    metadata["description"] = json!(description.clone());

                                    chunk_html
                                        .push_str(format!("\n\n<p>{}</p>", description).as_str());
                                }
                            }

                            // TODO: include request body and response bodies as markdown tables for better RAG

                            let mut tag_set = page_tags.clone();
                            tag_set.push("openapi-route".to_string());

                            let chunk = ChunkReqPayload {
                                chunk_html: Some(chunk_html.clone()),
                                link: Some(page_link.clone()),
                                tag_set: Some(tag_set),
                                metadata: Some(json!(metadata)),
                                tracking_id: Some(hash_function(&format!(
                                    "{}{}",
                                    page_link.trim_end_matches("/"),
                                    heading.clone()
                                ))),
                                upsert_by_tracking_id: Some(true),
                                group_tracking_ids: Some(vec![page_link.clone()]),
                                fulltext_boost: if scrape_request
                                    .crawl_options
                                    .boost_titles
                                    .unwrap_or(true)
                                {
                                    Some(FullTextBoost {
                                        phrase: fulltext_boost_phrase,
                                        boost_factor: 1.3,
                                    })
                                } else {
                                    None
                                },
                                semantic_boost: if scrape_request
                                    .crawl_options
                                    .boost_titles
                                    .unwrap_or(true)
                                {
                                    Some(SemanticBoost {
                                        phrase: semantic_boost_phrase,
                                        distance_factor: 0.3,
                                    })
                                } else {
                                    None
                                },
                                convert_html_to_text: Some(true),
                                ..Default::default()
                            };
                            chunks.push(chunk);
                        }

                        continue;
                    }
                }
            }
        }

        let chunked_html = chunk_html(
            &page_html.clone(),
            scrape_request.crawl_options.heading_remove_strings.clone(),
            scrape_request.crawl_options.body_remove_strings.clone(),
        );

        for chunk in chunked_html {
            let heading = chunk.0.clone();
            let chunk_html = chunk.1.clone();

            if chunk_html.is_empty() {
                println!("Skipping empty chunk for page: {}", page_link);
                continue;
            }

            let mut metadata = json!({
                "url": page_link.clone(),
                "hierarchy": chunk.0.clone(),
            });

            let mut semantic_boost_phrase = heading.clone();
            let mut fulltext_boost_phrase = heading.clone();
            metadata["heading"] = json!(heading.clone());

            if !page_title.is_empty() {
                semantic_boost_phrase.push_str(format!("\n\n{}", page_title).as_str());
                fulltext_boost_phrase.push_str(format!("\n\n{}", page_title).as_str());

                metadata["title"] = json!(page_title.clone());
                metadata["hierarchy"]
                    .as_array_mut()
                    .unwrap_or(&mut vec![])
                    .insert(0, json!(page_title.clone()));
            }
            if !page_description.is_empty() {
                semantic_boost_phrase.push_str(format!("\n\n{}", page_description).as_str());

                metadata["description"] = json!(page_description.clone());
            }

            let tracking_hash_val = if heading.is_empty() {
                chunk_html.clone()
            } else {
                heading.clone()
            };

            let chunk = ChunkReqPayload {
                chunk_html: Some(chunk_html.clone()),
                link: Some(page_link.clone()),
                tag_set: Some(page_tags.clone()),
                metadata: Some(json!(metadata)),
                tracking_id: Some(hash_function(&format!(
                    "{}{}",
                    page_link
                        .trim_end_matches("/")
                        .split("/")
                        .collect::<Vec<&str>>()
                        .split_at(3)
                        .1
                        .join("/"),
                    tracking_hash_val
                ))),
                upsert_by_tracking_id: Some(true),
                group_tracking_ids: Some(vec![if !page_title.is_empty() {
                    page_title.clone()
                } else {
                    page_link.clone()
                }]),
                fulltext_boost: if !fulltext_boost_phrase.is_empty()
                    && scrape_request.crawl_options.boost_titles.unwrap_or(true)
                {
                    Some(FullTextBoost {
                        phrase: fulltext_boost_phrase,
                        boost_factor: 1.3,
                    })
                } else {
                    None
                },
                semantic_boost: if !semantic_boost_phrase.is_empty()
                    && scrape_request.crawl_options.boost_titles.unwrap_or(true)
                {
                    Some(SemanticBoost {
                        phrase: semantic_boost_phrase,
                        distance_factor: 0.3,
                    })
                } else {
                    None
                },
                convert_html_to_text: Some(true),
                ..Default::default()
            };
            chunks.push(chunk);
        }
    }

    Ok((chunks, page_count))
}

#[allow(clippy::print_stdout)]
async fn crawl(
    crawl_request: CrawlRequest,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<ScrapeReport, ServiceError> {
    log::info!("Starting crawl for scrape_id: {}", crawl_request.id);
    let (page_count, chunks_created) = if let Some(ScrapeOptions::Shopify(_)) =
        crawl_request.crawl_options.scrape_options.clone()
    {
        let mut cur_page = 1;
        let mut chunk_count = 0;

        loop {
            let mut chunks: Vec<ChunkReqPayload> = Vec::new();
            let cleaned_url = crawl_request.url.trim_end_matches("/");
            let url = format!("{}/products.json?page={}", cleaned_url, cur_page);

            let response: ShopifyResponse = ureq::AgentBuilder::new()
                .tls_connector(Arc::new(native_tls::TlsConnector::new().map_err(|_| {
                    ServiceError::InternalServerError(
                        "Failed to acquire tls connection".to_string(),
                    )
                })?))
                .build()
                .get(&url)
                .call()
                .map_err(|e| ServiceError::InternalServerError(format!("Failed to fetch: {}", e)))?
                .into_json()
                .map_err(|e| {
                    ServiceError::InternalServerError(format!("Failed to parse JSON: {}", e))
                })?;

            if response.products.is_empty() {
                break;
            }

            for product in response.products {
                if product.variants.len() == 1 {
                    chunks.push(create_chunk_req_payload(
                        &product,
                        &product.variants[0],
                        cleaned_url,
                        &crawl_request,
                    )?);
                } else {
                    for variant in &product.variants {
                        chunks.push(create_chunk_req_payload(
                            &product,
                            variant,
                            cleaned_url,
                            &crawl_request,
                        )?);
                    }
                }
            }

            let chunks_to_upload = chunks.chunks(120);

            for chunk in chunks_to_upload {
                let (chunk_ingestion_message, chunk_metadatas) =
                    create_chunk_metadata(chunk.to_vec(), crawl_request.dataset_id).await?;

                let mut redis_conn = redis_pool
                    .get()
                    .await
                    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

                if !chunk_metadatas.is_empty() {
                    let serialized_message: String =
                        serde_json::to_string(&chunk_ingestion_message).map_err(|_| {
                            ServiceError::BadRequest(
                                "Failed to Serialize BulkUploadMessage".to_string(),
                            )
                        })?;

                    redis::cmd("lpush")
                        .arg("ingestion")
                        .arg(&serialized_message)
                        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
                        .await
                        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
                }
            }

            chunk_count += chunks.len();
            cur_page += 1;
        }

        (cur_page, chunk_count)
    } else {
        let (chunks, page_count) =
            get_chunks_with_firecrawl(crawl_request.clone(), pool.clone()).await?;
        let chunks_to_upload = chunks.chunks(120);

        for batch in chunks_to_upload {
            let (chunk_ingestion_message, chunk_metadatas) =
                create_chunk_metadata(batch.to_vec(), crawl_request.dataset_id).await?;

            let mut redis_conn = redis_pool
                .get()
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

            if !chunk_metadatas.is_empty() {
                let serialized_message: String = serde_json::to_string(&chunk_ingestion_message)
                    .map_err(|_| {
                        ServiceError::BadRequest(
                            "Failed to Serialize BulkUploadMessage".to_string(),
                        )
                    })?;

                redis::cmd("lpush")
                    .arg("ingestion")
                    .arg(&serialized_message)
                    .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
                    .await
                    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            }
        }
        (page_count, chunks.len())
    };

    update_crawl_status(
        crawl_request.scrape_id,
        CrawlStatus::Completed,
        pool.clone(),
    )
    .await?;

    update_next_crawl_at(
        crawl_request.scrape_id,
        crawl_request.next_crawl_at + crawl_request.interval,
        pool.clone(),
    )
    .await?;

    Ok(ScrapeReport {
        request_id: crawl_request.id,
        pages_scraped: page_count,
        chunks_created,
    })
}

#[allow(clippy::print_stdout)]
async fn scrape_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
    event_queue: actix_web::web::Data<EventQueue>,
) {
    log::info!("Starting scrape worker service thread");

    let mut redis_conn_sleep = std::time::Duration::from_secs(1);

    #[allow(unused_assignments)]
    let mut opt_redis_connection = None;

    loop {
        let borrowed_redis_connection = match redis_pool.get().await {
            Ok(redis_connection) => Some(redis_connection),
            Err(err) => {
                log::error!("Failed to get redis connection outside of loop: {:?}", err);
                None
            }
        };

        if borrowed_redis_connection.is_some() {
            opt_redis_connection = borrowed_redis_connection;
            break;
        }

        tokio::time::sleep(redis_conn_sleep).await;
        redis_conn_sleep = std::cmp::min(redis_conn_sleep * 2, std::time::Duration::from_secs(300));
    }

    let mut redis_connection =
        opt_redis_connection.expect("Failed to get redis connection outside of loop");

    let mut broken_pipe_sleep = std::time::Duration::from_secs(10);

    loop {
        if should_terminate.load(Ordering::Relaxed) {
            log::info!("Shutting down");
            break;
        }

        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("brpoplpush")
            .arg("scrape_queue")
            .arg("scrape_processing")
            .arg(1.0)
            .query_async(&mut redis_connection.clone())
            .await;

        let serialized_message = if let Ok(payload) = payload_result {
            broken_pipe_sleep = std::time::Duration::from_secs(10);

            if payload.is_empty() {
                continue;
            }

            payload
                .first()
                .expect("Payload must have a first element")
                .clone()
        } else {
            log::error!("Unable to process {:?}", payload_result);

            if payload_result.is_err_and(|err| err.is_io_error()) {
                tokio::time::sleep(broken_pipe_sleep).await;
                broken_pipe_sleep =
                    std::cmp::min(broken_pipe_sleep * 2, std::time::Duration::from_secs(300));
            }

            continue;
        };
        let crawl_request: CrawlRequest =
            serde_json::from_str(&serialized_message).expect("Failed to parse file message");

        match update_crawl_status(crawl_request.scrape_id, CrawlStatus::Pending, pool.clone()).await
        {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to update crawl status: {:?}", err);
                continue;
            }
        }

        event_queue
            .send(ClickHouseEvent::WorkerEvent(
                WorkerEvent::from_details(
                    crawl_request.dataset_id,
                    models::EventType::CrawlStarted {
                        scrape_id: crawl_request.scrape_id,
                        crawl_options: crawl_request.clone().crawl_options,
                    },
                )
                .into(),
            ))
            .await;

        match crawl(crawl_request.clone(), pool.clone(), redis_pool.clone()).await {
            Ok(scrape_report) => {
                log::info!("Scrape job completed: {:?}", scrape_report);

                event_queue
                    .send(ClickHouseEvent::WorkerEvent(
                        WorkerEvent::from_details(
                            crawl_request.dataset_id,
                            models::EventType::CrawlCompleted {
                                scrape_id: scrape_report.request_id,
                                pages_crawled: scrape_report.pages_scraped,
                                chunks_created: scrape_report.chunks_created,
                                crawl_options: crawl_request.crawl_options,
                            },
                        )
                        .into(),
                    ))
                    .await;

                match update_crawl_status(
                    scrape_report.request_id,
                    CrawlStatus::Completed,
                    pool.clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to update crawl status: {:?}", err);
                        continue;
                    }
                }

                let _ = redis::cmd("LREM")
                    .arg("scrape_processing")
                    .arg(1)
                    .arg(serialized_message)
                    .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_connection)
                    .await;
            }
            Err(err) => {
                log::error!("Failed to scrape website: {:?}", err);

                event_queue
                    .send(ClickHouseEvent::WorkerEvent(
                        WorkerEvent::from_details(
                            crawl_request.dataset_id,
                            models::EventType::CrawlFailed {
                                scrape_id: crawl_request.id,
                                crawl_options: crawl_request.crawl_options.clone(),
                                error: format!("{:?}", err),
                            },
                        )
                        .into(),
                    ))
                    .await;

                let _ = readd_error_to_queue(crawl_request, err, redis_pool.clone()).await;
            }
        };
    }
}

fn main() {
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

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
        .block_on(async move {
            let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
            let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
                .unwrap_or("2".to_string())
                .parse()
                .unwrap_or(2);

            let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url)
                .expect("Failed to connect to redis");

            let redis_pool = bb8_redis::bb8::Pool::builder()
                .max_size(redis_connections)
                .connection_timeout(std::time::Duration::from_secs(2))
                .build(redis_manager)
                .await
                .expect("Failed to create redis pool");

            let web_redis_pool = actix_web::web::Data::new(redis_pool);

            let should_terminate = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
                .expect("Failed to register shutdown hook");

            let event_queue = if std::env::var("USE_ANALYTICS")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap_or(false)
            {
                log::info!("Analytics enabled");

                let clickhouse_client = clickhouse::Client::default()
                    .with_url(
                        std::env::var("CLICKHOUSE_URL")
                            .unwrap_or("http://localhost:8123".to_string()),
                    )
                    .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
                    .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
                    .with_database(
                        std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()),
                    )
                    .with_option("async_insert", "1")
                    .with_option("wait_for_async_insert", "0");

                let mut event_queue = EventQueue::new(clickhouse_client.clone());
                event_queue.start_service();
                event_queue
            } else {
                log::info!("Analytics disabled");
                EventQueue::default()
            };
            let web_event_queue = actix_web::web::Data::new(event_queue);
            scrape_worker(should_terminate, web_redis_pool, web_pool, web_event_queue).await
        });
}

pub async fn readd_error_to_queue(
    mut payload: CrawlRequest,
    error: ServiceError,
    redis_pool: actix_web::web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    payload.attempt_number += 1;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("LREM")
        .arg("scrape_processing")
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    if payload.attempt_number == 3 {
        log::error!("Failed to insert data 3 times quitting {:?}", error);

        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        redis::cmd("lpush")
            .arg("dead_letters_scrape")
            .arg(old_payload_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        return Err(ServiceError::InternalServerError(format!(
            "Failed to create new qdrant point: {:?}",
            error
        )));
    }

    let new_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    log::error!(
        "Failed to insert data, re-adding {:?} retry: {:?}",
        error,
        payload.attempt_number
    );

    redis::cmd("lpush")
        .arg("scrape_queue")
        .arg(&new_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
