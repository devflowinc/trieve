use super::{
    auth_handler::{AdminOnly, LoggedUser},
    group_handler::DeleteGroupData,
};
use crate::{
    data::models::{
        ChunkReqPayloadMappings, CsvJsonlWorkerMessage, DatasetAndOrgWithSubAndPlan,
        DatasetConfiguration, File, FileAndGroupId, FileWithChunkGroups, FileWorkerMessage, Pool,
        RedisPool,
    },
    errors::ServiceError,
    operators::{
        crawl_operator::{process_crawl_doc, Document},
        file_operator::{
            create_file_query, delete_file_query, get_aws_bucket, get_csvjsonl_aws_bucket,
            get_dataset_files_and_group_ids_query, get_file_query, get_files_query,
        },
        organization_operator::{get_file_size_sum_org, hash_function},
    },
};
use actix_web::{web, HttpResponse};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use broccoli_queue::queue::BroccoliQueue;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn validate_file_name(s: String) -> Result<String, actix_web::Error> {
    let split_s = s.split('/').last();

    if let Some(name) = split_s {
        if name.contains("..") {
            return Err(ServiceError::BadRequest("Invalid file name".to_string()).into());
        }

        return Ok(name.to_string());
    }

    Err(ServiceError::BadRequest("Invalid file name".to_string()).into())
}

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

/// Will use [chunkr.ai](https://chunkr.ai) to process the file when this object is defined. See [docs.chunkr.ai/api-references/task/create-task](https://docs.chunkr.ai/api-references/task/create-task) for detailed information about what each field on this request payload does.
#[derive(Debug, Serialize, Clone, Deserialize, ToSchema, IntoParams)]
pub struct CreateFormWithoutFile {
    pub chunk_processing: Option<ChunkProcessing>,
    /// The number of seconds until task is deleted.
    /// Expried tasks can **not** be updated, polled or accessed via web interface.
    pub expires_in: Option<i32>,
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "file_name": "example.pdf",
    "base64_file": "<base64_encoded_file>",
    "tag_set": ["tag1", "tag2"],
    "description": "This is an example file",
    "link": "https://example.com",
    "time_stamp": "2021-01-01 00:00:00.000Z",
    "metadata": {
        "key1": "value1",
        "key2": "value2"
    },
    "create_chunks": true,
    "split_delimiters": [",",".","\n"],
    "target_splits_per_chunk": 20,
    "use_pdf2md_ocr": false
}))]
pub struct UploadFileReqPayload {
    /// Base64 encoded file.
    pub base64_file: String,
    /// Name of the file being uploaded, including the extension.
    pub file_name: String,
    /// Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Tags are used to filter chunks when searching. HNSW indices are created for each tag such that there is no performance loss when filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk.
    pub description: Option<String>,
    /// Link to the file. This can also be any string. This can be used to filter when searching for the file's resulting chunks. The link value will not affect embedding creation.
    pub link: Option<String>,
    /// Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file's chunks.
    pub time_stamp: Option<String>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file's chunks.
    pub metadata: Option<serde_json::Value>,
    /// Create chunks is a boolean which determines whether or not to create chunks from the file. If false, you can manually chunk the file and send the chunks to the create_chunk endpoint with the file_id to associate chunks with the file. Meant mostly for advanced users.
    pub create_chunks: Option<bool>,
    /// Rebalance chunks is an optional field which allows you to specify whether or not to rebalance the chunks created from the file. If not specified, the default true is used. If true, Trieve will evenly distribute remainder splits across chunks such that 66 splits with a `target_splits_per_chunk` of 20 will result in 3 chunks with 22 splits each.
    pub rebalance_chunks: Option<bool>,
    /// Split delimiters is an optional field which allows you to specify the delimiters to use when splitting the file before chunking the text. If not specified, the default [.!?\n] are used to split into sentences. However, you may want to use spaces or other delimiters.
    pub split_delimiters: Option<Vec<String>>,
    /// Target splits per chunk. This is an optional field which allows you to specify the number of splits you want per chunk. If not specified, the default 20 is used. However, you may want to use a different number.
    pub target_splits_per_chunk: Option<usize>,
    /// Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of `group_tracking_id|<index of chunk>`
    pub group_tracking_id: Option<String>,
    /// The request payload to use for the Chunkr API create task endpoint.
    pub chunkr_create_task_req_payload: Option<CreateFormWithoutFile>,
    /// Parameter to use pdf2md_ocr. If true, the file will be converted to markdown using gpt-4o. Default is false.
    pub pdf2md_options: Option<Pdf2MdOptions>,
    /// Split average will automatically split your file into multiple chunks and average all of the resulting vectors into a single output chunk. Default is false. Explicitly enabling this will cause each file to only produce a single chunk.
    pub split_avg: Option<bool>,
}

