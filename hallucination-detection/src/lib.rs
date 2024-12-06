use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::OnceCell;

#[cfg(feature = "ner")]
use {
    rust_bert::{
        pipelines::{
            common::{ModelResource, ModelType, ONNXModelResources},
            ner::{Entity, NERModel},
            token_classification::{LabelAggregationOption, TokenClassificationConfig},
        },
        resources::RemoteResource,
        RustBertError,
    },
    std::sync::mpsc,
    tokio::{sync::oneshot, task::JoinHandle},
};

const WORDS_URL: &str =
    "https://raw.githubusercontent.com/dwyl/english-words/refs/heads/master/words.txt";
const CACHE_FILE: &str = "~/.cache/hallucination-detection/english_words_cache.txt";

static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"-?\d*\.?\d+").unwrap());
static WORD_BOUNDARY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\w+\b").unwrap());
static ENGLISH_WORDS: OnceCell<Arc<HashSet<String>>> = OnceCell::const_new();

pub async fn get_english_words() -> Arc<HashSet<String>> {
    ENGLISH_WORDS.get_or_init(load_english_words).await.clone()
}

async fn load_english_words() -> Arc<HashSet<String>> {
    match load_from_cache().await {
        Ok(words) => Arc::new(words),
        Err(_) => {
            let words = download_words().await.unwrap_or_default();
            let _ = save_to_cache(&words).await;
            Arc::new(words)
        }
    }
}

async fn load_from_cache() -> Result<HashSet<String>, std::io::Error> {
    let content = tokio::fs::read_to_string(CACHE_FILE).await?;
    Ok(content.lines().map(|s| s.to_lowercase()).collect())
}

async fn save_to_cache(words: &HashSet<String>) -> Result<(), std::io::Error> {
    let content = words
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(CACHE_FILE, content).await
}

async fn download_words() -> Result<HashSet<String>, reqwest::Error> {
    let response = reqwest::get(WORDS_URL).await?.text().await?;
    Ok(response.lines().map(|s| s.to_lowercase()).collect())
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HallucinationScore {
    pub proper_noun_score: f64,
    pub unknown_word_score: f64,
    pub number_mismatch_score: f64,
    pub total_score: f64,
    pub detected_hallucinations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub proper_noun_weight: f64,
    pub unknown_word_weight: f64,
    pub number_mismatch_weight: f64,
}

#[derive(Debug, Clone)]
pub struct HallucinationOptions {
    pub weights: ScoreWeights,
    pub use_ner: bool,
}

impl Default for HallucinationOptions {
    fn default() -> Self {
        Self {
            weights: ScoreWeights {
                proper_noun_weight: 0.4,
                unknown_word_weight: 0.1,
                number_mismatch_weight: 0.5,
            },
            use_ner: cfg!(feature = "ner"),
        }
    }
}

#[derive(Debug)]
pub struct TextAnalysis {
    proper_nouns: HashSet<String>,
    unknown_words: HashSet<String>,
    numbers: Vec<f64>,
}

#[cfg(feature = "ner")]
type Message = (Vec<String>, oneshot::Sender<Vec<Vec<Entity>>>);

#[cfg(feature = "ner")]
#[derive(Debug, Clone)]
pub struct EntityRecognizer {
    sender: mpsc::SyncSender<Message>,
}

#[cfg(feature = "ner")]
impl EntityRecognizer {
    pub fn spawn(
        config: TokenClassificationConfig,
    ) -> (JoinHandle<Result<(), RustBertError>>, EntityRecognizer) {
        let (sender, receiver) = mpsc::sync_channel(100);
        let handle = tokio::task::spawn_blocking(move || Self::runner(receiver, config));
        (handle, EntityRecognizer { sender })
    }

    fn runner(
        receiver: mpsc::Receiver<Message>,
        config: TokenClassificationConfig,
    ) -> Result<(), RustBertError> {
        let model = NERModel::new(config)?;
        while let Ok((texts, sender)) = receiver.recv() {
            let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
            let sentiments = model.predict(&texts);
            sender.send(sentiments).expect("sending results");
        }
        Ok(())
    }

    pub async fn predict(&self, texts: Vec<String>) -> Result<Vec<Vec<Entity>>, Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();
        self.sender.send((texts, sender))?;
        Ok(receiver.await?)
    }
}

