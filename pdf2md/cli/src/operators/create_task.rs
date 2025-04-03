use std::sync::Arc;

use base64::Engine;

pub fn create_task(file: &str, base_url: &str, api_key: &str) {
    let file_buf = std::fs::read(file).expect("Failed to read file");
    let file_base64 = base64::prelude::BASE64_STANDARD.encode(file_buf);

    let request = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(
            native_tls::TlsConnector::new().expect("Failed to create TLS connector"),
        ))
        .build()
        .post(format!("{}/api/task", base_url).as_str())
        .set("Content-Type", "application/json")
        .set("Authorization", api_key)
        .send_json(serde_json::json!({
            "base64_file": file_base64,
            "file_name": file,
        }))
        .map_err(|e| e.to_string())
        .expect("Failed to send request");

    let response: serde_json::Value = request.into_json().expect("Failed to parse response");

    println!("{}", response);
}