/// We plan to deprecate pdf2md in favor of chunkr.ai. This is a legacy option for using a vision LLM to convert a given file into markdown and then ingest it.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pdf2MdOptions {
    /// Parameter to use pdf2md_ocr. If true, the file will be converted to markdown using gpt-4o. Default is false.
    pub use_pdf2md_ocr: bool,
    /// Prompt to use for the gpt-4o model. Default is None.
    pub system_prompt: Option<String>,
    /// Split headings is an optional field which allows you to specify whether or not to split headings into separate chunks. Default is false.
    pub split_headings: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileResponseBody {
    /// File object information. Id, name, tag_set, etc.
    pub file_metadata: File,
}

/// Upload File
///
/// Upload a file to S3 bucket attached to your dataset. You can select between a naive chunking strategy where the text is extracted with Apache Tika and split into segments with a target number of segments per chunk OR you can use a vision LLM to convert the file to markdown and create chunks per page. You must specifically use a base64url encoding. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file",
    context_path = "/api",
    tag = "File",
    request_body(content = UploadFileReqPayload, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 200, description = "Confirmation that the file is uploading", body = UploadFileResponseBody),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn upload_file_handler(
    data: web::Json<UploadFileReqPayload>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    broccoli_queue: web::Data<BroccoliQueue>,
) -> Result<HttpResponse, actix_web::Error> {
    // Disallow split_avg with pdf2md
    if let Some(Pdf2MdOptions { use_pdf2md_ocr, .. }) = data.pdf2md_options {
        if use_pdf2md_ocr && data.split_avg.unwrap_or(false) {
            return Err(ServiceError::BadRequest(
                "split_avg is not supported with pdf2md".to_string(),
            )
            .into());
        }
    }

    let file_size_sum_pool = pool.clone();
    let file_size_sum = get_file_size_sum_org(
        dataset_org_plan_sub.organization.organization.id,
        file_size_sum_pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    if file_size_sum
        >= dataset_org_plan_sub
            .clone()
            .organization
            .plan
            .unwrap_or_default()
            .file_storage()
    {
        return Err(ServiceError::BadRequest("File size limit reached".to_string()).into());
    }

    let upload_file_data = data.into_inner();

    let mut cleaned_base64 = upload_file_data
        .base64_file
        .replace('+', "-")
        .replace('/', "_");
    if cleaned_base64.ends_with('=') {
        cleaned_base64.pop();
    }
    let base64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    let decoded_file_data = base64_engine.decode(upload_file_data.base64_file.clone());

    let decoded_file_data = match decoded_file_data {
        Ok(data) => data,
        Err(e) => base64::prelude::BASE64_STANDARD
            .decode(upload_file_data.base64_file.as_bytes())
            .map_err(|_e| {
                log::error!("Could not decode base64 file: {:?}", e);
                ServiceError::BadRequest("Could not decode base64 file".to_string())
            })?,
    };

    let file_id = uuid::Uuid::new_v4();

    let bucket = get_aws_bucket()?;

    if upload_file_data.file_name.clone().ends_with(".pdf") {
        bucket
            .put_object_with_content_type(
                file_id.to_string(),
                decoded_file_data.as_slice(),
                "application/pdf",
            )
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;
    } else {
        bucket
            .put_object(file_id.to_string(), decoded_file_data.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;
    }

    let file_size_mb = (decoded_file_data.len() as f64 / 1024.0).ceil() as i64;

    create_file_query(
        file_id,
        file_size_mb,
        upload_file_data.clone(),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    let message = FileWorkerMessage {
        file_id,
        dataset_id: dataset_org_plan_sub.dataset.id,
        organization_id: dataset_org_plan_sub.organization.organization.id,
        upload_file_data: upload_file_data.clone(),
        attempt_number: 0,
    };

    broccoli_queue
        .publish(
            "file_ingestion",
            Some(dataset_org_plan_sub.dataset.id.to_string()),
            &message,
            None,
        )
        .await
        .map_err(|e| {
            log::error!("Could not publish message: {:?}", e);
            ServiceError::BadRequest("Could not publish message".to_string())
        })?;

    let result = UploadFileResponseBody {
        file_metadata: File::from_details(
            Some(file_id),
            &upload_file_data.file_name,
            decoded_file_data.len().try_into().unwrap_or_default(),
            upload_file_data
                .tag_set
                .map(|t| t.into_iter().map(Some).collect()),
            upload_file_data.metadata.clone(),
            upload_file_data.link.clone(),
            upload_file_data.time_stamp.clone(),
            dataset_org_plan_sub.dataset.id,
        ),
    };

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadHtmlPageReqPayload {
    pub data: Document,
    pub metadata: serde_json::Value,
    pub scrape_id: uuid::Uuid,
}

/// Upload HTML Page
///
/// Chunk HTML by headings and queue for indexing into the specified dataset.
#[utoipa::path(
    post,
    path = "/file/html_page",
    context_path = "/api",
    tag = "File",
    request_body(content = UploadHtmlPageReqPayload, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that html is being processed"),
        (status = 400, description = "Service error relating to processing the file", body = ErrorResponseBody),
    ),
)]
pub async fn upload_html_page(
    data: web::Json<UploadHtmlPageReqPayload>,
    broccoli_queue: web::Data<BroccoliQueue>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let req_payload = data.into_inner();

    let dataset_id = req_payload
        .metadata
        .as_object()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must be a JSON object".to_string())
        })?
        .get("dataset_id")
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field is required to specify dataset_id".to_string())
        })?
        .as_str()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?
        .parse::<uuid::Uuid>()
        .map_err(|_| {
            log::error!("metadata field must have a valid dataset_id");
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?;

    let webhook_secret = req_payload
        .metadata
        .as_object()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must be a JSON object".to_string())
        })?
        .get("webhook_secret")
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field is required to specify dataset_id".to_string())
        })?
        .as_str()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?
        .parse::<String>()
        .map_err(|_| {
            log::error!("metadata field must have a valid dataset_id");
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?;

    let cur_secret = hash_function(
        std::env::var("STRIPE_WEBHOOK_SECRET")
            .unwrap_or("firecrawl".to_string())
            .as_str(),
    );

    if webhook_secret != cur_secret {
        log::error!("Webhook secret does not match.");
        return Err(ServiceError::BadRequest("Webhook secret does not match.".to_string()).into());
    }

    process_crawl_doc(
        dataset_id,
        req_payload.scrape_id,
        req_payload.data,
        broccoli_queue,
        pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
pub struct FileSignedUrlOptions {
    /// The content type of the file
    pub content_type: Option<String>,
    /// The time to live of the signed url in seconds. Defaults to 86400 seconds (1 day).
    pub ttl: Option<u32>,
}

/// Get File with Signed URL
///
/// Get all of the information for a file along with a signed s3 url corresponding to the file_id requested such that you can download the file.
#[utoipa::path(
    get,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "The file's information and s3_url where the original file can be downloaded", body = FileDTO),
        (status = 400, description = "Service error relating to finding the file", body = ErrorResponseBody),
        (status = 404, description = "File not found", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("file_id" = uuid::Uuid, description = "The id of the file to fetch"),
        ("ttl" = Option<u32>, Query, description = "The time to live of the signed url in seconds"),
        ("content_type" = Option<String>, Query, description = "Optional field to override the presigned url's Content-Type header"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    options: web::Query<FileSignedUrlOptions>,
) -> Result<HttpResponse, actix_web::Error> {
    let ttl = options.ttl.unwrap_or(86400);

    let file = get_file_query(
        file_id.into_inner(),
        ttl,
        dataset_org_plan_sub.dataset.id,
        options.into_inner().content_type,
        pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(file))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "file_name": "example.pdf",
    "tag_set": ["tag1", "tag2"],
    "description": "This is an example file",
    "link": "https://example.com",
    "time_stamp": "2021-01-01 00:00:00.000Z",
    "metadata": {
        "key1": "value1",
        "key2": "value2"
    },
}))]
pub struct CreatePresignedUrlForCsvJsonlReqPayload {
    /// Name of the file being uploaded, including the extension. Will be used to determine CSV or JSONL for processing.
    pub file_name: String,
    /// Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Each tag will be joined with what's creatd per row of the CSV or JSONL file.
    pub tag_set: Option<Vec<String>>,
    /// Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk.
    pub description: Option<String>,
    /// Link to the file. This can also be any string. This can be used to filter when searching for the file's resulting chunks. The link value will not affect embedding creation.
    pub link: Option<String>,
    /// Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file's chunks.
    pub time_stamp: Option<String>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file's chunks.
    pub metadata: Option<serde_json::Value>,
    /// Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of `group_tracking_id|<index of chunk>`
    pub group_tracking_id: Option<String>,
    /// Specify all of the mappings between columns or fields in a CSV or JSONL file and keys in the ChunkReqPayload. Array fields like tag_set and image_urls can have multiple mappings. Boost phrase can also have multiple mappings which get concatenated. Other fields can only have one mapping and only the last mapping will be used.
    pub mappings: Option<ChunkReqPayloadMappings>,
    /// Upsert by tracking_id. If true, chunks will be upserted by tracking_id. If false, chunks with the same tracking_id as another already existing chunk will be ignored. Defaults to true.
    pub upsert_by_tracking_id: Option<bool>,
    /// Amount to multiplicatevly increase the frequency of the tokens in the boost phrase for each row's chunk by. Applies to fulltext (SPLADE) and keyword (BM25) search.
    pub fulltext_boost_factor: Option<f64>,
    /// Arbitrary float (positive or negative) specifying the multiplicate factor to apply before summing the phrase vector with the chunk_html embedding vector. Applies to semantic (embedding model) search.
    pub semantic_boost_factor: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreatePresignedUrlForCsvJsonResponseBody {
    /// File object information. Id, name, tag_set, etc.
    pub file_metadata: File,
    /// Signed URL to upload the file to.
    pub presigned_put_url: String,
}

/// Create Presigned CSV/JSONL S3 PUT URL
///
/// This route is useful for uploading very large CSV or JSONL files. Once you have completed the upload, chunks will be automatically created from the file for each line in the CSV or JSONL file. The chunks will be indexed and searchable. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file/csv_or_jsonl",
    context_path = "/api",
    tag = "File",
    request_body(content = CreatePresignedUrlForCsvJsonlReqPayload, description = "JSON request payload to upload a CSV or JSONL file", content_type = "application/json"),
    responses(
        (status = 200, description = "File object information and signed put URL", body = CreatePresignedUrlForCsvJsonResponseBody),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_presigned_url_for_csv_jsonl(
    data: web::Json<CreatePresignedUrlForCsvJsonlReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let create_presigned_put_url_data = data.into_inner();

    let file_id = uuid::Uuid::new_v4();

    let bucket = get_csvjsonl_aws_bucket()?;
    let presigned_put_url = bucket
        .presign_put(file_id.to_string(), 86400, None, None)
        .await
        .map_err(|e| {
            log::error!("Could not get presigned put url: {:?}", e);
            ServiceError::BadRequest("Could not get presigned put url".to_string())
        })?;

    let message = CsvJsonlWorkerMessage {
        file_id,
        dataset_id: dataset_org_plan_sub.dataset.id,
        create_presigned_put_url_data: create_presigned_put_url_data.clone(),
        created_at: chrono::Utc::now().naive_utc(),
        attempt_number: 0,
    };

    let serialized_message = serde_json::to_string(&message).map_err(|e| {
        log::error!("Could not serialize message: {:?}", e);
        ServiceError::BadRequest("Could not serialize message".to_string())
    })?;

    redis::cmd("lpush")
        .arg("csv_jsonl_ingestion")
        .arg(&serialized_message)
        .query_async::<_, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let result = CreatePresignedUrlForCsvJsonResponseBody {
        file_metadata: File::from_details(
            Some(file_id),
            &create_presigned_put_url_data.file_name,
            0,
            create_presigned_put_url_data
                .tag_set
                .map(|t| t.into_iter().map(Some).collect()),
            create_presigned_put_url_data.metadata.clone(),
            create_presigned_put_url_data.link.clone(),
            create_presigned_put_url_data.time_stamp.clone(),
            dataset_org_plan_sub.dataset.id,
        ),
        presigned_put_url,
    };

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize, Debug, Serialize, ToSchema)]
pub struct DatasetFilePathParams {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct FileData {
    pub file_and_group_ids: Vec<FileAndGroupId>,
    pub total_pages: i64,
}

/// Get Files and Group IDs for Dataset
///
/// Get all files and their group ids which belong to a given dataset specified by the dataset_id parameter. 10 files and group ids are returned per page. This route may return the same file multiple times if the file is associated with multiple groups.
#[utoipa::path(
    get,
    path = "/dataset/files/{dataset_id}/{page}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "JSON body representing the files and their group ids in the current dataset", body = FileData),
        (status = 400, description = "Service error relating to getting the files in the current datase", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch files for."),
        ("page" = u64, description = "The page number of files you wish to fetch. Each page contains at most 10 files."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[deprecated]
pub async fn get_dataset_files_and_group_ids_handler(
    data: web::Path<DatasetFilePathParams>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    if dataset_org_plan_sub.dataset.id != data.dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match given path".to_string(),
        )
        .into());
    }

