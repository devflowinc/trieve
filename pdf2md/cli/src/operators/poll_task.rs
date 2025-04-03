use std::sync::Arc;

pub fn poll_task(task_id: &str, base_url: &str, api_key: &str) {
    loop {
        let request = ureq::AgentBuilder::new()
            .tls_connector(Arc::new(
                native_tls::TlsConnector::new().expect("Failed to create TLS connector"),
            ))
            .build()
            .get(format!("{}/api/task/{}", base_url, task_id).as_str())
            .set("Content-Type", "application/json")
            .set("Authorization", api_key)
            .call()
            .expect("Failed to send request");

        let response: serde_json::Value = request.into_json().expect("Failed to parse response");

        if response["status"] == "Completed" || response["status"] == "Failed" {
            println!("{}", response);
            break;
        } else {
            println!("Task is still processing...");
            println!("{}", response);
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
