use core::f32;
use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use clickhouse::Row;
use hdbscan::HdbscanHyperParams;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Row, Deserialize)]
struct DatasetQueryRow {
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: Uuid,
}

#[derive(Clone, Deserialize)]
pub struct SetupArgs {
    /// Clickhouse URL
    pub url: Option<String>,
    /// Clickhouse User
    pub user: Option<String>,
    /// Clickhouse Password
    pub password: Option<String>,
    /// Clickhouse Database
    pub database: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get environment variable or quit
    let args = SetupArgs {
        url: Some(std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string())),
        user: Some(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string())),
        password: Some(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("password".to_string())),
        database: Some(std::env::var("CLICKHOUSE_DB").unwrap_or("default".to_string())),
    };

    let clickhouse_client = clickhouse::Client::default()
        .with_url(args.url.as_ref().unwrap())
        .with_user(args.user.as_ref().unwrap())
        .with_password(args.password.as_ref().unwrap())
        .with_database(args.database.as_ref().unwrap());

    let datasets: Vec<Uuid> = clickhouse_client
        .query(
            "
        SELECT DISTINCT dataset_id
        FROM default.search_queries
        ",
        )
        .fetch_all::<DatasetQueryRow>()
        .await?
        .into_iter()
        .map(|f| f.dataset_id)
        .collect();

    // Print all the datasets
    println!("Datasets: {:?}", datasets);

    let fake_datasets = vec![Uuid::from_str("7ab6502e-ac37-435b-ae36-643c488e282d").unwrap()];

    let req_client = Arc::new(reqwest::Client::new());

    // Use tokio to parallelize the dataset processing
    let mut thread_handles = Vec::new();
    for dataset_id in fake_datasets.clone() {
        let client = clickhouse_client.clone();
        let req_client = req_client.clone();
        let dataset_id = dataset_id.clone();
        let thread = tokio::spawn(async move {
            handle_dataset(dataset_id, client, req_client)
                .await
                .unwrap();
        });
        thread_handles.push(thread);
    }
    for thread in thread_handles {
        thread.await.unwrap();
    }

    Ok(())
}

#[derive(Row, Deserialize, Clone, Debug)]
struct QueryRow {
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    query: String,
    top_score: f32,
    query_vector: Vec<f32>,
}

async fn fetch_dataset_queries(
    client: clickhouse::Client,
    dataset_id: Uuid,
    limit: Option<u64>,
) -> Result<Vec<QueryRow>> {
    let query = format!(
        "
        SELECT id, query, top_score, query_vector 
        FROM default.search_queries 
        WHERE dataset_id = '{}'
            AND created_at >= now() - INTERVAL 7 DAY AND is_duplicate = 0
        ORDER BY rand() 
        LIMIT {}
        ",
        dataset_id,
        limit.unwrap_or(5000)
    );
    let result = client.query(&query);
    let rows = result.fetch_all::<QueryRow>().await?;
    Ok(rows)
}

// Taken from the clusterer source code
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| ((*x) - (*y)) * ((*x) - (*y)))
        .fold(0.0, std::ops::Add::add)
        .sqrt()
}

#[derive(Debug)]
struct QueryWithDistance {
    query: QueryRow,
    distance: f32,
}

#[derive(Debug)]
pub struct Cluster {
    dataset_id: Uuid,
    pos: Vec<f32>,
    queries: Vec<QueryWithDistance>,
}
impl Cluster {
    fn from_dataset_id(dataset_id: Uuid) -> Self {
        Self {
            dataset_id,
            pos: Vec::new(),
            queries: Vec::new(),
        }
    }
}

fn hdbscan_clustering(data: Vec<QueryRow>, dataset_id: Uuid) -> Result<Vec<Cluster>> {
    let vectors: Vec<Vec<f32>> = data.iter().map(|row| row.query_vector.clone()).collect();

    let params = HdbscanHyperParams::builder().min_cluster_size(2).build();

    let clusterer = hdbscan::Hdbscan::new(&vectors, params);

    let labels = clusterer.cluster()?;

    let max_cluster_index = labels.iter().max().unwrap().to_owned() as usize;

    let mut clusters: Vec<Cluster> = Vec::new();
    for i in 0..max_cluster_index + 1 {
        clusters.push(Cluster::from_dataset_id(dataset_id.clone()));
    }

    let centriods = clusterer.calc_centers(hdbscan::Center::Centroid, &labels)?;

    for (i, label) in labels.iter().enumerate() {
        if *label < 0 {
            continue;
        }
        if let Some(data) = data.get(i) {
            let data_with_distance = QueryWithDistance {
                query: data.clone(),
                distance: euclidean_distance(
                    &data.query_vector,
                    &centriods.get(*label as usize).unwrap(),
                ),
            };
            clusters[*label as usize].queries.push(data_with_distance)
        }
        // Assign the centroid positions
        if clusters.get(*label as usize).unwrap().pos.len() == 0 {
            clusters[*label as usize].pos = centriods.get(*label as usize).unwrap().clone();
        }
    }
    Ok(clusters)
}

