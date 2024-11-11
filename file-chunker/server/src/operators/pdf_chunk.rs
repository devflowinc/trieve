use crate::{errors::ServiceError, get_env, models::ChunkClickhouse};
use base64::Engine;
use image::{codecs::png::PngEncoder, ImageEncoder};
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionParametersBuilder, ChatCompletionResponseFormat, ChatMessage,
        ChatMessageContent, ChatMessageImageContentPart, ImageUrlType, JsonSchemaBuilder,
    },
};
use pdf2image::{image::DynamicImage, PDF};
use s3::creds::time::OffsetDateTime;
use serde::Deserialize;

const CHUNK_SYSTEM_PROMPT: &str = "You are an image transcription and chunking tool. You need to transcribe the text in the image, but separate it based on paragraph, heading etc. Return the chunks as a json array. Each chunk should be written as as markdown. Each chunk should not exceed more than 50 words";

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

#[derive(Debug, Deserialize)]
struct ChatResponse {
    pub chunks: Vec<String>,
}

async fn get_chunks_from_image(
    img: DynamicImage,
    page: usize,
    task_id: String,
    client: Client,
) -> Result<Vec<ChunkClickhouse>, ServiceError> {
    let llm_model: String = get_env!("LLM_MODEL", "LLM_MODEL should be set").into();

    let data_url = get_data_url_from_image(img)?;

    let messages = vec![
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
        .model(llm_model)
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

    let structured_output: ChatResponse = serde_json::from_str(content.as_ref()).map_err(|_| {
        ServiceError::InternalServerError(
            "Failed to parse chunks from chat completion response".to_string(),
        )
    })?;

    let mapped: Vec<ChunkClickhouse> = structured_output
        .chunks
        .into_iter()
        .map(|chunk| ChunkClickhouse {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.clone(),
            content: chunk,
            metadata: serde_json::json!({
                "page": page,
            })
            .to_string(),
            created_at: OffsetDateTime::now_utc(),
        })
        .collect();

    Ok(mapped)
}

pub async fn chunk_pdf(
    data: Vec<u8>,
    task_id: String,
) -> Result<Vec<ChunkClickhouse>, ServiceError> {
    let pdf = PDF::from_bytes(data)
        .map_err(|_| ServiceError::BadRequest("Failed to open PDF file".to_string()))?;

    let pages = pdf
        .render(pdf2image::Pages::All, None)
        .map_err(|_| ServiceError::BadRequest("Failed to render PDF file".to_string()))?;

    let mut result_chunks = vec![];

    let client = get_default_openai_client();

    for (page, page_image) in pages.into_iter().enumerate() {
        let chunks =
            get_chunks_from_image(page_image, page, task_id.clone(), client.clone()).await?;
        result_chunks.extend(chunks);
    }

    Ok(result_chunks)
}
