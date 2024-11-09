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
use reqwest::multipart::{Form, Part};
use s3::creds::time::OffsetDateTime;
use serde::{Deserialize, Deserializer, Serialize};

// The pdla server in some cases returns empty string if text can't be transcribed.
fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

const CHUNK_SYSTEM_PROMPT: &str = "
You are an image description/transcription tool. If the image is of text, transcribe it. Return the result in markdown. Only give the contents of the image. No commentary or explanation. If the image contains a chart or table. Use the markdown syntax to render a table. Do not say things like \"An image containing...\" or \"the image shows\". Try to give as much detail as possible. Do not output any markdown backticks. Use newlines to separate content semantically or to create tables or to separate headings. Only transcribe the contents of the image, not its layout or organization unless it is compatible with markdown. ";

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

fn crop_image(img: DynamicImage, box_: &PdlaBox) -> DynamicImage {
    let pdf_page_width = box_.page_width;
    let pdf_page_height = box_.page_height;
    let scale_x = img.width() as f32 / pdf_page_width;
    let scale_y = img.height() as f32 / pdf_page_height;

    let (x, y, w, h) = (
        (box_.left * scale_x).round() as u32,
        (box_.top * scale_y).round() as u32,
        (box_.width * scale_x).round() as u32,
        (box_.height * scale_y).round() as u32,
    );

    let x = x.min(img.width());
    let y = y.min(img.height());
    let w = w.min(img.width() - x);
    let h = h.min(img.height() - y);

    let cropped = img.crop_imm(x, y, w, h);
    cropped
}

async fn get_chunk_from_image(
    img: DynamicImage,
    task_id: String,
    client: Client,
    box_: &PdlaBox,
) -> Result<ChunkClickhouse, ServiceError> {
    // Convert img to dataurl format

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

    let model = get_env!("LLM_MODEL", "LLM_MODEL must be defined");

    let params = ChatCompletionParametersBuilder::default()
        .model(model)
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
        content,
        metadata: chunk_metadata_string(&box_),
        created_at: OffsetDateTime::now_utc(),
    })
}

#[derive(Debug, Deserialize, Serialize)]
enum PdlaTokenType {
    Formula,
    Footnote,
    #[serde(rename = "List item")]
    ListItem,
    Table,
    Picture,
    Title,
    Text,
    PageHeader,
    #[serde(rename = "Section header")]
    SectionHeader,
    Caption,
    #[serde(rename = "Section footer")]
    PageFooter,
}

#[derive(Debug, Deserialize)]
struct PdlaBox {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub page_number: usize,
    pub page_width: f32,
    pub page_height: f32,
    #[serde(deserialize_with = "empty_string_is_none")]
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub token_type: PdlaTokenType,
}

async fn get_chunk_bounds(pdf_data: Vec<u8>) -> Result<Vec<PdlaBox>, ServiceError> {
    let client = reqwest::Client::new();
    let base_url = get_env!("PDLA_SERVER_ORIGIN", "PDLA Server Origin Should be set");

    let form = Form::new().part(
        "file",
        Part::bytes(pdf_data)
            .file_name("input.pdf")
            .mime_str("application/pdf")
            .unwrap(),
    );

    let response = client
        .post(base_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Failed to send request: {:?}", e))
        })?;

    if !response.status().is_success() {
        return Err(ServiceError::InternalServerError(
            "Failed to send request".to_string(),
        ));
    }

    let response = response.text().await.map_err(|e| {
        ServiceError::InternalServerError(format!("Failed to get response: {:?}", e))
    })?;

    let boxes = serde_json::from_str::<Vec<PdlaBox>>(response.as_ref()).map_err(|e| {
        ServiceError::InternalServerError(format!("Failed to parse response: {:?}", e))
    })?;

    Ok(boxes)
}

fn chunk_metadata_string(bounding_box: &PdlaBox) -> String {
    serde_json::json!({
        "page": bounding_box.page_number,
        "page_width": bounding_box.page_width,
        "page_height": bounding_box.page_height,
        "top": bounding_box.top,
        "left": bounding_box.left,
        "width": bounding_box.width,
        "height": bounding_box.height,
        "type": bounding_box.token_type,
    })
    .to_string()
}

pub async fn chunk_pdf(
    data: Vec<u8>,
    task_id: String,
) -> Result<Vec<ChunkClickhouse>, ServiceError> {
    let pdf = PDF::from_bytes(data.clone())
        .map_err(|_| ServiceError::BadRequest("Failed to open PDF file".to_string()))?;

    let bounds = get_chunk_bounds(data).await?;

    let pages = pdf
        .render(pdf2image::Pages::All, None)
        .map_err(|_| ServiceError::BadRequest("Failed to render PDF file".to_string()))?;

    let client = get_default_openai_client();

    let mut results: Vec<ChunkClickhouse> = vec![];

    for (_, box_) in bounds.iter().enumerate() {
        let task_id = task_id.clone();
        let chunk = match box_.token_type {
            PdlaTokenType::Text => {
                if box_.text.is_none() {
                    // Treat as picture
                    let crop = crop_image(pages[box_.page_number as usize - 1].clone(), box_);
                    let result = get_chunk_from_image(crop, task_id, client.clone(), box_).await;
                    result?
                } else {
                    ChunkClickhouse {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: box_.text.clone().unwrap_or("".to_string()),
                        task_id,
                        created_at: OffsetDateTime::now_utc(),
                        metadata: chunk_metadata_string(box_),
                    }
                }
            }

            PdlaTokenType::Picture | PdlaTokenType::Table => {
                let crop = crop_image(pages[box_.page_number as usize - 1].clone(), box_);
                let result = get_chunk_from_image(crop, task_id, client.clone(), box_).await;
                result?
            }

            PdlaTokenType::Caption
            | PdlaTokenType::PageFooter
            | PdlaTokenType::SectionHeader
            | PdlaTokenType::Footnote
            | PdlaTokenType::Title
            | PdlaTokenType::PageHeader
            | PdlaTokenType::Formula
            | PdlaTokenType::ListItem => {
                if box_.text.is_none() {
                    continue;
                }
                ChunkClickhouse {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: box_.text.clone().unwrap_or("".to_string()),
                    task_id,
                    created_at: OffsetDateTime::now_utc(),
                    metadata: chunk_metadata_string(box_),
                }
            }
        };

        results.push(chunk);
    }

    Ok(results)
}