#[derive(Row, Deserialize)]
struct ClusterTopicRow {
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    dataset_id: Uuid,
    topic: String,
    density: u32,
    avg_score: f32,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Row, Deserialize)]
struct ClusterMembershipRow {
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    search_id: Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    cluster_id: Uuid,
    distance_to_centroid: f32,
}

struct Topic {
    topic: ClusterTopicRow,
    memberships: Vec<ClusterMembershipRow>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ClaudeRequestBody {
    model: String,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicReponseContent {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicReponseContent>,
    role: Option<String>,
}

async fn form_topics(mut cluster: Cluster, req_client: Arc<reqwest::Client>) -> Result<Topic> {
    // Sort by distance
    cluster.queries.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_5: Vec<&QueryWithDistance> = cluster.queries.iter().take(5).collect();

    let query_string = top_5
        .iter()
        .map(|query| query.query.query.clone())
        .collect::<Vec<String>>()
        .join(", ");

    println!("Top 5 queries: {:?}", query_string);

    let system_prompt = "You are a data scientist. You have been tasked with clustering search queries into topics. You have just finished clustering a set of queries into a group. You have been asked to generate a 3-5 word topic name for this cluster. ONLY RETURN THE TOPIC AND NO OTHER CONTEXT OR WORDS";

    let req_body = ClaudeRequestBody {
        model: "claude-3-haiku-20240307".to_string(),
        system: Some(system_prompt.to_string()),
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: format!("Here are some search queries from a cluster: {query_string}"),
        }],
        max_tokens: 50,
    };

    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();

    let response = req_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&req_body)
        .send()
        .await?;

    // println!("Response: {:?}", response.text().await?);

    let response: AnthropicResponse = response.json().await?;
    let topic_name = response.content[0].text.clone();

    let avg_score = cluster
        .queries
        .iter()
        .map(|q| q.query.top_score)
        .sum::<f32>()
        / cluster.queries.len() as f32;

    let cluster_topic_row = ClusterTopicRow {
        id: uuid::Uuid::new_v4(),
        dataset_id: cluster.dataset_id,
        topic: topic_name,
        density: cluster.queries.len() as u32,
        avg_score: avg_score,
        created_at: OffsetDateTime::now_utc(),
    };

    let memberships: Vec<ClusterMembershipRow> = cluster
        .queries
        .iter()
        .map(|q| ClusterMembershipRow {
            id: uuid::Uuid::new_v4(),
            search_id: q.query.id,
            cluster_id: cluster_topic_row.id,
            distance_to_centroid: q.distance,
        })
        .collect();

    Ok(Topic {
        topic: cluster_topic_row,
        memberships,
    })
}

async fn handle_dataset(
    dataset_id: Uuid,
    client: clickhouse::Client,
    req_client: Arc<reqwest::Client>,
) -> Result<()> {
    let data = fetch_dataset_queries(client.clone(), dataset_id, None).await?;

    // // Perform spherical k-means clustering
    let clusters = hdbscan_clustering(data, dataset_id)?;

    println!("Clusters: {:?}", clusters);

    // Process all topic formations in parallel
    let promises: Vec<tokio::task::JoinHandle<Result<Topic>>> = clusters
        .into_iter()
        .map(|cluster| tokio::spawn(form_topics(cluster, req_client.clone())))
        .collect();

    let mut topics: Vec<Topic> = Vec::new();

    for promise in promises {
        match promise.await {
            Ok(result) => match result {
                // Ok(topic) => topics.push(topic),
                Ok(topic) => {}
                Err(e) => eprintln!("Error forming topic: {:?}", e),
            },
            Err(e) => eprintln!("Task panicked: {:?}", e),
        }
    }

    // let topics = create_clusters

    // // Find the closest queries to the centroids
    // let topics = get_topics(hdbscan, clusters, data);

    // // Insert the topics into the database
    // insert_centroids(client, data, dataset_id, topics, clusters)?;

    Ok(())
}
