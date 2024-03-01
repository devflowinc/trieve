use regex::Regex;
use regex_split::RegexSplit;
use scraper::{Html, Selector};
use std::{cmp, collections::HashMap, sync::Arc};

pub fn create_english_dict() -> HashMap<String, bool> {
    let mut english_dict = HashMap::new();
    let mut english_words_path = "".to_string();
    for entry in
        std::fs::read_dir(std::env::current_dir().expect("Current dir should be locatable"))
            .expect("Must be able to read current dir")
    {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.ends_with("english_words.txt") {
                    english_words_path = path.to_str().unwrap().to_string();
                    break;
                }
            }
            Err(e) => {
                log::error!("Error reading directory entry: {:?}", e);
            }
        }
    }

    let words = std::fs::read_to_string(english_words_path).expect("Could not read file");
    for word in words.lines() {
        english_dict.insert(word.to_string(), true);
    }

    english_dict
}

pub fn convert_html_to_text(html: &str) -> String {
    let dom = Html::parse_fragment(html);
    let text = dom.root_element().text().collect::<String>();
    text
}

pub fn coarse_remove_large_chunks(cur_chunks: Vec<String>) -> Vec<String> {
    let max_chunk_len = 10000;
    let mut chunks = cur_chunks;
    let mut new_chunks: Vec<String> = vec![];
    for chunk in chunks.iter_mut() {
        let char_count = chunk.chars().count() as f32;

        if char_count < (max_chunk_len as f32) {
            new_chunks.push(chunk.to_string());
            continue;
        }

        let num_new_chunks = (char_count / max_chunk_len as f32).ceil() as usize;
        let chunk_size = (char_count / num_new_chunks as f32).ceil();
        let mut total_length = char_count;

        while total_length > 0.0 {
            let amt_to_remove = cmp::min(chunk_size as usize, total_length as usize);
            let mut amt_to_take = amt_to_remove;
            while !chunk.is_char_boundary(amt_to_take) {
                amt_to_take -= 1;
            }

            let new_chunk = chunk.chars().take(amt_to_take).collect::<String>();
            new_chunks.push(new_chunk);
            chunk.drain(0..amt_to_take as usize);
            total_length -= amt_to_remove as f32;
        }
    }

    new_chunks.retain(|x| !x.is_empty());
    new_chunks
}

pub fn coarse_doc_chunker(document: String) -> Vec<String> {
    let document_without_newlines = document.replace('\n', " ");
    let dom = Html::parse_fragment(&document_without_newlines);

    // get the raw text from the HTML
    let clean_text = dom.root_element().text().collect::<String>();

    // split the text into sentences
    let split_sentence_regex = Regex::new(r"[.!?\n]+").expect("Invalid regex");
    let mut sentences: Vec<&str> = split_sentence_regex
        .split_inclusive_left(&clean_text)
        .collect();

    let mut groups: Vec<String> = vec![];
    let target_group_size = 30;

    if sentences.len() < target_group_size {
        groups.push(sentences.join(""));
        return coarse_remove_large_chunks(groups);
    }

    let mut remainder = (sentences.len() % target_group_size) as f32;
    let group_count = ((sentences.len() / target_group_size) as f32).floor();
    let remainder_per_group = (remainder / group_count).ceil();

    while remainder > 0.0 {
        let group_size =
            target_group_size + cmp::min(remainder as usize, remainder_per_group as usize) as usize;
        let group = sentences
            .iter()
            .take(group_size)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        sentences.drain(0..group_size);
        remainder -= remainder_per_group;
    }

    while !sentences.is_empty() {
        let group = sentences
            .iter()
            .take(target_group_size)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        sentences.drain(0..target_group_size);
    }

    coarse_remove_large_chunks(groups)
}

pub fn get_sentences(text: &String) -> Vec<String> {
    let split_sentence_regex = Regex::new(r"[.!?]+").expect("Invalid regex");
    let sentences: Vec<String> = split_sentence_regex
        .split(&text)
        .map(|x| x.to_string())
        .collect();
    sentences
}

