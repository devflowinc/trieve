use lazy_static::lazy_static;
use regex::Regex;
use rust_bert::{
    pipelines::{
        common::{ModelResource, ModelType, ONNXModelResources},
        ner::NERModel,
        token_classification::{LabelAggregationOption, TokenClassificationConfig},
    },
    resources::RemoteResource,
};
use rust_stemmers::{Algorithm, Stemmer};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref NUMBER_REGEX: Regex = Regex::new(r"-?\d*\.?\d+").unwrap();
    static ref WORD_BOUNDARY_REGEX: Regex = Regex::new(r"\b\w+\b").unwrap();
    static ref ENGLISH_WORDS: HashSet<String> = {
        std::fs::read_to_string("/home/denssumesh/Documents/trieve/trieve/server/src/words.txt")
            .expect("Failed to read words file")
            .lines()
            .map(|s| s.to_lowercase())
            .collect()
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HallucinationScore {
    proper_noun_score: f64,
    unknown_word_score: f64,
    number_mismatch_score: f64,
    total_score: f64,
    detected_hallucinations: Vec<String>,
}

pub struct HallucinationDetector {
    ner_model: NERModel,
    stemmer: Stemmer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextAnalysis {
    proper_nouns: HashSet<String>,
    unknown_words: Vec<String>,
    numbers: Vec<f64>,
}

impl HallucinationDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ner_model: NERModel::new(TokenClassificationConfig::new(
                ModelType::Bert,
                ModelResource::ONNX(ONNXModelResources {
                    encoder_resource: Some(Box::new(RemoteResource::new(
                        "https://huggingface.co/optimum/bert-base-NER/resolve/main/model.onnx",
                        "onnx-bert-base-NER",
                    ))),
                    ..Default::default()
                }),
                RemoteResource::new(
                    "https://huggingface.co/optimum/bert-base-NER/resolve/main/config.json",
                    "onnx-bert-base-NER",
                ),
                RemoteResource::new(
                    "https://huggingface.co/optimum/bert-base-NER/resolve/main/vocab.txt",
                    "onnx-bert-base-NER",
                ),
                None,
                false,
                None,
                None,
                LabelAggregationOption::First,
            ))?,
            stemmer: Stemmer::create(Algorithm::English),
        })
    }

    pub fn detect_hallucinations(
        &self,
        llm_output: String,
        references: &[String],
    ) -> HallucinationScore {
        let mut all_texts = vec![llm_output.to_string()];
        all_texts.extend(references.iter().cloned());

        // Analyze everything in one batch
        let all_analyses = self.analyze_text(&all_texts);

        let (output_analysis, ref_analyses) = all_analyses.split_first().unwrap();

        let all_ref_proper_nouns: HashSet<_> = ref_analyses
            .iter()
            .flat_map(|analysis| analysis.proper_nouns.iter().cloned())
            .collect();

        let all_ref_numbers: Vec<_> = ref_analyses
            .iter()
            .flat_map(|analysis| analysis.numbers.iter().cloned())
            .collect();

        // Calculate differences and scores
        let proper_noun_diff: Vec<_> = output_analysis
            .proper_nouns
            .difference(&all_ref_proper_nouns)
            .cloned()
            .collect();

        let number_diff = self.compare_numbers(&output_analysis.numbers, &all_ref_numbers);

        let proper_noun_score = (!proper_noun_diff.is_empty()) as u8 as f64;
        let unknown_word_score = (output_analysis.unknown_words.len() as f64 / 100.0).min(1.0);
        let number_mismatch_score = (!number_diff.is_empty()) as u8 as f64;

        let total_score =
            (proper_noun_score * 0.4 + unknown_word_score * 0.1 + number_mismatch_score * 0.5)
                .clamp(0.0, 1.0);

        HallucinationScore {
            proper_noun_score,
            unknown_word_score,
            number_mismatch_score,
            total_score,
            detected_hallucinations: [
                proper_noun_diff,
                output_analysis.unknown_words.clone(),
                number_diff.iter().map(|n| n.to_string()).collect(),
            ]
            .concat(),
        }
    }

    fn analyze_text(&self, texts: &[String]) -> Vec<TextAnalysis> {
        // Process all texts in one NER batch
        let all_entities = self.ner_model.predict(texts);

        // Process each text in parallel with the corresponding NER results
        texts
            .iter()
            .zip(all_entities.iter())
            .map(|(text, entities)| {
                let mut unknown_words = Vec::new();
                let mut numbers = Vec::new();

                // Convert entities to proper nouns for this specific text
                let proper_nouns: HashSet<String> = entities
                    .iter()
                    .map(|entity| entity.word.to_lowercase())
                    .collect();

                let mut word_map = HashMap::new();

                // Process words in this text
                for cap in WORD_BOUNDARY_REGEX.find_iter(text) {
                    let word = cap.as_str();
                    let word_lower = word.to_lowercase();

                    if let Ok(num) = word.parse::<f64>() {
                        numbers.push(num);
                        continue;
                    }

                    // Process each word only once
                    word_map.entry(word_lower.clone()).or_insert_with(|| {
                        if !proper_nouns.contains(&word_lower)
                            && !ENGLISH_WORDS.contains(&word_lower)
                            && !ENGLISH_WORDS.contains(&self.stemmer.stem(&word_lower).to_string())
                        {
                            unknown_words.push(word.to_string());
                        }
                        true
                    });
                }

                TextAnalysis {
                    proper_nouns,
                    unknown_words,
                    numbers,
                }
            })
            .collect()
    }

    fn compare_numbers(&self, output_numbers: &[f64], ref_numbers: &[f64]) -> Vec<f64> {
        output_numbers
            .iter()
            .filter(|&num| {
                !ref_numbers
                    .iter()
                    .any(|ref_num| (num - ref_num).abs() < 1e-10)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::sync::Once;

    #[derive(Debug)]
    struct ExpectedScores {
        proper_noun: bool,
        number: bool,
        total_min: f64,
    }

    static INIT: Once = Once::new();
    static mut DETECTOR: Option<HallucinationDetector> = None;

    // Helper function to create detector instance for tests
    fn get_detector() -> &'static HallucinationDetector {
        unsafe {
            INIT.call_once(|| {
                DETECTOR = Some(HallucinationDetector::new().expect("Failed to create detector"));
            });
            DETECTOR.as_ref().unwrap()
        }
    }

    #[test]
    fn test_zero_hallucination() {
        let detector = get_detector();
        let llm_output = String::from("Elon Musk is the CEO of Tesla.");
        let references = vec![String::from("Elon Musk is the CEO of Tesla.")];

        let score = detector.detect_hallucinations(llm_output, &references);
        println!("Zero Hallucination Score: {:?}", score);

        assert_eq!(score.proper_noun_score, 0.0);
        assert_eq!(score.number_mismatch_score, 0.0);
        assert!(score.total_score < 1e-2);
        assert!(score.detected_hallucinations.is_empty());
    }

    #[test]
    fn test_multiple_references() {
        let detector = get_detector();
        let llm_output =
            String::from("Apple and Microsoft are tech companies worth 3 trillion dollars.");
        let references = vec![
            String::from("Apple's market cap reached 3 trillion dollars."),
            String::from("Microsoft is a leading tech company."),
        ];

        let score = detector.detect_hallucinations(llm_output, &references);
        println!("Multiple References Score: {:?}", score);
        assert_eq!(score.proper_noun_score, 0.0); // Both companies are in references
        assert_eq!(score.number_mismatch_score, 0.0); // Number matches reference
    }

    #[test]
    fn test_edge_cases() {
        let detector = get_detector();

        // Empty input
        let score_empty = detector.detect_hallucinations(String::from(""), &[String::from("")]);
        assert_eq!(score_empty.total_score, 0.0);

        // Only numbers
        let score_numbers = detector
            .detect_hallucinations(String::from("123 456.789"), &[String::from("123 456.789")]);
        assert_eq!(score_numbers.number_mismatch_score, 0.0);

        // Only proper nouns
        let score_nouns = detector.detect_hallucinations(
            String::from("John Smith"),
            &[String::from("Different Person")],
        );
        assert!(score_nouns.proper_noun_score > 0.0);
    }

    #[rstest]
    #[case(
        "Apple announced new offices in Seattle and Portland.",
        vec!["Apple is expanding its presence in Seattle."],
        ExpectedScores { proper_noun: true, number: false, total_min: 0.3 },
        "Location hallucination"
    )]
    #[case(
        "Microsoft hired 5000 engineers in 2023.",
        vec!["Microsoft expanded its workforce by 3000 people in 2023."],
        ExpectedScores { proper_noun: false, number: true, total_min: 0.3 },
        "Number hallucination"
    )]
    #[case(
        "Google opened a 50000 sqft office in Miami with 2500 employees.",
        vec!["Google is expanding in Florida.", "The company plans to hire new employees."],
        ExpectedScores { proper_noun: true, number: true, total_min: 0.5 },
        "Multiple hallucinations"
    )]
    #[case(
        "Samsung launched the Galaxy X1 for $899 with 12GB RAM.",
        vec!["Samsung announced a new Galaxy phone with advanced features."],
        ExpectedScores { proper_noun: true, number: true, total_min: 0.3 },
        "Product details hallucination"
    )]
    #[case(
        "Tesla revealed Model Y in Berlin on March 15.",
        vec!["Tesla is planning to reveal new models in Europe."],
        ExpectedScores { proper_noun: true, number: true, total_min: 0.4 },
        "Event details hallucination"
    )]
    #[case(
        "Amazon reported revenue of 514 billion dollars in Q4.",
        vec!["Amazon's Q4 revenue was 513.5 billion dollars."],
        ExpectedScores { proper_noun: false, number: true, total_min: 0.2 },
        "Financial number hallucination"
    )]
    #[case(
        "Netflix gained 8.5M subscribers in Asia, reaching 15M total subscribers.",
        vec!["Netflix reported subscriber growth in Asia."],
        ExpectedScores { proper_noun: false, number: true, total_min: 0.4 },
        "Metric hallucination"
    )]
    #[case(
        "Study shows 75% of participants improved with new treatment.",
        vec!["The study demonstrated positive results with the new treatment."],
        ExpectedScores { proper_noun: false, number: true, total_min: 0.3 },
        "Statistical hallucination"
    )]
    #[case(
        "Dr. Smith at Stanford University found significant results in 89% of cases.",
        vec!["Recent research showed promising results in the majority of cases."],
        ExpectedScores { proper_noun: true, number: true, total_min: 0.5 },
        "Research details hallucination"
    )]
    #[case(
        "According to Dr. Johnson at Harvard, 87% of the 1500 participants in Boston showed improvement.",
        vec!["A recent medical study showed positive results in participants."],
        ExpectedScores { proper_noun: true, number: true, total_min: 0.6 },
        "Mixed hallucination"
    )]
    fn test_hallucination_detection(
        #[case] llm_output: &str,
        #[case] references: Vec<&str>,
        #[case] expected: ExpectedScores,
        #[case] test_name: &str,
    ) {
        let detector = get_detector();
        let score = detector.detect_hallucinations(
            String::from(llm_output),
            &references.into_iter().map(String::from).collect::<Vec<_>>(),
        );

        println!("Test '{}' Score: {:?}", test_name, score);

        if expected.proper_noun {
            assert!(
                score.proper_noun_score > 0.0,
                "{}: Should detect proper noun hallucination",
                test_name
            );
        }

        if expected.number {
            assert!(
                score.number_mismatch_score > 0.0,
                "{}: Should detect number hallucination",
                test_name
            );
        }

        assert!(
            score.total_score > expected.total_min,
            "{}: Total score should be above {}",
            test_name,
            expected.total_min
        );
    }
}
