# hallucination-detection

A high-performance Rust library for detecting hallucinations in Large Language Model (LLM) outputs using BERT Named Entity Recognition (NER), proper noun analysis, and numerical comparisons.

[![Crates.io](https://img.shields.io/crates/v/hallucination-detection)](https://crates.io/crates/hallucination-detection)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Fast and accurate hallucination detection for RAG (Retrieval-Augmented Generation) systems
- BERT-based Named Entity Recognition for proper noun analysis
- Numerical comparison and validation
- Unknown word detection using comprehensive English word dictionary
- Configurable scoring weights and detection options
- Async/await support with Tokio runtime
- Optional ONNX support for improved performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hallucination-detection = "0.1.1"
```

If you want to use NER and ONNX features:

```toml
[dependencies]
hallucination-detection = { version = "0.1.1", features = ["ner", "onnx"] }
```

## Quick Start

```rust
use hallucination_detection::{HallucinationDetector, HallucinationOptions};

#[tokio::main]
async fn main() {
    // Create detector with default options
    let detector = HallucinationDetector::new(Default::default())
        .expect("Failed to create detector");

    // Example texts
    let llm_output = String::from("Tesla sold 500,000 cars in Europe last quarter.");
    let references = vec![
        String::from("Tesla reported strong sales in European markets."),
        String::from("The company's global deliveries increased.")
    ];

    // Detect hallucinations
    let score = detector.detect_hallucinations(&llm_output, &references).await;

    println!("Hallucination Score: {:#?}", score);
}
```

## Configuration

You can customize the detector's behavior using `HallucinationOptions`:

```rust
use hallucination_detection::{HallucinationOptions, ScoreWeights};

let options = HallucinationOptions {
    weights: ScoreWeights {
        proper_noun_weight: 0.4,
        unknown_word_weight: 0.1,
        number_mismatch_weight: 0.5,
    },
    use_ner: true,
};

let detector = HallucinationDetector::new(options)
    .expect("Failed to create detector");
```

## Output

The detector returns a `HallucinationScore` struct containing:

```rust
pub struct HallucinationScore {
    pub proper_noun_score: f64,
    pub unknown_word_score: f64,
    pub number_mismatch_score: f64,
    pub total_score: f64,
    pub detected_hallucinations: Vec<String>,
}
```

- Scores range from 0.0 (no hallucination) to 1.0 (complete hallucination)
- `detected_hallucinations` contains specific elements that were flagged

## Performance Considerations

- The NER model is loaded once and reused across predictions
- English word dictionary is cached locally for faster subsequent runs
- Async operations allow for non-blocking execution
- ONNX runtime provides optimized model inference

## Features Flags

- `ner`: Enables BERT Named Entity Recognition (default: enabled)
- `onnx`: Uses ONNX runtime for improved performance (default: enabled)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Authors

Devflow Inc. <humans@trieve.ai>

## Acknowledgments

- Uses [rust-bert](https://github.com/guillaume-be/rust-bert) for NER capabilities
- English word list from [dwyl/english-words](https://github.com/dwyl/english-words)