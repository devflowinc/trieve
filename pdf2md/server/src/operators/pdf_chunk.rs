use crate::{errors::ServiceError, get_env, models::ChunkClickhouse};
use base64::Engine;
use image::{codecs::png::PngEncoder, ImageEncoder};
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionParametersBuilder, ChatMessage, ChatMessageContent,
        ChatMessageImageContentPart, ImageUrlType,
    },
};
use pdf2image::{image::DynamicImage, PDF};
use regex::Regex;
use s3::creds::time::OffsetDateTime;

const CHUNK_SYSTEM_PROMPT: &str = "    
    Convert the following PDF page to markdown.
    Return only the markdown with no explanation text.
    Do not exclude any content from the page.";

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

fn get_default_openai_client() -> Client {
    let base_url = get_env!("LLM_BASE_URL", "LLM_BASE_URL should be set").into();

    let llm_api_key: String = get_env!(
        "LLM_API_KEY",
        "LLM_API_KEY for openrouter or self-hosted should be set"
    )
    .into();

    Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    }
}

async fn get_pages_from_image(
    img: DynamicImage,
    prev_md_doc: Option<String>,
    page: u32,
    task_id: String,
    client: Client,
) -> Result<ChunkClickhouse, ServiceError> {
    let llm_model: String = get_env!("LLM_MODEL", "LLM_MODEL should be set").into();

    let data_url = get_data_url_from_image(img)?;

    let mut messages = vec![
        ChatMessage::System {
            content: (ChatMessageContent::Text(CHUNK_SYSTEM_PROMPT.to_string())),
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
                "Markdown must maintain consistent formatting with the following page: \n\n {}",
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

    Ok(ChunkClickhouse {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: task_id.clone(),
        content: format_markdown(&content),
        metadata: serde_json::json!({
            "page": page,
        })
        .to_string(),
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

pub async fn chunk_pdf(
    data: Vec<u8>,
    task_id: String,
    page_range: (u32, u32),
) -> Result<Vec<ChunkClickhouse>, ServiceError> {
    let pdf = PDF::from_bytes(data)
        .map_err(|_| ServiceError::BadRequest("Failed to open PDF file".to_string()))?;

    let pages = pdf
        .render(pdf2image::Pages::All, None)
        .map_err(|_| ServiceError::BadRequest("Failed to render PDF file".to_string()))?;

    let mut result_pages = vec![];

    let client = get_default_openai_client();
    let mut prev_md_doc = None;

    for (page_image, page_num) in pages.into_iter().zip(page_range.0..page_range.1) {
        let page = get_pages_from_image(
            page_image,
            prev_md_doc,
            page_num,
            task_id.clone(),
            client.clone(),
        )
        .await?;
        prev_md_doc = Some(page.content.clone());
        log::info!("Page {} processed", page_num);

        result_pages.push(page);
    }

    Ok(result_pages)
}
