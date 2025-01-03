use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    data::models::{RedisPool, TypoOptions, TypoRange},
    errors::ServiceError,
    operators::search_operator::ParsedQuery,
};
use actix_web::web;
use dashmap::DashMap;
use flate2::{
    write::{GzDecoder, GzEncoder},
    Compression,
};
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    word: String,
    count: i32,
    children: Vec<(isize, Node)>,
}

/// A BK-tree datastructure
///
#[derive(Clone, Debug)]
pub struct BkTree {
    root: Option<Box<Node>>,
}

#[derive(Serialize, Deserialize)]
struct FlatNode {
    parent_index: Option<usize>,
    distance: Option<isize>,
    word: String,
    count: i32,
}

impl Serialize for BkTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut queue = VecDeque::new();
        let mut flat_tree = Vec::new();

        if let Some(root) = &self.root {
            queue.push_back((None, None, root.as_ref()));
        }

        while let Some((parent_index, distance, node)) = queue.pop_front() {
            let current_index = flat_tree.len();
            flat_tree.push(FlatNode {
                parent_index,
                distance,
                word: node.word.clone(),
                count: node.count,
            });

            for (child_distance, child) in &node.children {
                queue.push_back((Some(current_index), Some(*child_distance), child));
            }
        }

        let binary_data = bincode::serialize(&flat_tree).map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&binary_data)
    }
}

impl<'de> Deserialize<'de> for BkTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let binary_data: Vec<u8> = Vec::deserialize(deserializer)?;
        let flat_tree: Vec<FlatNode> =
            bincode::deserialize(&binary_data).map_err(serde::de::Error::custom)?;

        if flat_tree.is_empty() {
            return Ok(BkTree { root: None });
        }

        let mut nodes: Vec<Node> = flat_tree
            .iter()
            .map(|flat_node| Node {
                word: flat_node.word.clone(),
                count: flat_node.count,
                children: Vec::new(),
            })
            .collect();

        // Reconstruct the tree structure
        for i in (1..nodes.len()).rev() {
            let parent_index = flat_tree[i].parent_index.unwrap();
            let distance = flat_tree[i].distance.unwrap();
            let child = nodes.remove(i);
            nodes[parent_index].children.push((distance, child));
        }

        Ok(BkTree {
            root: Some(Box::new(nodes.remove(0))),
        })
    }
}

impl Default for BkTree {
    fn default() -> Self {
        Self::new()
    }
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    // Early exit for empty strings
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // Ensure s1 is the shorter string for optimization
    if len1 > len2 {
        return levenshtein_distance(s2, s1);
    }

    // Convert strings to byte slices for faster access
    let s1 = s1.as_bytes();
    let s2 = s2.as_bytes();

    // Use only one row of the matrix, reducing space complexity
    let mut prev_row = Vec::with_capacity(len2 + 1);
    let mut curr_row = vec![0; len2 + 1];

    // Initialize the first row
    for i in 0..=len2 {
        prev_row.push(i);
    }

    for (i, &ch1) in s1.iter().enumerate() {
        curr_row[0] = i + 1;

        for (j, &ch2) in s2.iter().enumerate() {
            let substitute_cost = if ch1 == ch2 { 0 } else { 1 };
            curr_row[j + 1] = min(
                curr_row[j] + 1, // Insertion
                min(
                    prev_row[j + 1] + 1,           // Deletion
                    prev_row[j] + substitute_cost, // Substitution
                ),
            );
        }

        // Swap rows
        std::mem::swap(&mut curr_row, &mut prev_row);
    }

    prev_row[len2]
}

