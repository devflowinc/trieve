use crate::models::RedisPool;
use crate::operators::clickhouse::get_last_page;
use crate::{
    errors::ServiceError,
    get_env,
    models::{ChunkClickhouse, ChunkingParams, ChunkingTask},
    operators::{clickhouse::insert_page, webhook_template::send_webhook},
};
use base64::Engine;
use image::{codecs::png::PngEncoder, ImageEncoder};
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionParametersBuilder, ChatMessage, ChatMessageContent,
        ChatMessageImageContentPart, ImageUrlType,
    },
};
use pdf2image::image::DynamicImage;
use regex::Regex;
use s3::creds::time::OffsetDateTime;

const CHUNK_SYSTEM_PROMPT: &str = "    
Convert this PDF page to markdown formatting, following these requirements:

1. Break the content into logical sections with clear markdown headings (# for main sections, ## for subsections, etc.)
2. Create section headers that accurately reflect the content and hierarchy of each part
3. Include all body content from the page
4. Exclude any PDF headers and footers
5. Return only the formatted markdown without any explanatory text
6. Match the original document's content organization but with explicit markdown structure

Please provide the markdown version using this structured approach.
";

fn get_data_url_from_image(img: DynamicImage) -> Result<String, ServiceError> {
    let mut encoded = Vec::new();

    let png_encoder = PngEncoder::new(&mut encoded);

    png_encoder
        .write_image(
            img.as_bytes(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|_| ServiceError::BadRequest("Failed to encode image".to_string()))?;

    // Encode result base64 - utf-8

    let encoded = base64::prelude::BASE64_STANDARD.encode(encoded);

    let prefix = "data:image/png;base64,";

    let final_encoded = format!("{prefix}{encoded}");

    Ok(final_encoded)
}

fn get_llm_client(params: ChunkingParams) -> Client {
    let base_url = get_env!("LLM_BASE_URL", "LLM_BASE_URL should be set").into();

    let llm_api_key: String = params.llm_api_key.unwrap_or(
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into(),
    );

    Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    }
}

async fn get_markdown_from_image(
    img: DynamicImage,
    prev_md_doc: Option<String>,
    page: u32,
    task: ChunkingTask,
    client: Client,
) -> Result<ChunkClickhouse, ServiceError> {
    let llm_model: String = task
        .params
        .llm_model
        .unwrap_or(get_env!("LLM_MODEL", "LLM_MODEL should be set").into());

    let data_url = get_data_url_from_image(img)?;

    let mut messages = vec![
        ChatMessage::System {
            content: (ChatMessageContent::Text(
                task.params
                    .system_prompt
                    .unwrap_or(CHUNK_SYSTEM_PROMPT.to_string()),
            )),
            name: None,
        },
        ChatMessage::User {
            content: ChatMessageContent::ImageContentPart(vec![ChatMessageImageContentPart {
                r#type: "image_url".to_string(),
                image_url: ImageUrlType {
                    url: data_url,
                    detail: None,
                },
            }]),
            name: None,
        },
    ];

    if let Some(prev_md_doc) = prev_md_doc {
        let prev_md_doc_message = ChatMessage::System {
            content: ChatMessageContent::Text(format!(
                "Markdown must maintain consistent formatting with the following page, DO NOT INCLUDE CONTENT FROM THIS PAGE IN YOUR RESPONSE: \n\n {}",
                prev_md_doc
            )),
            name: None,
        };

        messages.insert(1, prev_md_doc_message);
    }

    let params = ChatCompletionParametersBuilder::default()
        .model(llm_model)
        .messages(messages)
        .build()
        .map_err(|_| {
            ServiceError::BadRequest("Failed to build chat completion parameters".to_string())
        })?;

    let response = client.chat().create(params).await.map_err(|e| {
        ServiceError::InternalServerError(
            format!("Failed to get chat completion response: {:?}", e).to_string(),
        )
    })?;

    let message_response = response
        .choices
        .first()
        .ok_or(ServiceError::InternalServerError(
            "No choices in chat completion response".to_string(),
        ))?;

    let content = match &message_response.message {
        ChatMessage::Assistant {
            content: Some(ChatMessageContent::Text(content)),
            ..
        } => content.clone(),

        _ => {
            return Err(ServiceError::InternalServerError(
                "Unexpected message response".to_string(),
            ))
        }
    };

    let mut metadata = serde_json::json!({});
    if let Some(usage) = response.usage {
        metadata = serde_json::json!(usage);
    }

    Ok(ChunkClickhouse {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: task.id.to_string().clone(),
        content: format_markdown(&content),
        page,
        usage: metadata.to_string(),
        created_at: OffsetDateTime::now_utc(),
    })
}

fn format_markdown(text: &str) -> String {
    let formatted_markdown = Regex::new(r"(?m)^```[a-z]*\n([\s\S]*?)\n```$")
        .unwrap()
        .replace_all(text, "$1");
    let formatted_markdown = Regex::new(r"(?m)^```\n([\s\S]*?)\n```$")
        .unwrap()
        .replace_all(&formatted_markdown, "$1");
    formatted_markdown.into_owned()
}

pub async fn chunk_sub_pages(
    data: Vec<u8>,
    task: ChunkingTask,
    clickhouse_client: &clickhouse::Client,
    redis_pool: &RedisPool,
) -> Result<(), ServiceError> {
    log::info!("Chunking page {} for {:?}", task.page_num, task.id);

    let page = image::load_from_memory_with_format(&data, image::ImageFormat::Jpeg)
        .map_err(|err| ServiceError::BadRequest(format!("Failed to render PDF file {:?}", err)))?;

    let client = get_llm_client(task.params.clone());

    let prev_md_doc = if task.page_num > 1 {
        let prev_page = get_last_page(task.id, clickhouse_client).await?;

        prev_page.map(|p| p.content)
    } else {
        None
    };

    let page = get_markdown_from_image(
        page,
        prev_md_doc,
        task.page_num,
        task.clone(),
        client.clone(),
    )
    .await?;

    let data = insert_page(task.clone(), page.clone(), clickhouse_client, redis_pool).await?;

    send_webhook(
        task.params.webhook_url.clone(),
        task.params.webhook_payload_template.clone(),
        data,
    )
    .await?;

    log::info!("Page {} processed", task.page_num);

    Ok(())
}
