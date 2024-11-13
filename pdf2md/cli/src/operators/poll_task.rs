pub fn poll_task(task_id: &str, base_url: &str, api_key: &str) {
    loop {
        let request = ureq::get(format!("{}/api/task/{}", base_url, task_id).as_str())
            .set("Content-Type", "application/json")
            .set("Authorization", api_key)
            .call()
            .expect("Failed to send request");

        let response: serde_json::Value = request.into_json().expect("Failed to parse response");

        if (response["status"] == "Completed"
            || response["total_document_pages"].as_i64() != Some(0))
            && response["chunks"].as_array() != Some(&vec![])
        {
            println!("{}", response);
            break;
        } else {
            println!("Task is still processing...");
            println!("{}", response);
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