impl BkTree {
    /// Create a new BK-tree
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Insert every element from a given iterator in the BK-tree
    pub fn insert_all<I: IntoIterator<Item = (String, i32)>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }

    /// Insert a new element in the BK-tree
    pub fn insert(&mut self, val: (String, i32)) {
        match self.root {
            None => {
                self.root = Some(Box::new(Node {
                    word: val.0,
                    count: val.1,
                    children: Vec::new(),
                }))
            }
            Some(ref mut root_node) => {
                let mut u = &mut **root_node;
                loop {
                    let k = levenshtein_distance(&u.word, &val.0) as isize;
                    if k == 0 {
                        u.count = val.1;
                        return;
                    }

                    if val.1 == 1 {
                        return;
                    }

                    let v = u.children.iter().position(|(dist, _)| *dist == k);
                    match v {
                        None => {
                            u.children.push((
                                k,
                                Node {
                                    word: val.0,
                                    count: val.1,
                                    children: Vec::new(),
                                },
                            ));
                            return;
                        }
                        Some(pos) => {
                            let (_, ref mut vnode) = u.children[pos];
                            u = vnode;
                        }
                    }
                }
            }
        }
    }

    /// Find the closest elements to a given value present in the BK-tree
    ///
    /// Returns pairs of element references and distances
    pub fn find(&self, val: String, max_dist: isize) -> Vec<((&String, &i32), isize)> {
        match self.root {
            None => Vec::new(),
            Some(ref root) => {
                let found = Arc::new(Mutex::new(Vec::new()));
                let mut candidates: Vec<&Node> = vec![root];

                while !candidates.is_empty() {
                    let next_candidates: Vec<&Node> = if candidates.len() > 1000 {
                        candidates
                            .par_iter()
                            .flat_map(|&n| {
                                let distance = levenshtein_distance(&n.word, &val) as isize;
                                let mut local_candidates = Vec::new();

                                if distance <= max_dist {
                                    found.lock().unwrap().push(((&n.word, &n.count), distance));
                                }

                                for (arc, node) in &n.children {
                                    if (*arc - distance).abs() <= max_dist {
                                        local_candidates.push(node);
                                    }
                                }

                                local_candidates
                            })
                            .collect()
                    } else {
                        candidates
                            .iter()
                            .flat_map(|&n| {
                                let distance = levenshtein_distance(&n.word, &val) as isize;
                                if distance <= max_dist {
                                    found.lock().unwrap().push(((&n.word, &n.count), distance));
                                }
                                n.children
                                    .iter()
                                    .filter(|(arc, _)| (*arc - distance).abs() <= max_dist)
                                    .map(|(_, node)| node)
                                    .collect::<Vec<_>>()
                            })
                            .collect()
                    };

                    candidates = next_candidates;
                }

                let mut result = Arc::try_unwrap(found).unwrap().into_inner().unwrap();
                result.sort_by_key(|&(_, dist)| dist);
                result
            }
        }
    }

    /// Create an iterator over references of BK-tree elements, in no particular order
    pub fn iter(&self) -> Iter {
        let mut queue = Vec::new();
        if let Some(ref root) = self.root {
            queue.push(&**root);
        }
        Iter { queue }
    }

    pub async fn from_redis(
        dataset_id: uuid::Uuid,
        redis_pool: web::Data<RedisPool>,
    ) -> Result<Option<Self>, ServiceError> {
        let mut redis_conn = redis_pool.get().await.map_err(|_| {
            ServiceError::InternalServerError("Failed to get redis connection".to_string())
        })?;

        let compressed_bk_tree: Option<Vec<u8>> = redis::cmd("GET")
            .arg(format!("bk_tree_{}", dataset_id))
            .query_async(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        if let Some(compressed_bk_tree) = compressed_bk_tree {
            let buf = Vec::new();
            let mut decoder = GzDecoder::new(buf);
            decoder.write_all(&compressed_bk_tree).map_err(|err| {
                ServiceError::InternalServerError(format!("Failed to decompress bk tree {}", err))
            })?;

            let serialized_bk_tree = decoder.finish().map_err(|err| {
                ServiceError::InternalServerError(format!(
                    "Failed to finish decompressing bk tree {}",
                    err
                ))
            })?;

            let tree = bincode::deserialize(&serialized_bk_tree).map_err(|err| {
                ServiceError::InternalServerError(format!("Failed to deserialize bk tree {}", err))
            })?;

            Ok(Some(tree))
        } else {
            Ok(None)
        }
    }

    pub async fn save(
        &self,
        dataset_id: uuid::Uuid,
        redis_pool: web::Data<RedisPool>,
    ) -> Result<(), ServiceError> {
        if self.root.is_none() {
            return Ok(());
        }
        let mut redis_conn = redis_pool.get().await.map_err(|_| {
            ServiceError::InternalServerError("Failed to get redis connection".to_string())
        })?;

        let uncompressed_bk_tree = bincode::serialize(self).map_err(|_| {
            ServiceError::InternalServerError("Failed to serialize bk tree".to_string())
        })?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed_bk_tree).map_err(|_| {
            ServiceError::InternalServerError("Failed to compress bk tree".to_string())
        })?;

        let serialized_bk_tree = encoder.finish().map_err(|_| {
            ServiceError::InternalServerError("Failed to finish compressing bk tree".to_string())
        })?;

        redis::cmd("SET")
            .arg(format!("bk_tree_{}", dataset_id))
            .arg(serialized_bk_tree)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        Ok(())
    }
}

