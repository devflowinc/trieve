extern crate hallucination_detection;
use csv::WriterBuilder;
use hallucination_detection::HallucinationDetector;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]

struct SourceInfo {
    source_id: String,
    task_type: String,
    source: String,
    source_info: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Label {
    start: usize,
    end: usize,
    text: String,
    meta: String,
    label_type: String,
    implicit_true: bool,
    due_to_null: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResponseData {
    id: String,
    source_id: String,
    model: String,
    temperature: f64,
    labels: Vec<Label>,
    split: String,
    quality: String,
    response: String,
}

#[derive(Debug, Serialize)]
struct TestResult {
    source: String,
    response: String,
    hallucination_score: String,
    matches_expected: bool,
}

async fn fetch_jsonl_data(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    Ok(content)
}

async fn load_summary_source_ids() -> Result<(HashSet<String>, Vec<SourceInfo>), Box<dyn Error>> {
    let source_url =
        "https://github.com/ParticleMedia/RAGTruth/raw/refs/heads/main/dataset/source_info.jsonl";
    let content = fetch_jsonl_data(source_url).await?;
    let reader = BufReader::new(content.as_bytes());

    let mut summary_ids = HashSet::new();
    let mut source_info_map = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(line) = serde_json::from_str(&line) {
            let source_info: SourceInfo = line;
            if source_info.task_type == "Summary" {
                summary_ids.insert(source_info.source_id.clone());
                source_info_map.push(source_info);
            }
        }
    }

    Ok((summary_ids, source_info_map))
}

async fn run_hallucination_test() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    // Create output file
    let output_path = Path::new("hallucination_results.csv");
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    let detector = HallucinationDetector::new(Default::default())?;

    // Load summary source IDs and info
    println!("Loading summary source IDs...");
    let (summary_ids, source_info_vec) = load_summary_source_ids().await?;
    println!("Found {} summary tasks", summary_ids.len());

    // Create a mapping of source_id to source_info for quick lookup
    let source_info_map: std::collections::HashMap<String, String> = source_info_vec
        .into_iter()
        .map(|info| (info.source_id, info.source_info))
        .collect();

    // Read and process responses
    let response_url =
        "https://github.com/ParticleMedia/RAGTruth/raw/refs/heads/main/dataset/response.jsonl";
    let response_content = fetch_jsonl_data(response_url).await?;
    let reader = BufReader::new(response_content.as_bytes());

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let record: ResponseData = serde_json::from_str(&line)?;

        // Skip if not a summary task
        if !summary_ids.contains(&record.source_id) {
            continue;
        }

        println!("Processing record {} (ID: {})", i, record.id);

        // Get source info for this response
        let source_info = source_info_map
            .get(&record.source_id)
            .ok_or("Source info not found")?;

        // Determine if this is ground truth based on empty labels
        let is_ground_truth = record.labels.is_empty();

        // Run hallucination detection comparing response against source info
        let start = std::time::Instant::now();
        let hallucination_score = detector
            .detect_hallucinations(&record.response, &[source_info.clone()])
            .await;
        let elapsed = start.elapsed();
        println!("Hallucination detection took: {:?}", elapsed);

        // Create test result record
        let test_result = TestResult {
            response: record.response,
            source: source_info.clone(),
            hallucination_score: serde_json::to_string(&hallucination_score)?,
            matches_expected: (hallucination_score.total_score > 0.3 && !is_ground_truth)
                || (hallucination_score.total_score <= 0.3 && is_ground_truth),
        };

        // Write result to JSONL
        csv_writer.serialize(test_result)?;
    }

    csv_writer.flush()?;
    println!("Testing completed. Results written to hallucination_results.jsonl");

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run_hallucination_test().await {
        eprintln!("Error running hallucination test: {}", err);
        std::process::exit(1);
    }
}