pub fn get_words(text: &String) -> Vec<String> {
    let split_word_regex = Regex::new(r"\s+").expect("Invalid regex");
    let words: Vec<String> = split_word_regex
        .split(&text)
        .map(|x| x.to_string())
        .collect();
    words
}

pub fn percentage_english_words(text: &String, english_dict: Arc<HashMap<String, bool>>) -> f32 {
    let words = get_words(text);
    let mut english_word_count = 0;
    for word in words.iter() {
        if english_dict.contains_key(word) {
            english_word_count += 1;
        }
    }
    let percentage = (english_word_count as f32) / (words.len() as f32);
    percentage
}

pub fn loop_split_single_sentence(sentences: Vec<String>, word_limit: usize) -> Vec<String> {
    let first_sentence = sentences.first().unwrap_or(&"".to_string()).to_string();
    let mut max_single_sentence_word_count = get_words(&first_sentence).len();
    let first_sentence_word_count = max_single_sentence_word_count;
    let mut word_split_factor = 1;
    let mut new_sentences = sentences.clone();
    while max_single_sentence_word_count > word_limit {
        word_split_factor += 1;
        let mut words = get_words(&first_sentence);
        let new_word_size =
            ((first_sentence_word_count / word_split_factor) as f32).floor() as usize;
        let mut remainder = max_single_sentence_word_count % word_split_factor;
        let mut word_lengths = vec![new_word_size; word_split_factor];
        while remainder > 0 {
            word_lengths[remainder - 1] += 1;
            remainder -= 1;
        }

        new_sentences = vec![];
        for word_length in word_lengths.iter() {
            let new_sentence = words
                .iter()
                .take(*word_length)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            new_sentences.push(new_sentence);
            words.drain(0..*word_length);
        }

        max_single_sentence_word_count = 0;
        for new_sentence in new_sentences.iter() {
            if get_words(&new_sentence).len() > max_single_sentence_word_count {
                max_single_sentence_word_count = get_words(&new_sentence).len();
            }
        }
    }

    new_sentences
}

#[derive(Debug, Clone)]
pub struct ParsedChunk {
    pub heading: String,
    pub content_items: Vec<String>,
}

impl ParsedChunk {
    pub fn new(heading: String, content_items: Vec<String>) -> ParsedChunk {
        ParsedChunk {
            heading,
            content_items,
        }
    }