    let files = get_dataset_files_and_group_ids_query(data.dataset_id, data.page, pool).await?;

    Ok(HttpResponse::Ok().json(FileData {
        file_and_group_ids: files
            .iter()
            .map(|f| FileAndGroupId {
                file: f.0.clone(),
                group_id: f.2,
            })
            .collect(),
        total_pages: files
            .first()
            .map(|file| (file.1 as f64 / 10.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetFilesCursorReqQuery {
    /// File ids are compared to the cursor using a greater than or equal to. This is used to paginate through files.
    pub cursor: Option<uuid::Uuid>,
    /// The page size of files you wish to fetch. Defaults to 10.
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(title = "GetFilesCursorResponseBody")]
pub struct GetFilesCursorResponseBody {
    /// This is a paginated list of files and their associated groups. The page size is specified in the request. The cursor is used to fetch the next page of files.
    pub file_with_chunk_groups: Vec<FileWithChunkGroups>,
    /// Parameter for the next cursor offset. This is used to fetch the next page of files. If there are no more files, this will be None.
    pub next_cursor: Option<uuid::Uuid>,
}

/// Scroll Files with Groups
///
/// Scroll through the files along with their groups in a dataset. This is useful for paginating through files. The cursor is used to fetch the next page of files. The page size is used to specify how many files to fetch per page. The default page size is 10.
#[utoipa::path(
    get,
    path = "/dataset/scroll_files",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "JSON body representing the files along with their associated groups in the current dataset", body = GetFilesCursorResponseBody),
        (status = 400, description = "Service error relating to getting the files in the current datase", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("cursor" = Option<uuid::Uuid>, Query, description = "The cursor to fetch files from. If not specified, will fetch from the beginning. File ids are compared to the cursor using a greater than or equal to."),
        ("page_size" = Option<u64>, Query, description = "The page size of files you wish to fetch. Defaults to 10."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_files_cursor_handler(
    data: web::Query<GetFilesCursorReqQuery>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    let page_size = data.page_size;
    let cursor = data.cursor;

    let files_cursor_resp_body = get_files_query(
        dataset_org_plan_sub.dataset.id,
        cursor,
        page_size,
        pool.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(files_cursor_resp_body))
}

/// Delete File
///
/// Delete a file from S3 attached to the server based on its id. This will disassociate chunks from the file, but only delete them all together if you specify delete_chunks to be true. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 204, description = "Confirmation that the file has been deleted"),
        (status = 400, description = "Service error relating to finding or deleting the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("file_id" = uuid::Uuid, description = "The id of the file to delete"),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    query: web::Query<DeleteGroupData>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    delete_file_query(
        file_id.into_inner(),
        query.delete_chunks,
        dataset_org_plan_sub.dataset,
        pool,
        dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetImageResponse {
    pub signed_url: String,
}

/// Get presigned url for file (deprecated)
///
/// Returns the presigned url for a file.
#[utoipa::path(
    get,
    path = "/get_signed_url/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "JSON body representing the signed url", body = GetImageResponse),
        (status = 400, description = "Service error relating to getting the signed url", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("ttl" = u32, Query, description = "The time to live of the signed url in seconds"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[deprecated]
pub async fn get_signed_url(
    file_id: web::Path<String>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let bucket = get_aws_bucket()?;

    let unlimited = std::env::var("UNLIMITED")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let s3_path = match unlimited {
        true => "files".to_string(),
        false => dataset_org_plan_sub
            .organization
            .organization
            .id
            .to_string(),
    };

    let signed_url = bucket
        .presign_get(format!("{}/{}", s3_path, file_id.into_inner()), 86400, None)
        .await
        .map_err(|e| {
            log::error!("Error getting signed url: {}", e);
            ServiceError::BadRequest(format!("Error getting signed url: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(GetImageResponse { signed_url }))
}
