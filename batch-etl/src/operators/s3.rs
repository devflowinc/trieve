use s3::{creds::Credentials, Bucket, Region};

use crate::{errors::ServiceError, get_env};

pub fn get_aws_bucket() -> Result<Bucket, ServiceError> {
    let aws_region_name = std::env::var("AWS_REGION").unwrap_or("".to_string());
    let s3_endpoint = get_env!("S3_ENDPOINT", "S3_ENDPOINT should be set").into();
    let s3_bucket_name = get_env!("S3_BUCKET", "S3_BUCKET should be set");

    let aws_region = Region::Custom {
        region: aws_region_name,
        endpoint: s3_endpoint,
    };

    let aws_credentials = if let Ok(creds) = Credentials::from_instance_metadata() {
        creds
    } else {
        let s3_access_key = get_env!("S3_ACCESS_KEY", "S3_ACCESS_KEY should be set").into();
        let s3_secret_key = get_env!("S3_SECRET_KEY", "S3_SECRET_KEY should be set").into();
        Credentials {
            access_key: Some(s3_access_key),
            secret_key: Some(s3_secret_key),
            security_token: None,
            session_token: None,
            expiration: None,
        }
    };

    let aws_bucket = Bucket::new(s3_bucket_name, aws_region, aws_credentials)
        .map_err(|e| {
            log::error!("Could not create or get bucket {:?}", e);
            ServiceError::BadRequest("Could not create or get bucket".to_string())
        })?
        .with_path_style();

    Ok(*aws_bucket)
}

pub async fn get_signed_url(bucket: &Bucket, key: String) -> Result<String, ServiceError> {
    let url = bucket.presign_get(key, 86400, None).await.map_err(|e| {
        log::error!("Could not get signed url {:?}", e);
        ServiceError::BadRequest("Could not get signed url".to_string())
    })?;

    Ok(url)
}

pub async fn upload_to_s3(bucket: &Bucket, key: String, data: &[u8]) -> Result<(), ServiceError> {
    bucket.put_object(key, data).await.map_err(|e| {
        log::error!("Failed to upload to S3: {:?}", e);
        ServiceError::InternalServerError("Failed to upload to S3".to_string())
    })?;

    Ok(())
}

pub async fn download_from_s3(bucket: &Bucket, key: String) -> Result<Vec<u8>, ServiceError> {
    let data = bucket.get_object(key).await.map_err(|e| {
        log::error!("Failed to download from S3: {:?}", e);
        ServiceError::InternalServerError("Failed to download from S3".to_string())
    })?;

    Ok(data.to_vec())
}

pub async fn get_presigned_put_url(
    bucket: &Bucket,
    key: String,
    expires: u32,
) -> Result<String, ServiceError> {
    let url = bucket
        .presign_put(key, expires, None, None)
        .await
        .map_err(|e| {
            log::error!("Could not get presigned put url: {:?}", e);
            ServiceError::BadRequest("Could not get presigned put url".to_string())
        })?;

    Ok(url)
}
