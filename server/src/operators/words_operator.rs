use std::collections::{HashMap, HashSet};

use crate::{
    data::models::{RedisPool, TypoOptions, TypoRange},
    errors::ServiceError,
};
use actix_web::web;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct Node {
    word: String,
    count: i32,
    children: Vec<(isize, Node)>,
}

/// A BK-tree datastructure
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BkTree {
    root: Option<Box<Node>>,
}

impl Default for BkTree {
    fn default() -> Self {
        Self::new()
    }
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
                    let k = bktree::levenshtein_distance(&u.word, &val.0);
                    if k == 0 {
                        u.count = val.1;
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
                let mut found = Vec::new();

                let mut candidates: std::collections::VecDeque<&Node> =
                    std::collections::VecDeque::new();
                candidates.push_back(root);

                while let Some(n) = candidates.pop_front() {
                    let distance = bktree::levenshtein_distance(&n.word, &val);
                    if distance <= max_dist {
                        found.push(((&n.word, &n.count), distance));
                    }

                    candidates.extend(
                        n.children
                            .iter()
                            .filter(|(arc, _)| (*arc - distance).abs() <= max_dist)
                            .map(|(_, node)| node),
                    );
                }
                found
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
}

/// Iterator over BK-tree elements
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

pub async fn get_bktree_from_redis_query(
    dataset_id: uuid::Uuid,
    redis_pool: web::Data<RedisPool>,
) -> Result<Option<BkTree>, ServiceError> {
    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    let serialized_bk_tree: Option<Vec<u8>> = redis::cmd("GET")
        .arg(format!("bk_tree_{}", dataset_id))
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    if let Some(serialized_bk_tree) = serialized_bk_tree {
        let tree: BkTree = rmp_serde::from_slice(&serialized_bk_tree)
            .map_err(|_| ServiceError::BadRequest("Failed to deserialize bk tree".to_string()))?;
        Ok(Some(tree))
    } else {
        Ok(None)
    }
}

lazy_static! {
    static ref bktree_cache: RwLock<HashMap<uuid::Uuid, BkTree>> = RwLock::new(HashMap::new());
}

fn correct_query_helper(tree: &BkTree, query: String, options: &TypoOptions) -> String {
    let query_split_by_whitespace = query
        .split_whitespace()
        .map(|s| s.to_string())
        .collect_vec();
    let mut query_split_to_correction: HashMap<String, String> = HashMap::new();
    let excluded_words = options
        .clone()
        .disable_on_word
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect::<HashSet<String>>();

    for split in &query_split_by_whitespace {
        if excluded_words.contains(&split.to_lowercase()) {
            continue;
        }

        let exact_match = tree.find(split.to_string(), 0);

        if !exact_match.is_empty() {
            continue;
        }

        let mut corrections = vec![];

        let num_chars = split.chars().collect_vec().len();

        let single_typo_range = options.clone().one_typo_word_range.unwrap_or(TypoRange {
            min: 5,
            max: Some(8),
        });

        if num_chars >= (single_typo_range.min as usize)
            && num_chars <= (single_typo_range.max.unwrap_or(u32::MAX) as usize)
        {
            corrections.extend_from_slice(&tree.find(split.to_string(), 1));
        }

        let two_typo_range = options
            .clone()
            .two_typo_word_range
            .unwrap_or(TypoRange { min: 8, max: None });

        if num_chars >= (two_typo_range.min as usize)
            && num_chars <= (two_typo_range.max.unwrap_or(u32::MAX) as usize)
        {
            corrections.extend_from_slice(&tree.find(split.to_string(), 2));
        }

        corrections.sort_by(|((_, freq_a), _), ((_, freq_b), _)| (**freq_b).cmp(*freq_a));

        if let Some(((correction, _), _)) = corrections.get(0) {
            query_split_to_correction.insert(split.to_string(), correction.to_string());
        }
    }

    dbg!(&query_split_to_correction);

    let mut corrected_query = query.clone();

    if !query_split_to_correction.is_empty() {
        for (og_string, correction) in query_split_to_correction {
            corrected_query = corrected_query.replacen(&og_string, &correction, 1);
        }
    }

    corrected_query
}

#[tracing::instrument(skip(redis_pool))]
pub async fn correct_query(
    query: String,
    dataset_id: uuid::Uuid,
    redis_pool: web::Data<RedisPool>,
    options: &TypoOptions,
) -> Result<String, ServiceError> {
    if matches!(options.correct_typos, None | Some(false)) {
        return Ok(query);
    }

    match bktree_cache.try_read() {
        Ok(in_mem_cache) => match in_mem_cache.get(&dataset_id) {
            Some(tree) => Ok(correct_query_helper(tree, query, options)),
            None => {
                drop(in_mem_cache);
                let dataset_id = dataset_id;
                let redis_pool = redis_pool.clone();
                tokio::spawn(async move {
                    if let Ok(mut in_mem_cache) = bktree_cache.try_write() {
                        if let Ok(Some(bktree)) =
                            get_bktree_from_redis_query(dataset_id, redis_pool).await
                        {
                            in_mem_cache.insert(dataset_id, bktree);
                        };
                    }
                });
                Ok(query)
            }
        },
        Err(_) => Ok(query),
    }
}