/// Iterator over BK-tree elements
#[allow(dead_code)]
pub struct IntoIter {
    queue: Vec<Node>,
}

impl Iterator for IntoIter {
    type Item = (String, i32);
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|node| {
            self.queue.extend(node.children.into_iter().map(|(_, n)| n));
            (node.word, node.count)
        })
    }
}

/// Iterator over BK-tree elements, by reference
pub struct Iter<'a> {
    queue: Vec<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a i32);
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|node| {
            self.queue.extend(node.children.iter().map(|(_, n)| n));
            (&node.word, &node.count)
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessWordsFromDatasetMessage {
    pub chunks_to_process: Vec<(uuid::Uuid, uuid::Uuid)>, // chunk_id, dataset_id
    pub attempt_number: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateBkTreeMessage {
    pub dataset_id: uuid::Uuid,
    pub attempt_number: usize,
}

struct BKTreeCacheEntry {
    bktree: Arc<BkTree>,
    expiration: Instant,
}

pub struct BKTreeCache {
    cache: DashMap<uuid::Uuid, BKTreeCacheEntry>,
}

lazy_static! {
    static ref BKTREE_CACHE: BKTreeCache = BKTreeCache::new();
    static ref ENGLISH_WORDS: HashSet<String> = {
        include_str!("../words.txt")
            .lines()
            .map(|s| s.to_lowercase())
            .collect()
    };
    static ref PREFIX_TRIE: Trie = {
        let prefixes = vec![
            "anti", "auto", "de", "dis", "down", "extra", "hyper", "il", "im", "in", "ir", "inter",
            "mega", "mid", "mis", "non", "over", "out", "post", "pre", "pro", "re", "semi", "sub",
            "super", "tele", "trans", "ultra", "un", "under", "up",
        ];
        Trie::new(&prefixes, TrieSearchDirection::Prefix)
    };
    static ref SUFFIX_TRIE: Trie = {
        let suffixes = vec![
            "able", "al", "ance", "ation", "ative", "ed", "en", "ence", "ent", "er", "es", "est",
            "ful", "ian", "ible", "ic", "ing", "ion", "ious", "ise", "ish", "ism", "ist", "ity",
            "ive", "ize", "less", "ly", "ment", "ness", "or", "ous", "s", "sion", "tion", "ty",
            "y",
        ];
        Trie::new(&suffixes, TrieSearchDirection::Suffix)
    };
}

struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
}

struct Trie {
    root: TrieNode,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            is_end: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum TrieSearchDirection {
    Prefix,
    Suffix,
}

impl Trie {
    fn new(elements: &[&str], search_direction: TrieSearchDirection) -> Self {
        let mut trie = Trie {
            root: TrieNode::new(),
        };
        for &element in elements {
            trie.insert(element, search_direction);
        }
        trie
    }

    fn insert(&mut self, element: &str, search_direction: TrieSearchDirection) {
        let mut node = &mut self.root;
        match search_direction {
            TrieSearchDirection::Prefix => {
                for ch in element.chars() {
                    node = node.children.entry(ch).or_insert(TrieNode::new());
                }
            }
            TrieSearchDirection::Suffix => {
                for ch in element.chars().rev() {
                    node = node.children.entry(ch).or_insert(TrieNode::new());
                }
            }
        };

        node.is_end = true;
    }

    fn longest_prefix(&self, word: &str) -> Option<usize> {
        let mut node = &self.root;
        let mut last_match = None;

        for (i, ch) in word.chars().enumerate() {
            if let Some(next_node) = node.children.get(&ch) {
                node = next_node;
                if node.is_end {
                    last_match = Some(i + 1);
                }
            } else {
                break;
            }
        }

        last_match
    }

    fn longest_suffix(&self, word: &str) -> Option<usize> {
        let mut node = &self.root;
        let mut last_match = None;

        for (i, ch) in word.chars().rev().enumerate() {
            if let Some(next_node) = node.children.get(&ch) {
                node = next_node;
                if node.is_end {
                    last_match = Some(i + 1);
                }
            } else {
                break;
            }
        }

        last_match
    }
}

impl BKTreeCache {
    fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    fn insert_with_ttl(&self, id: uuid::Uuid, bktree: BkTree, ttl: Duration) {
        let entry = BKTreeCacheEntry {
            bktree: Arc::new(bktree),
            expiration: Instant::now() + ttl,
        };
        self.cache.insert(id, entry);
    }

    fn get_if_valid(&self, id: &uuid::Uuid) -> Option<Arc<BkTree>> {
        let result = self.cache.get(id).and_then(|entry| {
            if Instant::now() < entry.expiration {
                Some(Arc::clone(&entry.bktree))
            } else {
                None
            }
        });
        result
    }

    fn remove_expired(&self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| now < entry.expiration);
    }

    pub fn enforce_cache_ttl() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Run every 60 seconds

            loop {
                interval.tick().await;
                BKTREE_CACHE.remove_expired();
            }
        });
    }
}