#[derive(Debug, Clone)]
pub struct HallucinationDetector {
    #[cfg(feature = "ner")]
    ner_model: Option<EntityRecognizer>,
    options: HallucinationOptions,
}

impl HallucinationDetector {
    pub fn new(options: HallucinationOptions) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "onnx")]
        #[cfg(feature = "ner")]
        let config = TokenClassificationConfig::new(
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
        );

        #[cfg(not(feature = "onnx"))]
        #[cfg(feature = "ner")]
        let config = TokenClassificationConfig::default();

        #[cfg(feature = "ner")]
        let ner_model = if options.use_ner {
            Some(EntityRecognizer::spawn(config).1)
        } else {
            None
        };

        Ok(Self {
            #[cfg(feature = "ner")]
            ner_model,
            options,
        })
    }

    pub async fn detect_hallucinations(
        &self,
        llm_output: &String,
        references: &[String],
    ) -> HallucinationScore {
        let mut all_texts = vec![llm_output.to_string()];
        all_texts.extend(references.iter().cloned());

        let all_analyses = self.analyze_text(&all_texts).await;

        let (output_analysis, ref_analyses) = all_analyses.split_first().unwrap();

        let all_ref_proper_nouns: HashSet<_> = ref_analyses
            .iter()
            .flat_map(|analysis| analysis.proper_nouns.iter().cloned())
            .collect();

        let all_ref_numbers: Vec<_> = ref_analyses
            .iter()
            .flat_map(|analysis| analysis.numbers.iter().cloned())
            .collect();

        let all_ref_unknown_words: HashSet<_> = ref_analyses
            .iter()
            .flat_map(|analysis| analysis.unknown_words.iter().cloned())
            .collect();

        let proper_noun_diff: Vec<_> = output_analysis
            .proper_nouns
            .difference(&all_ref_proper_nouns)
            .cloned()
            .collect();

        let unknown_word_diff: Vec<_> = output_analysis
            .unknown_words
            .difference(&all_ref_unknown_words)
            .cloned()
            .collect();

        let number_diff = self.compare_numbers(&output_analysis.numbers, &all_ref_numbers);

        let proper_noun_score =
            proper_noun_diff.len() as f64 / output_analysis.proper_nouns.len().max(1) as f64;
        let unknown_word_score =
            unknown_word_diff.len() as f64 / output_analysis.unknown_words.len().max(1) as f64;
        let number_mismatch_score =
            number_diff.len() as f64 / output_analysis.numbers.len().max(1) as f64;

        let total_score = (proper_noun_score * self.options.weights.proper_noun_weight
            + unknown_word_score * self.options.weights.unknown_word_weight
            + number_mismatch_score * self.options.weights.number_mismatch_weight)
            .clamp(0.0, 1.0);

        HallucinationScore {
            proper_noun_score,
            unknown_word_score,
            number_mismatch_score,
            total_score,
            detected_hallucinations: [
                proper_noun_diff,
                unknown_word_diff,
                number_diff.iter().map(|n| n.to_string()).collect(),
            ]
            .concat(),
        }
    }

    #[allow(unused_variables)]
    async fn analyze_text(&self, texts: &[String]) -> Vec<TextAnalysis> {
        #[cfg(feature = "ner")]
        let entities = if let Some(ner_model) = &self.ner_model {
            ner_model.predict(texts.to_vec()).await.unwrap()
        } else {
            vec![Vec::new(); texts.len()]
        };

        #[cfg(not(feature = "ner"))]
        let entities: Vec<Vec<String>> = vec![Vec::new(); texts.len()];

        let english_words = get_english_words().await;

        texts
            .iter()
            .zip(entities.iter())
            .map(|(text, entities)| {
                let mut unknown_words = HashSet::new();
                let numbers: Vec<f64> = NUMBER_REGEX
                    .find_iter(text)
                    .filter_map(|m| m.as_str().parse::<f64>().ok())
                    .collect();

                let proper_nouns: HashSet<String> = if self.options.use_ner {
                    #[cfg(feature = "ner")]
                    {
                        entities
                            .iter()
                            .filter(|entity| {
                                !["O", "B-MIS", "I-MIS"].contains(&entity.label.as_str())
                            })
                            .map(|entity| entity.word.to_lowercase())
                            .collect()
                    }
                    #[cfg(not(feature = "ner"))]
                    {
                        HashSet::new()
                    }
                } else {
                    HashSet::new()
                };

                let mut word_map = HashMap::new();

                for cap in WORD_BOUNDARY_REGEX.find_iter(text) {
                    let word = cap.as_str();
                    let word_lower = word.to_lowercase();

                    word_map.entry(word_lower.clone()).or_insert_with(|| {
                        if !proper_nouns.contains(&word_lower)
                            && !english_words.contains(&word_lower)
                        {
                            unknown_words.insert(word.to_string());
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
                DETECTOR = Some(
                    HallucinationDetector::new(Default::default())
                        .expect("Failed to create detector"),
                );
            });
            DETECTOR.as_ref().unwrap()
        }
    }

    #[tokio::test]
    async fn test_zero_hallucination() {
        let detector = get_detector();
        let llm_output = String::from("Elon Musk is the CEO of Tesla.");
        let references = vec![String::from("Elon Musk is the CEO of Tesla.")];

        let score = detector
            .detect_hallucinations(&llm_output, &references)
            .await;
        println!("Zero Hallucination Score: {:?}", score);

        assert_eq!(score.proper_noun_score, 0.0);
        assert_eq!(score.number_mismatch_score, 0.0);
        assert!(score.total_score < 1e-2);
        assert!(score.detected_hallucinations.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_references() {
        let detector = get_detector();
        let llm_output =
            String::from("Apple and Microsoft are tech companies worth 3 trillion dollars.");
        let references = vec![
            String::from("Apple's market cap reached 3 trillion dollars."),
            String::from("Microsoft is a leading tech company."),
        ];

        let score = detector
            .detect_hallucinations(&llm_output, &references)
            .await;
        println!("Multiple References Score: {:?}", score);
        assert_eq!(score.proper_noun_score, 0.0); // Both companies are in references
        assert_eq!(score.number_mismatch_score, 0.0); // Number matches reference
    }

    #[tokio::test]
    async fn test_edge_cases() {
        let detector = get_detector();

        // Empty input
        let score_empty = detector
            .detect_hallucinations(&String::from(""), &[String::from("")])
            .await;
        assert_eq!(score_empty.total_score, 0.0);

        // Only numbers
        let score_numbers = detector
            .detect_hallucinations(&String::from("123 456.789"), &[String::from("123 456.789")])
            .await;
        assert_eq!(score_numbers.number_mismatch_score, 0.0);

        // Only proper nouns
        let score_nouns = detector
            .detect_hallucinations(
                &String::from("John Smith"),
                &[String::from("Different Person")],
            )
            .await;
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
    #[tokio::test]
    async fn test_hallucination_detection(
        #[case] llm_output: &str,
        #[case] references: Vec<&str>,
        #[case] expected: ExpectedScores,
        #[case] test_name: &str,
    ) {
        let detector = get_detector();
        let score = detector
            .detect_hallucinations(
                &String::from(llm_output),
                &references.into_iter().map(String::from).collect::<Vec<_>>(),
            )
            .await;

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
