use base64::Engine;

pub fn create_task(file: &str, base_url: &str, api_key: &str) {
    let file = std::fs::read(file).expect("Failed to read file");
    let file = base64::prelude::BASE64_STANDARD.encode(file);

    let request = ureq::post(format!("{}/api/task/create", base_url).as_str())
        .set("Content-Type", "application/json")
        .set("Authorization", api_key)
        .send_json(serde_json::json!({
            "base64_file": file,
        }))
        .expect("Failed to send request");

    let response: serde_json::Value = request.into_json().expect("Failed to parse response");

    println!("{}", response);
}
