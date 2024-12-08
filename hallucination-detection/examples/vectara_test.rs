extern crate hallucination_detection;
use csv::{Reader, WriterBuilder};
use hallucination_detection::HallucinationDetector;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;
use std::path::Path; // Assuming this crate exists

#[derive(Debug, Serialize)]
struct TestResult {
    source: String,
    llm_output: String,
    hallucination_score: String,
    matches_expected: bool,
}

#[derive(Debug, Deserialize)]
struct InputData {
    #[serde(rename = "Dataset")]
    _dataset: String,
    #[serde(rename = "Model")]
    _model: String,
    #[serde(rename = "Reference/Sources")]
    source: String,
    #[serde(rename = "Original Summary")]
    og_sum: String,
    #[serde(rename = "Corrected Summary")]
    _corr_sum: String,
    #[serde(rename = "Original Summary HHEM Factuality Prediction")]
    factuality_score: f64,
    #[serde(rename = "Corrected Summary HHEM Factuality Prediction")]
    _corrected_factuality_score: f64,
}

async fn fetch_csv_data() -> Result<String, Box<dyn Error>> {
    let response = reqwest::get("https://huggingface.co/datasets/vectara/hcm-examples-aug-2024/resolve/main/Hallucination%20Correction%20Examples.csv").await?;
    let content = response.text().await?;
    Ok(content)
}

fn split_references(text: &str) -> Vec<String> {
    // Create regex to match "Reference [N]:" pattern
    let re = Regex::new(r"Reference \[\d+\]:").unwrap();

    // Get all match positions
    let matches: Vec<_> = re.find_iter(text).map(|m| m.start()).collect();

    // Split text at match positions
    let mut references = Vec::new();

    for i in 0..matches.len() {
        let start = matches[i];
        let end = if i < matches.len() - 1 {
            matches[i + 1]
        } else {
            text.len()
        };

        // Extract the reference content and trim whitespace
        let content = text[start..end].trim();
        references.push(content.to_string());
    }

    references
}

async fn run_hallucination_test() -> Result<(), Box<dyn Error>> {
    // Create CSV writer
    let output_path = Path::new("hallucination_results.csv");
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    let detector = HallucinationDetector::new(Default::default())?;
    // Read input data
    let csv_content = fetch_csv_data().await?;

    let mut rdr = Reader::from_reader(Cursor::new(csv_content));

    for (i, result) in rdr.deserialize().enumerate() {
        let record: InputData = result?;
        println!("Processing record {}", i);

        let references = split_references(&record.source);

        // Detect hallucinations using the provided function
        let start = std::time::Instant::now();
        let hallucination_score = detector
            .detect_hallucinations(&record.og_sum, &references)
            .await
            .unwrap();
        let elapsed = start.elapsed();
        println!("Hallucination detection took: {:?}", elapsed);

        // Create test result record
        let test_result = TestResult {
            source: record.source,
            llm_output: record.og_sum,
            hallucination_score: serde_json::to_string(&hallucination_score)?,
            matches_expected: (hallucination_score.total_score > 0.3
                && record.factuality_score < 0.5)
                || (hallucination_score.total_score < 0.3 && record.factuality_score > 0.5),
        };

        // Write result to CSV
        writer.serialize(test_result)?;
    }

    writer.flush()?;
    println!("Testing completed. Results written to hallucination_results.csv");

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run_hallucination_test().await {
        eprintln!("Error running hallucination test: {}", err);
        std::process::exit(1);
    }
}
