use std::sync::Arc;

use anyhow::Result;
use clickhouse::Row;
use hdbscan::{HdbscanError, HdbscanHyperParams, HyperParamBuilder};
use serde::Deserialize;
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

    // Use tokio to parallelize the dataset processing
    let mut thread_handles = Vec::new();
    for dataset_id in datasets.clone() {
        let client = clickhouse_client.clone();
        let dataset_id = dataset_id.clone();
        let thread = tokio::spawn(async move {
            handle_dataset(dataset_id, client).await.unwrap();
        });
        thread_handles.push(thread);
    }
    for thread in thread_handles {
        thread.await.unwrap();
    }

    Ok(())
}

#[derive(Row, Deserialize)]
struct QueryRow {
    #[serde(with = "clickhouse::serde::uuid")]
    id: Uuid,
    query: String,
    top_score: f32,
    query_vector: Vec<f32>,
}

async fn fetch_dataset_vectors(
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

fn hdbscan_clustering(data: Vec<QueryRow>) -> Result<()> {
    let vectors: Vec<Vec<f32>> = data.iter().map(|row| row.query_vector.clone()).collect();

    let params = HdbscanHyperParams::builder().min_cluster_size(30).build();

    let clusterer = hdbscan::Hdbscan::new(&vectors, params);

    let labels = clusterer.cluster()?;
    // let something = clusterer.calc_centers(hdbscan::Center::Centroid, &labels)?;

    // Print all the labels
    println!("Labels: {:?}", labels);

    // Match the something back to the data
    for (i, label) in labels.iter().enumerate() {
        if *label < 0 {
            continue;
        }
        let row = &data[i];
        let query = &row.query;
        let top_score = &row.top_score;
        println!("{}: {}", query, top_score);
    }

    Ok(())
}

async fn handle_dataset(dataset_id: Uuid, client: clickhouse::Client) -> Result<()> {
    let data = fetch_dataset_vectors(client.clone(), dataset_id, None).await?;

    // // Perform spherical k-means clustering
    let clusters = hdbscan_clustering(data)?;
    //
    // let clusters = get_clusters(hdbscan, data);
    //
    // // Find the closest queries to the centroids
    // let topics = get_topics(hdbscan, clusters, data);
    //
    // // Insert the topics into the database
    // insert_centroids(client, data, dataset_id, topics, clusters)?;

    Ok(())
}
