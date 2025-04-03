use std::collections::HashMap;

use serde_json::Value;

use crate::{errors::ServiceError, models::WebhookPayloadData};

pub async fn send_webhook(
    webhook_url: Option<String>,
    template: Option<String>,
    data: WebhookPayloadData,
) -> Result<(), ServiceError> {
    if let Some(url) = webhook_url {
        let client = reqwest::Client::new();
        let body = render_webhook_payload(template.clone(), &data)?;
        client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Failed to send webhook: {}", e)))?
            .error_for_status()
            .map_err(|e| ServiceError::BadRequest(format!("Failed to send webhook: {}", e)))?;

        Ok(())
    } else {
        Ok(())
    }
}

pub fn render_webhook_payload(
    template: Option<String>,
    data: &WebhookPayloadData,
) -> Result<String, ServiceError> {
    let default_template = r#"{
        "id": "{{task_id}}",
        "fileName": "{{file_name}}",
        "totalPages": "{{pages}}",
        "pagesProcessed": "{{pages_processed}}",
        "content": "{{content}}",
        "status": "{{status}}",
        "timestamp": "{{timestamp}}"
        "metadata": {{metadata}}
    }"#;

    let template = template.unwrap_or(default_template.to_string());

    let mut replacements = HashMap::new();
    replacements.insert("task_id", data.task_id.to_string());
    replacements.insert("file_name", data.file_name.to_string());
    replacements.insert("pages", data.pages.to_string());
    replacements.insert("page_num", data.page_num.to_string());
    replacements.insert("pages_processed", data.pages_processed.to_string());
    replacements.insert("content", data.content.to_string());
    replacements.insert("status", data.status.to_string());
    replacements.insert("timestamp", data.timestamp.to_string());

    let metadata_value = if let Ok(parsed) = serde_json::from_str::<Value>(&data.usage) {
        serde_json::to_string(&parsed)
            .map_err(|e| ServiceError::BadRequest(format!("Invalid JSON in metadata: {}", e)))?
    } else {
        serde_json::to_string(&data.usage)
            .map_err(|e| ServiceError::BadRequest(format!("Invalid JSON in metadata: {}", e)))?
    };
    replacements.insert("metadata", metadata_value);

    let mut processed = template.clone();
    for (key, value) in &replacements {
        if ["pages", "pages_processed", "page_num"].contains(key) {
            processed = processed.replace(&format!("\"{{{{{}}}}}\"", key), value);
            processed = processed.replace(&format!("{{{{{}}}}}", key), value);
        } else {
            processed = processed.replace(&format!("{{{{{}}}}}", key), value);
        }
    }

    let parsed: Value = serde_json::from_str(&processed)
        .map_err(|e| ServiceError::BadRequest(format!("Invalid JSON after processing: {}", e)))?;

    serde_json::to_string_pretty(&parsed)
        .map_err(|e| ServiceError::BadRequest(format!("Failed to pretty-print JSON: {}", e)))
}

#[cfg(test)]
mod tests {
    use s3::creds::time::OffsetDateTime;

    use crate::models::UploadFileReqPayload;

    use super::*;
    #[test]
    fn test_webhook_payload_templating() {
        let payload = UploadFileReqPayload {
            base64_file: "test".to_string(),
            webhook_url: Some("https://example.com".to_string()),
            file_name: "test.pdf".to_string(),
            provider: None,
            webhook_payload_template: Some(
                r#"{
                "taskInfo": {
                    "id": "{{task_id}}",
                    "fileName": "{{file_name}}",
                    "totalPages": {{pages}},
                    "pagesProcessed": {{pages_processed}}
                },
                "metadata": {{metadata}},
                "content": "{{content}}",
                "status": "{{status}}",
                "page_num": "{{page_num}}",
                "timestamp": "{{timestamp}}"
            }"#
                .to_string(),
            ),
            llm_model: None,
            llm_api_key: None,
            system_prompt: None,
            chunkr_api_key: None,
            chunkr_create_task_req_payload: None,
        };

        let data = WebhookPayloadData {
            task_id: "123".to_string(),
            file_name: "test.pdf".to_string(),
            pages: 10,
            pages_processed: 7,
            content: "Sample content".to_string(),
            page_num: 3,
            usage: r#"{"author": "John Doe", "created": "2024-01-01"}"#.to_string(),
            status: "processing".to_string(),
            timestamp: OffsetDateTime::now_utc().to_string(),
        };

        let rendered = render_webhook_payload(payload.webhook_payload_template, &data).unwrap();
        println!("Rendered payload: {}", rendered);

        // Verify the result is valid JSON and contains expected values
        let parsed: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["taskInfo"]["id"], "123");
        assert_eq!(parsed["taskInfo"]["totalPages"], 10);

        // Verify nested JSON in metadata is properly handled
        assert_eq!(parsed["metadata"]["author"], "John Doe");
    }

    #[test]
    fn test_invalid_template_handling() {
        let payload = UploadFileReqPayload {
            base64_file: "test".to_string(),
            webhook_url: Some("https://example.com".to_string()),
            file_name: "test.pdf".to_string(),
            provider: None,
            webhook_payload_template: Some(
                r#"{
                "metadata": {{metadata}},
                "status": "{{status}}",
            }"#
                .to_string(),
            ),
            llm_model: None,
            llm_api_key: None,
            system_prompt: None,
            chunkr_api_key: None,
            chunkr_create_task_req_payload: None,
        };

        let data = WebhookPayloadData {
            task_id: "123".to_string(),
            file_name: "test.pdf".to_string(),
            pages: 10,
            pages_processed: 7,
            content: "Sample content".to_string(),
            page_num: 3,
            usage: "Invalid JSON".to_string(), // Not valid JSON
            status: "processing".to_string(),
            timestamp: OffsetDateTime::now_utc().to_string(),
        };

        let err = render_webhook_payload(payload.webhook_payload_template, &data)
            .err()
            .unwrap();

        assert_eq!(
            err,
            ServiceError::BadRequest(
                "Invalid JSON after processing: trailing comma at line 4 column 13".to_string()
            )
        );
    }
}
