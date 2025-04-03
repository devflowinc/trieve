use crate::errors::ServiceError;
use derive_more::Display;
use reqwest;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn default_target_length() -> u32 {
    512
}

pub fn default_ignore_headers_and_footers() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Display, PartialEq, Eq)]
pub enum EmbedSource {
    HTML,
    Markdown,
    LLM,
    Content,
}

fn default_embed_sources() -> Vec<EmbedSource> {
    vec![EmbedSource::Markdown]
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
/// Controls the processing and generation for the segment.
/// - `crop_image` controls whether to crop the file's images to the segment's bounding box.
///   The cropped image will be stored in the segment's `image` field. Use `All` to always crop,
///   or `Auto` to only crop when needed for post-processing.
/// - `html` is the HTML output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `llm` is the LLM-generated output for the segment, this uses off-the-shelf models to generate a custom output for the segment
/// - `markdown` is the Markdown output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `embed_sources` defines which content sources will be included in the chunk's embed field and counted towards the chunk length.
///   The array's order determines the sequence in which content appears in the embed field (e.g., [Markdown, LLM] means Markdown content
///   is followed by LLM content). This directly affects what content is available for embedding and retrieval.
pub struct AutoGenerationConfig {
    #[serde(default = "default_cropping_strategy")]
    #[schema(value_type = CroppingStrategy, default = "Auto")]
    pub crop_image: CroppingStrategy,
    #[serde(default = "default_auto_generation_strategy")]
    #[schema(default = "Auto")]
    pub html: GenerationStrategy,
    /// Prompt for the LLM mode
    pub llm: Option<String>,
    #[serde(default = "default_auto_generation_strategy")]
    #[schema(default = "Auto")]
    pub markdown: GenerationStrategy,
    #[serde(default = "default_embed_sources")]
    #[schema(value_type = Vec<EmbedSource>, default = "[Markdown]")]
    pub embed_sources: Vec<EmbedSource>,
}

fn default_cropping_strategy() -> CroppingStrategy {
    CroppingStrategy::Auto
}

fn default_picture_cropping_strategy() -> PictureCroppingStrategy {
    PictureCroppingStrategy::All
}

fn default_auto_generation_strategy() -> GenerationStrategy {
    GenerationStrategy::Auto
}

fn default_llm_generation_strategy() -> GenerationStrategy {
    GenerationStrategy::LLM
}

impl Default for AutoGenerationConfig {
    fn default() -> Self {
        Self {
            html: GenerationStrategy::Auto,
            llm: None,
            markdown: GenerationStrategy::Auto,
            crop_image: default_cropping_strategy(),
            embed_sources: default_embed_sources(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Display, Eq, PartialEq, Serialize, ToSchema)]
/// Controls the cropping strategy for an item (e.g. segment, chunk, etc.)
/// - `All` crops all images in the item
/// - `Auto` crops images only if required for post-processing
pub enum CroppingStrategy {
    All,
    #[default]
    Auto,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
/// Controls the processing and generation for the segment.
/// - `crop_image` controls whether to crop the file's images to the segment's bounding box.
///   The cropped image will be stored in the segment's `image` field. Use `All` to always crop,
///   or `Auto` to only crop when needed for post-processing.
/// - `html` is the HTML output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `llm` is the LLM-generated output for the segment, this uses off-the-shelf models to generate a custom output for the segment
/// - `markdown` is the Markdown output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `embed_sources` defines which content sources will be included in the chunk's embed field and counted towards the chunk length.
///   The array's order determines the sequence in which content appears in the embed field (e.g., [Markdown, LLM] means Markdown content
///   is followed by LLM content). This directly affects what content is available for embedding and retrieval.
pub struct LlmGenerationConfig {
    #[serde(default = "default_cropping_strategy")]
    #[schema(value_type = CroppingStrategy, default = "Auto")]
    pub crop_image: CroppingStrategy,
    #[serde(default = "default_llm_generation_strategy")]
    #[schema(default = "LLM")]
    pub html: GenerationStrategy,
    /// Prompt for the LLM model
    pub llm: Option<String>,
    #[serde(default = "default_llm_generation_strategy")]
    #[schema(default = "LLM")]
    pub markdown: GenerationStrategy,
    #[serde(default = "default_embed_sources")]
    #[schema(value_type = Vec<EmbedSource>, default = "[Markdown]")]
    pub embed_sources: Vec<EmbedSource>,
}

impl Default for LlmGenerationConfig {
    fn default() -> Self {
        Self {
            html: GenerationStrategy::LLM,
            llm: None,
            markdown: GenerationStrategy::LLM,
            crop_image: default_cropping_strategy(),
            embed_sources: default_embed_sources(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Display, Eq, PartialEq, Serialize, ToSchema)]
/// Controls the cropping strategy for an item (e.g. segment, chunk, etc.)
/// - `All` crops all images in the item
/// - `Auto` crops images only if required for post-processing
pub enum PictureCroppingStrategy {
    #[default]
    All,
    Auto,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
/// Controls the processing and generation for the segment.
/// - `crop_image` controls whether to crop the file's images to the segment's bounding box.
///   The cropped image will be stored in the segment's `image` field. Use `All` to always crop,
///   or `Auto` to only crop when needed for post-processing.
/// - `html` is the HTML output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `llm` is the LLM-generated output for the segment, this uses off-the-shelf models to generate a custom output for the segment
/// - `markdown` is the Markdown output for the segment, generated either through huerstics (`Auto`) or using Chunkr fine-tuned models (`LLM`)
/// - `embed_sources` defines which content sources will be included in the chunk's embed field and counted towards the chunk length.
///   The array's order determines the sequence in which content appears in the embed field (e.g., [Markdown, LLM] means Markdown content
///   is followed by LLM content). This directly affects what content is available for embedding and retrieval.
pub struct PictureGenerationConfig {
    #[serde(default = "default_picture_cropping_strategy")]
    #[schema(value_type = PictureCroppingStrategy, default = "All")]
    pub crop_image: PictureCroppingStrategy,
    #[serde(default = "default_llm_generation_strategy")]
    #[schema(default = "LLM")]
    pub html: GenerationStrategy,
    /// Prompt for the LLM model
    pub llm: Option<String>,
    #[serde(default = "default_llm_generation_strategy")]
    #[schema(default = "LLM")]
    pub markdown: GenerationStrategy,
    #[serde(default = "default_embed_sources")]
    #[schema(value_type = Vec<EmbedSource>, default = "[Markdown]")]
    pub embed_sources: Vec<EmbedSource>,
}

impl Default for PictureGenerationConfig {
    fn default() -> Self {
        Self {
            html: GenerationStrategy::Auto,
            llm: None,
            markdown: GenerationStrategy::Auto,
            crop_image: default_picture_cropping_strategy(),
            embed_sources: default_embed_sources(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Display, PartialEq, Eq)]
pub enum GenerationStrategy {
    LLM,
    Auto,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
/// Controls the post-processing of each segment type.
///
/// Allows you to generate HTML and Markdown from chunkr models for each segment type.
/// By default, the HTML and Markdown are generated manually using the segmentation information except for `Table`, `Formula` and `Picture`.
/// You can optionally configure custom LLM prompts and models to generate an additional `llm` field with LLM-processed content for each segment type.
///
/// The configuration of which content sources (HTML, Markdown, LLM, Content) of the segment
/// should be included in the chunk's `embed` field and counted towards the chunk length can be configured through the `embed_sources` setting.
pub struct SegmentProcessing {
    #[serde(rename = "Title", alias = "title")]
    pub title: Option<AutoGenerationConfig>,
    #[serde(rename = "SectionHeader", alias = "section_header")]
    pub section_header: Option<AutoGenerationConfig>,
    #[serde(rename = "Text", alias = "text")]
    pub text: Option<AutoGenerationConfig>,
    #[serde(rename = "ListItem", alias = "list_item")]
    pub list_item: Option<AutoGenerationConfig>,
    #[serde(rename = "Table", alias = "table")]
    pub table: Option<LlmGenerationConfig>,
    #[serde(rename = "Picture", alias = "picture")]
    pub picture: Option<PictureGenerationConfig>,
    #[serde(rename = "Caption", alias = "caption")]
    pub caption: Option<AutoGenerationConfig>,
    #[serde(rename = "Formula", alias = "formula")]
    pub formula: Option<LlmGenerationConfig>,
    #[serde(rename = "Footnote", alias = "footnote")]
    pub footnote: Option<AutoGenerationConfig>,
    #[serde(rename = "PageHeader", alias = "page_header")]
    pub page_header: Option<AutoGenerationConfig>,
    #[serde(rename = "PageFooter", alias = "page_footer")]
    pub page_footer: Option<AutoGenerationConfig>,
    #[serde(rename = "Page", alias = "page")]
    pub page: Option<LlmGenerationConfig>,
}

impl Default for SegmentProcessing {
    fn default() -> Self {
        Self {
            title: Some(AutoGenerationConfig::default()),
            section_header: Some(AutoGenerationConfig::default()),
            text: Some(AutoGenerationConfig::default()),
            list_item: Some(AutoGenerationConfig::default()),
            table: Some(LlmGenerationConfig::default()),
            picture: Some(PictureGenerationConfig::default()),
            caption: Some(AutoGenerationConfig::default()),
            formula: Some(LlmGenerationConfig::default()),
            footnote: Some(AutoGenerationConfig::default()),
            page_header: Some(AutoGenerationConfig::default()),
            page_footer: Some(AutoGenerationConfig::default()),
            page: Some(LlmGenerationConfig::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Display)]
/// Specifies which tokenizer to use for the chunking process.
///
/// This type supports two ways of specifying a tokenizer:
/// 1. Using a predefined tokenizer from the `Tokenizer` enum
/// 2. Using any Hugging Face tokenizer by providing its model ID as a string
///    (e.g. "facebook/bart-large", "Qwen/Qwen-tokenizer", etc.)
///
/// When using a string, any valid Hugging Face tokenizer ID can be specified,
/// which will be loaded using the Hugging Face tokenizers library.
pub enum TokenizerType {
    /// Use one of the predefined tokenizer types
    Enum(Tokenizer),
    /// Use any Hugging Face tokenizer by specifying its model ID
    /// Examples: "Qwen/Qwen-tokenizer", "facebook/bart-large"
    String(String),
}

// Add Default implementation for TokenizerType
impl Default for TokenizerType {
    fn default() -> Self {
        TokenizerType::Enum(Tokenizer::default())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Default, Display)]
/// Common tokenizers used for text processing.
///
/// These values represent standard tokenization approaches and popular pre-trained
/// tokenizers from the Hugging Face ecosystem.
pub enum Tokenizer {
    /// Split text by word boundaries
    #[default]
    Word,
    /// For OpenAI models (e.g. GPT-3.5, GPT-4, text-embedding-ada-002)
    Cl100kBase,
    /// For RoBERTa-based multilingual models
    XlmRobertaBase,
    /// BERT base uncased tokenizer
    BertBaseUncased,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
/// Controls the setting for the chunking and post-processing of each chunk.
pub struct ChunkProcessing {
    #[serde(default = "default_ignore_headers_and_footers")]
    #[schema(value_type = bool, default = true)]
    /// Whether to ignore headers and footers in the chunking process.
    /// This is recommended as headers and footers break reading order across pages.
    pub ignore_headers_and_footers: bool,
    #[serde(default = "default_target_length")]
    #[schema(value_type = u32, default = 512)]
    /// The target number of words in each chunk. If 0, each chunk will contain a single segment.
    pub target_length: u32,
    /// The tokenizer to use for the chunking process.
    #[schema( value_type = TokenizerType, default = "Word")]
    #[serde(default)]
    pub tokenizer: TokenizerType,
}

/// Default implementation for ChunkProcessing
impl Default for ChunkProcessing {
    fn default() -> Self {
        Self {
            ignore_headers_and_footers: default_ignore_headers_and_footers(),
            target_length: default_target_length(),
            tokenizer: TokenizerType::Enum(Tokenizer::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema, Display)]
/// Controls the Optical Character Recognition (OCR) strategy.
/// - `All`: Processes all pages with OCR. (Latency penalty: ~0.5 seconds per page)
/// - `Auto`: Selectively applies OCR only to pages with missing or low-quality text. When text layer is present the bounding boxes from the text layer are used.
#[derive(Default)]
pub enum OcrStrategy {
    #[default]
    All,
    #[serde(alias = "Off")]
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, Eq, PartialEq, ToSchema, Default)]
/// Controls the segmentation strategy:
/// - `LayoutAnalysis`: Analyzes pages for layout elements (e.g., `Table`, `Picture`, `Formula`, etc.) using bounding boxes. Provides fine-grained segmentation and better chunking. (Latency penalty: ~TBD seconds per page).
/// - `Page`: Treats each page as a single segment. Faster processing, but without layout element detection and only simple chunking.
pub enum SegmentationStrategy {
    #[default]
    LayoutAnalysis,
    Page,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema, Display, Default)]
pub enum PipelineType {
    Azure,
    #[default]
    Chunkr,
}

#[derive(Debug, Deserialize)]
pub struct JobConfig {
    #[serde(default = "default_task_timeout")]
    pub task_timeout: u32,
    pub expiration_time: Option<i32>,
    #[serde(default = "default_job_interval")]
    pub job_interval: u64,
}

fn default_job_interval() -> u64 {
    600
}

fn default_task_timeout() -> u32 {
    1800
}
#[derive(Debug, Serialize, Clone, Deserialize, ToSchema, IntoParams)]
pub struct CreateForm {
    pub chunk_processing: Option<ChunkProcessing>,
    /// The number of seconds until task is deleted.
    /// Expried tasks can **not** be updated, polled or accessed via web interface.
    pub expires_in: Option<i32>,
    /// The file to be uploaded. Can be a URL or a base64 encoded file.
    pub file: String,
    /// The name of the file to be uploaded. If not set a name will be generated.
    pub file_name: Option<String>,
    /// Whether to use high-resolution images for cropping and post-processing. (Latency penalty: ~7 seconds per page)
    #[schema(default = false)]
    pub high_resolution: Option<bool>,
    #[schema(default = "All")]
    pub ocr_strategy: Option<OcrStrategy>,
    #[schema(default = "Azure")]
    /// Choose the provider whose models will be used for segmentation and OCR.
    /// The output will be unified to the Chunkr `output` format.
    pub pipeline: Option<PipelineType>,
    pub segment_processing: Option<SegmentProcessing>,
    #[schema(default = "LayoutAnalysis")]
    pub segmentation_strategy: Option<SegmentationStrategy>,
}

impl CreateForm {
    fn get_chunk_processing(&self) -> ChunkProcessing {
        self.chunk_processing.clone().unwrap_or_default()
    }

    fn get_high_resolution(&self) -> bool {
        self.high_resolution.unwrap_or(false)
    }

    fn get_ocr_strategy(&self) -> OcrStrategy {
        self.ocr_strategy.clone().unwrap_or_default()
    }

    fn get_segment_processing(&self) -> SegmentProcessing {
        let user_config = self.segment_processing.clone().unwrap_or_default();
        SegmentProcessing {
            title: user_config
                .title
                .or_else(|| SegmentProcessing::default().title),
            section_header: user_config
                .section_header
                .or_else(|| SegmentProcessing::default().section_header),
            text: user_config
                .text
                .or_else(|| SegmentProcessing::default().text),
            list_item: user_config
                .list_item
                .or_else(|| SegmentProcessing::default().list_item),
            table: user_config
                .table
                .or_else(|| SegmentProcessing::default().table),
            picture: user_config
                .picture
                .or_else(|| SegmentProcessing::default().picture),
            caption: user_config
                .caption
                .or_else(|| SegmentProcessing::default().caption),
            formula: user_config
                .formula
                .or_else(|| SegmentProcessing::default().formula),
            footnote: user_config
                .footnote
                .or_else(|| SegmentProcessing::default().footnote),
            page_header: user_config
                .page_header
                .or_else(|| SegmentProcessing::default().page_header),
            page_footer: user_config
                .page_footer
                .or_else(|| SegmentProcessing::default().page_footer),
            page: user_config
                .page
                .or_else(|| SegmentProcessing::default().page),
        }
    }

    fn get_segmentation_strategy(&self) -> SegmentationStrategy {
        self.segmentation_strategy.clone().unwrap_or_default()
    }

    fn get_pipeline(&self) -> Option<PipelineType> {
        Some(self.pipeline.clone().unwrap_or_default())
    }

    pub fn to_configuration(&self) -> Configuration {
        Configuration {
            chunk_processing: self.get_chunk_processing(),
            expires_in: None,
            high_resolution: self.get_high_resolution(),
            input_file_url: None,
            ocr_strategy: self.get_ocr_strategy(),
            pipeline: self.get_pipeline(),
            segment_processing: self.get_segment_processing(),
            segmentation_strategy: self.get_segmentation_strategy(),
        }
    }
}

fn get_chunkr_credentials(api_key: Option<&str>) -> Result<(String, String), ServiceError> {
    let api_url = std::env::var("CHUNKR_API_URL").unwrap_or("https://api.chunkr.ai".to_string());
    let api_key = match api_key {
        Some(key) => key.to_string(),
        None => std::env::var("CHUNKR_API_KEY").map_err(|_| {
            ServiceError::InternalServerError("CHUNKR_API_KEY should be set".to_string())
        })?,
    };
    Ok((format!("{}/api/v1/task", api_url), api_key))
}

pub async fn create_chunkr_task(
    file_name: &str,
    file_base64: &str,
    api_key: Option<&str>,
    chunkr_create_task_req_payload: Option<CreateForm>,
) -> Result<TaskResponse, ServiceError> {
    let client = reqwest::Client::new();
    let (api_url, api_key) = get_chunkr_credentials(api_key)?;

    let req_payload = chunkr_create_task_req_payload.unwrap_or_else(|| CreateForm {
        file_name: Some(file_name.to_string()),
        file: file_base64.to_string(),
        pipeline: Some(PipelineType::Azure),
        high_resolution: Some(true),
        chunk_processing: Some(ChunkProcessing {
            ignore_headers_and_footers: default_ignore_headers_and_footers(),
            target_length: default_target_length(),
            tokenizer: TokenizerType::Enum(Tokenizer::default()),
        }),
        expires_in: None,
        ocr_strategy: None,
        segment_processing: None,
        segmentation_strategy: None,
    });

    let response = client
        .post(format!("{}/parse", api_url))
        .header("Authorization", api_key)
        .json(&req_payload)
        .send()
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Failed to send create chunkr task: {}", e))
        })?
        .error_for_status()
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Failed to create chunkr task: {}", e))
        })?
        .json::<TaskResponse>()
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!(
                "Failed to parse create chunkr task response: {}",
                e
            ))
        })?;

    Ok(response)
}

pub async fn get_chunkr_task(
    task_id: &str,
    api_key: Option<&str>,
) -> Result<TaskResponse, ServiceError> {
    let client = reqwest::Client::new();
    let (api_url, api_key) = get_chunkr_credentials(api_key)?;
    let response = client
        .get(format!("{}/{}", api_url, task_id))
        .header("Authorization", api_key)
        .send()
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!(
                "Failed to send get chunkr task request: {}",
                e
            ))
        })?
        .error_for_status()
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Failed to get chunkr task: {}", e))
        })?
        .json::<TaskResponse>()
        .await
        .map_err(|e| {
            ServiceError::InternalServerError(format!(
                "Failed to parse get chunkr task response: {}",
                e
            ))
        })?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoundingBox {
    pub height: f32,
    pub left: f32,
    pub top: f32,
    pub width: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
    pub chunk_id: String,
    pub chunk_length: u32,
    pub segments: Vec<Segment>,
    pub embed: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
/// The configuration used for the task.
pub struct Configuration {
    pub chunk_processing: ChunkProcessing,
    #[serde(alias = "expires_at")]
    /// The number of seconds until task is deleted.
    /// Expried tasks can **not** be updated, polled or accessed via web interface.
    pub expires_in: Option<i32>,
    /// Whether to use high-resolution images for cropping and post-processing.
    pub high_resolution: bool,
    /// The presigned URL of the input file.
    pub input_file_url: Option<String>,
    pub ocr_strategy: OcrStrategy,
    pub segment_processing: SegmentProcessing,
    pub segmentation_strategy: SegmentationStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<PipelineType>,
}

impl<'de> Deserialize<'de> for Configuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            chunk_processing: Option<ChunkProcessing>,
            #[serde(alias = "expires_at")]
            expires_in: Option<i32>,
            #[serde(default)]
            high_resolution: bool,
            input_file_url: Option<String>,
            #[serde(default)]
            ocr_strategy: Option<OcrStrategy>,
            #[serde(default)]
            segment_processing: Option<SegmentProcessing>,
            #[serde(default)]
            segmentation_strategy: Option<SegmentationStrategy>,
            target_chunk_length: Option<u32>,
            pipeline: Option<PipelineType>,
        }

        let helper = Helper::deserialize(deserializer)?;

        // If chunk_processing is None but target_chunk_length exists,
        // create a default ChunkProcessing with the specified target length
        let chunk_processing = match (helper.chunk_processing, helper.target_chunk_length) {
            (Some(cp), _) => cp,
            (None, Some(target_length)) => ChunkProcessing {
                target_length,
                ..ChunkProcessing::default()
            },
            (None, None) => ChunkProcessing::default(),
        };

        Ok(Configuration {
            chunk_processing,
            expires_in: helper.expires_in,
            high_resolution: helper.high_resolution,
            input_file_url: helper.input_file_url,
            ocr_strategy: helper.ocr_strategy.unwrap_or(OcrStrategy::default()),
            segment_processing: helper
                .segment_processing
                .unwrap_or(SegmentProcessing::default()),
            segmentation_strategy: helper
                .segmentation_strategy
                .unwrap_or(SegmentationStrategy::default()),
            pipeline: helper.pipeline,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedField {
    pub name: String,
    pub field_type: String,
    #[serde(
        serialize_with = "serialize_value",
        deserialize_with = "deserialize_value"
    )]
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtractedJson {
    pub title: String,
    pub schema_type: String,
    pub extracted_fields: Vec<ExtractedField>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub description: String,
    pub field_type: String,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonSchema {
    pub title: String,
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Vec<Property>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OCRResult {
    pub bbox: BoundingBox,
    pub confidence: Option<f32>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputResponse {
    pub chunks: Vec<Chunk>,
    pub file_name: Option<String>,
    pub page_count: Option<u32>,
    pub pdf_url: Option<String>,
    pub extracted_json: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Property {
    pub name: String,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: Option<String>,
    pub default: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
    pub bbox: BoundingBox,
    pub confidence: Option<f32>,
    pub content: String,
    pub html: String,
    pub image: Option<String>,
    pub llm: Option<String>,
    pub markdown: String,
    pub ocr: Option<Vec<OCRResult>>,
    pub page_height: f32,
    pub page_number: u32,
    pub page_width: f32,
    pub segment_id: String,
    pub segment_type: SegmentType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SegmentType {
    Caption,
    Footnote,
    Formula,
    ListItem,
    Page,
    PageFooter,
    PageHeader,
    Picture,
    SectionHeader,
    Table,
    Text,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum Status {
    #[display("Canceled")]
    Canceled,
    #[display("Failed")]
    Failed,
    #[display("Processing")]
    Processing,
    #[display("Starting")]
    Starting,
    #[display("Completed")] // To match pdf2md output
    Succeeded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResponse {
    pub configuration: Configuration,
    /// The date and time when the task was created and queued.
    pub created_at: String,
    pub expires_at: Option<String>,
    /// The date and time when the task was finished.
    pub finished_at: Option<String>,
    /// A message describing the task's status or any errors that occurred.
    pub message: String,
    pub output: Option<OutputResponse>,
    pub started_at: Option<String>,
    pub status: Status,
    /// The unique identifier for the task.
    pub task_id: String,
    /// The presigned URL of the task.
    pub task_url: Option<String>,
}

fn serialize_value<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.serialize(serializer)
}

fn deserialize_value<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_json::Value::deserialize(deserializer)
}
