use ndarray::Array2;
use regex::Regex;
use regex_split::RegexSplit;
use scraper::Html;
use std::cmp;

use crate::errors::ServiceError;

#[tracing::instrument]
pub fn convert_html_to_text(html: &str) -> String {
    let dom = Html::parse_fragment(html);
    let text = dom.root_element().text().collect::<String>();
    text
}

#[tracing::instrument]
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

pub fn build_chunking_regex(delimiters: Vec<String>) -> Result<Regex, regex::Error> {
    let escaped_delimiters: Vec<String> = delimiters.iter().map(|x| regex::escape(x)).collect();
    let pattern = escaped_delimiters.join("|");
    let re = Regex::new(&pattern)?;
    return Ok(re);
}

#[tracing::instrument]
pub fn coarse_doc_chunker(
    document: String,
    split_pattern: Option<Regex>,
    target_splits_per_chunk: usize,
) -> Vec<String> {
    let document_without_newlines = document.replace('\n', " ");
    let dom = Html::parse_fragment(&document_without_newlines);

    // get the raw text from the HTML
    let clean_text = dom.root_element().text().collect::<String>();

    let pattern = match split_pattern {
        Some(pattern) => pattern,
        None => Regex::new(r"[.!?\n]+").expect("Invalid regex"),
    };
    // log the pattern
    log::info!("Pattern: {:?}", pattern);

    // split the text into sentences
    let mut splits: Vec<&str> = pattern.split_inclusive(&clean_text).collect();
    // log the splits length
    log::info!("Splits: {:?}", splits.len());

    let mut groups: Vec<String> = vec![];

    if splits.len() < target_splits_per_chunk {
        groups.push(splits.join(""));
        return groups;
    }

    let mut remainder = (splits.len() % target_splits_per_chunk) as f32;
    let group_count = ((splits.len() / target_splits_per_chunk) as f32).floor();
    let remainder_per_group = (remainder / group_count).ceil();

    while remainder > 0.0 {
        let group_size = target_splits_per_chunk
            + cmp::min(remainder as usize, remainder_per_group as usize) as usize;
        let group = splits
            .iter()
            .take(group_size)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        splits.drain(0..group_size);
        remainder -= remainder_per_group;
    }

    while !splits.is_empty() {
        let group = splits
            .iter()
            .take(target_splits_per_chunk)
            .copied()
            .collect::<Vec<&str>>()
            .join("");
        groups.push(group);
        splits.drain(0..target_splits_per_chunk);
    }

    groups
}

#[tracing::instrument(skip(embeddings))]
pub fn average_embeddings(embeddings: Vec<Vec<f32>>) -> Result<Vec<f32>, ServiceError> {
    let first_embedding_len = match embeddings.first() {
        Some(embedding) => embedding.len(),
        None => {
            return Err(ServiceError::BadRequest(
                "No embeddings provided".to_string(),
            ));
        }
    };

    let shape = (embeddings.len(), first_embedding_len);
    let flat: Vec<f32> = embeddings.iter().flatten().cloned().collect();
    let arr: Array2<f32> = Array2::from_shape_vec(shape, flat).map_err(|e| {
        log::error!("Error creating ndarray from embeddings: {}", e);
        ServiceError::InternalServerError(
            "Error creating ndarray from embeddings to average".to_string(),
        )
    })?;

    Ok((arr.sum_axis(ndarray::Axis(0)) / (embeddings.len() as f32)).to_vec())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_average_embeddings() {
        let embeddings = vec![
            vec![3.0, 2.5, 1.0],
            vec![1.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
            vec![2.0, 2.5, 1.0],
        ];

        let result = average_embeddings(embeddings).unwrap();
        assert!(result == vec![2.0, 2.5, 1.0]);
    }
}
