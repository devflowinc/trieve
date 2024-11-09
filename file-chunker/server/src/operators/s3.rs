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
