use crate::{
    errors::ServiceError,
    handlers::{chunk_handler::ChunkReqPayload, webhook_handler::ContentValue},
};

pub async fn publish_content<T: Into<ChunkReqPayload>>(
    dataset: uuid::Uuid,
    value: T,
) -> Result<(), ServiceError> {
    todo!();
}