fn correct_query_helper(
    tree: &BkTree,
    mut query: ParsedQuery,
    options: &TypoOptions,
) -> CorrectedQuery {
    let query_words: Vec<&str> = query.query.split_whitespace().collect();

    let mut corrections = HashMap::new();
    let mut new_quote_words = Vec::new();

    let excluded_words: HashSet<_> = options
        .disable_on_word
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let single_typo_range = options.one_typo_word_range.clone().unwrap_or(TypoRange {
        min: 4,
        max: Some(6),
    });

    let two_typo_range = options
        .two_typo_word_range
        .clone()
        .unwrap_or(TypoRange { min: 6, max: None });

    for &word in &query_words {
        if corrections.contains_key(word) {
            continue;
        }

        if excluded_words.contains(&word.to_lowercase()) {
            continue;
        }

        if word.contains(|c: char| !c.is_alphabetic()) {
            continue;
        }

        if is_likely_english_word(&word.to_lowercase()) {
            continue;
        }

        if !tree.find(word.to_lowercase(), 0).is_empty() {
            if options.prioritize_domain_specifc_words.unwrap_or(true) {
                new_quote_words.push(word);
                query.quote_words = match query.quote_words {
                    Some(mut existing_words) => {
                        existing_words.push(word.to_string());
                        Some(existing_words)
                    }
                    None => Some(vec![word.to_string()]),
                };
            }
            continue;
        }

        let num_chars = word.chars().count();
        let max_distance = if num_chars >= two_typo_range.min as usize
            && num_chars <= two_typo_range.max.unwrap_or(u32::MAX) as usize
        {
            2
        } else if num_chars >= single_typo_range.min as usize
            && num_chars <= single_typo_range.max.unwrap_or(u32::MAX) as usize
        {
            1
        } else {
            0
        };

        if max_distance > 0 {
            let mut best_correction = None;
            let mut best_score = 0;

            for ((correction, freq), distance) in tree.find(word.to_lowercase(), max_distance) {
                if distance == 0 {
                    best_correction = None;
                    break;
                }
                if !is_best_correction(word.to_lowercase(), correction.to_string()) {
                    continue;
                }

                let score = (max_distance - distance) * 1000 + *freq as isize;

                if score > best_score || best_correction.is_none() {
                    best_correction = Some(correction);
                    best_score = score;
                }
            }

            if let Some(correction) = best_correction {
                corrections.insert(word, correction.to_string());
            }
        }
    }

    if corrections.is_empty() && new_quote_words.is_empty() {
        CorrectedQuery {
            query: Some(query),
            corrected: false,
        }
    } else {
        let mut corrected_query = query.query.clone();

        for (original, correction) in corrections {
            corrected_query = corrected_query.replace(original, &correction);
        }

        for word in new_quote_words {
            corrected_query = corrected_query.replace(word, &format!("\"{}\"", word));
        }

        query.query = corrected_query;
        CorrectedQuery {
            query: Some(query),
            corrected: true,
        }
    }
}

