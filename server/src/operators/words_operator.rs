use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    data::models::{RedisPool, TypoOptions, TypoRange},
    errors::ServiceError,
};
use actix_web::web;
use flate2::{
    write::{GzDecoder, GzEncoder},
    Compression,
};
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use tokio::sync::RwLock;

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

pub fn levenshtein_distance<S: AsRef<str>>(a: &S, b: &S) -> isize {
    let a = a.as_ref().to_lowercase();
    let b = b.as_ref().to_lowercase();

    if a == b {
        return 0;
    }

    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len as isize;
    }

    if b_len == 0 {
        return a_len as isize;
    }

    let mut res = 0;
    let mut cache: Vec<usize> = (1..).take(a_len).collect();
    let mut a_dist;
    let mut b_dist;

    for (ib, cb) in b.chars().enumerate() {
        res = ib;
        a_dist = ib;
        for (ia, ca) in a.chars().enumerate() {
            b_dist = if ca == cb { a_dist } else { a_dist + 1 };
            a_dist = cache[ia];

            res = if a_dist > res {
                if b_dist > res {
                    res + 1
                } else {
                    b_dist
                }
            } else if b_dist > a_dist {
                a_dist + 1
            } else {
                b_dist
            };

            cache[ia] = res;
        }
    }

    res as isize
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
                    let k = levenshtein_distance(&u.word, &val.0);
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
                                let distance = levenshtein_distance(&n.word, &val);
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
                                let distance = levenshtein_distance(&n.word, &val);
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
            .query_async(&mut *redis_conn)
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
    bktree: BkTree,
    expiration: Instant,
}

pub struct BKTreeCache {
    cache: RwLock<HashMap<uuid::Uuid, BKTreeCacheEntry>>,
}

lazy_static! {
    static ref BKTREE_CACHE: BKTreeCache = BKTreeCache::new();
}

impl BKTreeCache {
    fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn insert_with_ttl(&self, id: uuid::Uuid, bktree: BkTree, ttl: Duration) {
        if let Ok(mut cache) = self.cache.try_write() {
            let entry = BKTreeCacheEntry {
                bktree,
                expiration: Instant::now() + ttl,
            };
            cache.insert(id, entry);
        };
    }

    fn get_if_valid(&self, id: &uuid::Uuid) -> Option<BkTree> {
        match self.cache.try_read() {
            Ok(cache) => cache.get(id).and_then(|entry| {
                if Instant::now() < entry.expiration {
                    Some(entry.bktree.clone())
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn remove_expired(&self) {
        if let Ok(mut cache) = self.cache.try_write() {
            cache.retain(|_, entry| Instant::now() < entry.expiration);
        }
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

fn correct_query_helper(tree: &BkTree, query: String, options: &TypoOptions) -> Option<String> {
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let mut corrections = HashMap::new();
    let excluded_words: HashSet<_> = options
        .disable_on_word
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let single_typo_range = options.one_typo_word_range.clone().unwrap_or(TypoRange {
        min: 5,
        max: Some(8),
    });

    let two_typo_range = options
        .two_typo_word_range
        .clone()
        .unwrap_or(TypoRange { min: 8, max: None });

    for &word in &query_words {
        if excluded_words.contains(&word.to_lowercase()) {
            continue;
        }

        if !tree.find(word.to_string(), 0).is_empty() {
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

            for ((correction, freq), distance) in tree.find(word.to_string(), max_distance) {
                if distance == 0 {
                    best_correction = None;
                    break;
                }

                if !is_similar_enough(word, correction) {
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

    if corrections.is_empty() {
        None
    } else {
        let mut corrected_query = query.to_string();
        for (original, correction) in corrections {
            corrected_query = corrected_query.replacen(original, &correction, 1);
        }
        Some(corrected_query)
    }
}

fn is_similar_enough(word: &str, correction: &str) -> bool {
    // Length-based filter
    let len_diff = (word.len() as i32 - correction.len() as i32).abs();
    if len_diff > 2 {
        return false;
    }

    // Prefix matching (adjust the length as needed)
    let prefix_len = std::cmp::min(3, std::cmp::min(word.len(), correction.len()));
    if word[..prefix_len] != correction[..prefix_len] {
        return false;
    }

    // Character set comparison
    let word_chars: HashSet<char> = word.chars().collect();
    let correction_chars: HashSet<char> = correction.chars().collect();
    let common_chars = word_chars.intersection(&correction_chars).count();
    let similarity_ratio =
        common_chars as f32 / word_chars.len().max(correction_chars.len()) as f32;

    similarity_ratio >= 0.7
}

#[tracing::instrument(skip(redis_pool))]
pub async fn correct_query(
    query: String,
    dataset_id: uuid::Uuid,
    redis_pool: web::Data<RedisPool>,
    options: &TypoOptions,
) -> Result<Option<String>, ServiceError> {
    if matches!(options.correct_typos, None | Some(false)) {
        return Ok(None);
    }

    match BKTREE_CACHE.get_if_valid(&dataset_id) {
        Some(tree) => Ok(correct_query_helper(&tree, query, options)),
        None => {
            let dataset_id = dataset_id;
            let redis_pool = redis_pool.clone();
            log::info!("Pulling new BK tree from Redis");
            tokio::spawn(async move {
                match BkTree::from_redis(dataset_id, redis_pool).await {
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
                    }
                    Ok(None) => {
                        log::info!("No BK tree found in Redis for dataset_id: {:?}", dataset_id);
                    }
                    Err(e) => {
                        log::info!(
                            "Failed to insert new BK tree into cache {:?} for dataset_id: {:?}",
                            e,
                            dataset_id
                        );
                    }
                };
            });
            Ok(None)
        }
    }
}
