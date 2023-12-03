use crate::errors::DefaultError;
use serde_json::json;

// create index
pub fn create_lnx_index(index_name: String) -> Result<(), DefaultError> {
    let lnx_URL = std::env::var("LNX_URL")
        .expect("LNX_URL must be set")
        .to_string();

    let request_body = json!({
        "index" {
            "name": index_name,
            "storage_type": "filesystem",
            "fields": {
                "ids": {
                    "type": "string",
                    "indexed": true,
                    "stored": true
                }
            }
        },
    });

    let lnx_client = reqwest::Client::new();

    Ok(())
}