    pub fn output(&self, english_dict: Arc<HashMap<String, bool>>) -> Vec<String> {
        let mut first_content_item = self
            .content_items
            .first()
            .unwrap_or(&"".to_string())
            .to_string();

        let recurring_space_regex = Regex::new(r"[\s]+").expect("Invalid regex");
        let recurring_tab_regex = Regex::new(r"[\t]+").expect("Invalid regex");
        let starting_newline_regex = Regex::new(r"^\n").expect("Invalid regex");
        let ending_newline_regex = Regex::new(r"\n$").expect("Invalid regex");

        first_content_item = recurring_space_regex
            .replace_all(&first_content_item, " ")
            .to_string();
        first_content_item = recurring_tab_regex
            .replace_all(&first_content_item, " ")
            .to_string();
        first_content_item = starting_newline_regex
            .replace_all(&first_content_item, "")
            .to_string();
        first_content_item = ending_newline_regex
            .replace_all(&first_content_item, "")
            .to_string();

        if get_words(&first_content_item).len() < 20 {
            return vec![];
        }

        let total_content = format!("{}{}", self.heading, first_content_item);

        let heading_word_count = get_words(&self.heading).len();
        let mut largest_content_item_word_count = get_words(&first_content_item).len();

        let mut split_factor = 1;
        let mut new_p_bodies = vec![first_content_item.clone()];

        let word_limit = 340;

        while (heading_word_count + largest_content_item_word_count) > word_limit {
            split_factor += 1;
            let mut sentences = get_sentences(&total_content);
            let new_html_size = ((sentences.len() / split_factor) as f32).floor() as usize;
            let mut remainder = sentences.len() % split_factor;
            let mut lengths = vec![new_html_size; split_factor];
            while remainder > 0 {
                lengths[remainder - 1] += 1;
                remainder -= 1;
            }
            lengths = lengths
                .iter()
                .filter_map(|x| if *x > 0 { Some(*x) } else { None })
                .collect();

            new_p_bodies = vec![];
            for length in lengths.iter() {
                let temp_sentences = sentences
                    .iter()
                    .take(*length)
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                let mut new_sentences = temp_sentences.clone();
                if length.clone() == (1 as usize) {
                    new_sentences = loop_split_single_sentence(
                        temp_sentences.clone(),
                        word_limit - heading_word_count,
                    );
                }

                for new_sentence in new_sentences.iter() {
                    new_p_bodies.push(new_sentence.to_string());
                }

                sentences.drain(0..*length);
            }

            largest_content_item_word_count = 0;
            for body in new_p_bodies.iter() {
                if get_words(&body).len() > largest_content_item_word_count {
                    largest_content_item_word_count = get_words(&body).len();
                }
            }
        }

        let mut html_chunks: Vec<String> = vec![];
        for body in new_p_bodies.iter() {
            let words = get_words(&body);
            let unique_english_words = words
                .iter()
                .filter_map(|x| {
                    if english_dict.contains_key(x) {
                        Some(x.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            let count_unique_english_words = unique_english_words.len();
            let english_percentage = percentage_english_words(body, english_dict.clone());
            if count_unique_english_words < 10 && english_percentage < 0.75 {
                continue;
            }
            if count_unique_english_words < 30 && english_percentage < 0.10 {
                continue;
            }

            let mut cur_html = "<div>".to_string();
            if self.heading.len() > 0 {
                cur_html.push_str(&format!("<h3>{}</h3>", self.heading));
            }
            cur_html.push_str(&format!("<p>{}</p>", body));
            cur_html.push_str("</div>");
            html_chunks.push(cur_html);
        }

        html_chunks
    }
}

pub fn chunk_html(html_content: &str, english_dict: Arc<HashMap<String, bool>>) -> Vec<String> {
    let mut cur_heading = "".to_string();
    let mut chunks: Vec<ParsedChunk> = vec![];

    let dom = Html::parse_fragment(html_content);
    for node in dom
        .select(&Selector::parse("*").expect("valid selector"))
        .into_iter()
    {
        let text = node.text().collect::<String>();
        let words = get_words(&text);
        let tag_name = node.value().name();

        if words.len() == 0 {
            continue;
        }

        if vec!["h1", "h2", "h3", "h4", "h5", "h6"].contains(&tag_name) {
            cur_heading = text;
        } else if vec!["ul", "ol"].contains(&tag_name) {
            chunks.push(ParsedChunk::new(cur_heading.clone(), vec![text]));
        } else if vec!["p", "div"].contains(&tag_name) {
            let sub_children = node
                .select(&Selector::parse("b, i, em, strong").expect("valid selector"))
                .into_iter()
                .filter_map(|x| {
                    let text = x.text().collect::<String>();
                    if text != " " && text != "\n" {
                        Some(text)
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            if sub_children.len() == 1 {
                cur_heading = sub_children
                    .first()
                    .expect("must exist at this point")
                    .clone();
                continue;
            }

            chunks.push(ParsedChunk::new(cur_heading.clone(), vec![text]));
            cur_heading = "".to_string();
        }
    }

    let ret_chunks: Vec<String> = chunks
        .into_iter()
        .map(|x| x.output(english_dict.clone()))
        .flatten()
        .collect();
    log::info!("Returning {} chunks", ret_chunks.len());
    ret_chunks
}
