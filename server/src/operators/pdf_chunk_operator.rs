use std::{path::PathBuf, sync::Arc};

use base64::Engine;
use image::{codecs::png::PngEncoder, ImageEncoder};
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionParametersBuilder, ChatCompletionResponseFormat, ChatMessage,
        ChatMessageContent, ImageUrl, ImageUrlType, JsonSchemaBuilder,
    },
};
use pdf2image::{image::DynamicImage, PDF};
use serde::Deserialize;
use tokio::task::JoinSet;

use crate::{get_env, ServiceError};

const CHUNK_SYSTEM_PROMPT: &str = "You are an image transcription and chunking tool. You need to transcribe the text in the image, but separate it based on paragraph, heading etc. Return the chunks as a json array. Each chunk should be written as as markdown. Each chunk should not exceed more than 50 words";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PDFChunk {
    content: String,
    page: usize,
}

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
    let base_url = "https://openrouter.ai/api/v1".to_string();

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

#[derive(Debug, Deserialize)]
struct ChatResponse {
    pub chunks: Vec<String>,
}

async fn get_chunks_from_image(
    img: DynamicImage,
    page: usize,
    client: Arc<Client>,
) -> Result<Vec<PDFChunk>, ServiceError> {
    // Convert img to dataurl format
    let data_url = get_data_url_from_image(img)?;

    let messages = vec![
        ChatMessage::System {
            content: (ChatMessageContent::Text(CHUNK_SYSTEM_PROMPT.to_string())),
            name: None,
        },
        ChatMessage::User {
            content: ChatMessageContent::ImageUrl(vec![ImageUrl {
                r#type: "image_url".to_string(),
                text: None,
                image_url: ImageUrlType {
                    url: data_url,
                    detail: None,
                },
            }]),
            name: None,
        },
    ];

    let result_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "chunks": {
                "type": "array",
                "items": {
                    "type": "string",
                }
            },
        },
        "required": ["chunks"],
        "additionalProperties": false
    });

    let params = ChatCompletionParametersBuilder::default()
        .model("gpt-4o-mini")
        .messages(messages)
        .response_format(ChatCompletionResponseFormat::JsonSchema(
            JsonSchemaBuilder::default()
                .name("result_chunks")
                .schema(result_schema)
                .strict(true)
                .build()
                .expect("Can build"),
        ))
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
        .get(0)
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

    let structured_output: ChatResponse = serde_json::from_str(content.as_ref()).map_err(|_| {
        ServiceError::InternalServerError(
            "Failed to parse chunks from chat completion response".to_string(),
        )
    })?;

    let mapped: Vec<PDFChunk> = structured_output
        .chunks
        .into_iter()
        .map(|chunk| PDFChunk {
            content: chunk,
            page,
        })
        .collect();

    Ok(mapped)
}

pub async fn chunk_pdf(pdf_path: PathBuf) -> Result<Vec<PDFChunk>, ServiceError> {
    let pdf = PDF::from_file(pdf_path)
        .map_err(|_| ServiceError::BadRequest("Failed to open PDF file".to_string()))?;

    let pages = pdf
        .render(pdf2image::Pages::All, None)
        .map_err(|_| ServiceError::BadRequest("Failed to render PDF file".to_string()))?;

    let mut join_set = JoinSet::new();
    let page_count = pages.len();

    let client = Arc::new(get_default_openai_client());

    for (page, page_image) in pages.into_iter().enumerate() {
        let client = client.clone();
        join_set.spawn(async move { get_chunks_from_image(page_image, page, client).await });
    }

    let mut chunk_results = Vec::with_capacity(page_count);
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(t) => {
                chunk_results.push(t);
            }
            Err(err) => {
                return Err(ServiceError::BadRequest(format!(
                    "Failed to get chunks from image {:?}",
                    err
                )));
            }
        }
    }

    let flattened_chunks: Vec<PDFChunk> = chunk_results
        .into_iter()
        .filter_map(Result::ok) // Keep only the Ok variants
        .flatten() // Flatten Vec<Vec<PDFChunk>> to Vec<PDFChunk>
        .collect();

    Ok(flattened_chunks)
}