fn is_best_correction(word: String, correction: String) -> bool {
    // Length-based filter
    let len_diff = (word.len() as i32 - correction.len() as i32).abs();
    if len_diff > 2 {
        return false;
    }

    // Prefix matching (adjust the length as needed)
    let prefix_len = std::cmp::min(1, std::cmp::min(word.len(), correction.len()));
    if word.chars().take(prefix_len).collect::<Vec<_>>()
        != correction.chars().take(prefix_len).collect::<Vec<_>>()
    {
        return false;
    }

    // Character set comparison
    let word_chars: HashSet<char> = word.chars().collect();
    let correction_chars: HashSet<char> = correction.chars().collect();
    let common_chars = word_chars.intersection(&correction_chars).count();
    let similarity_ratio =
        common_chars as f32 / word_chars.len().max(correction_chars.len()) as f32;

    similarity_ratio >= 0.8
}

fn is_likely_english_word(word: &str) -> bool {
    if ENGLISH_WORDS.contains(&word.to_lowercase()) {
        return true;
    }

    // Check for prefix
    if let Some(prefix_len) = PREFIX_TRIE.longest_prefix(word) {
        if ENGLISH_WORDS.contains(&word[prefix_len..].to_lowercase()) {
            return true;
        }
    }

    // Check for suffix
    if let Some(suffix_len) = SUFFIX_TRIE.longest_suffix(word) {
        if ENGLISH_WORDS.contains(&word[..word.len() - suffix_len].to_lowercase()) {
            return true;
        }
    }

    // Check for compound words
    if word.contains('-') {
        let parts: Vec<&str> = word.split('-').collect();
        if parts
            .iter()
            .all(|part| ENGLISH_WORDS.contains(&part.to_lowercase()))
        {
            return true;
        }
    }

    false
}

#[derive(Debug, Default)]
pub struct CorrectedQuery {
    pub query: Option<ParsedQuery>,
    pub corrected: bool,
}

pub async fn correct_query(
    query: ParsedQuery,
    dataset_id: uuid::Uuid,
    redis_pool: web::Data<RedisPool>,
    options: &TypoOptions,
) -> Result<CorrectedQuery, ServiceError> {
    if matches!(options.correct_typos, None | Some(false)) {
        return Ok(CorrectedQuery::default());
    }

    match BKTREE_CACHE.get_if_valid(&dataset_id) {
        Some(tree) => {
            let result = correct_query_helper(&tree, query, options);
            Ok(result)
        }
        None => {
            let redis_pool = redis_pool.clone();
            log::info!("Pulling new BK tree from Redis");
            let tree = match BkTree::from_redis(dataset_id, redis_pool).await {
                // TTL of 1 day
                Ok(Some(bktree)) => {
                    BKTREE_CACHE.insert_with_ttl(
                        dataset_id,
                        bktree,
                        Duration::from_secs(60 * 60 * 24),
                    );
                    log::info!(
                        "Inserted new BK tree into cache for dataset_id: {:?}",
                        dataset_id
                    );
                    BKTREE_CACHE.get_if_valid(&dataset_id)
                }
                Ok(None) => {
                    log::info!("No BK tree found in Redis for dataset_id: {:?}", dataset_id);
                    return Ok(CorrectedQuery::default());
                }
                Err(e) => {
                    log::info!(
                        "Failed to insert new BK tree into cache {:?} for dataset_id: {:?}",
                        e,
                        dataset_id
                    );
                    return Ok(CorrectedQuery::default());
                }
            };

            match tree {
                Some(tree) => {
                    let result = correct_query_helper(&tree, query, options);
                    Ok(result)
                }
                None => Ok(CorrectedQuery::default()),
            }
        }
    }
}
