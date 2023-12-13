extern crate diesel;
pub mod tantivy_operator;

use diesel::r2d2::ConnectionManager;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, Queryable, RunQueryDsl, SelectableHelper};
use dotenvy::dotenv;
use itertools::Itertools;
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::{env, sync::RwLock};

use crate::{
    data::models::{CardMetadata, Dataset},
    tantivy_operator::TantivyIndexMap,
};

pub mod data;

pub fn establish_connection() -> Arc<r2d2::Pool<ConnectionManager<PgConnection>>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to {}", database_url);
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Arc::new(
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool."),
    )
}

#[derive(Queryable, Debug, Clone, Serialize)]
struct DatasetAndOrg {
    org_id: uuid::Uuid,
    dataset: Dataset,
}

fn main() {
    println!("Recovery started.");
    std::io::stdout().flush().unwrap();
    let pool = establish_connection();
    use crate::data::schema::organizations::dsl as org_columns;
    let datasets_and_orgs = org_columns::organizations
        .inner_join(crate::data::schema::datasets::dsl::datasets)
        .select((org_columns::id, Dataset::as_select()))
        .load::<DatasetAndOrg>(
            &mut pool
                .get()
                .map_err(|_| println!("Failed getting pool connection"))
                .unwrap(),
        )
        .map_err(|_| println!("Error loading organizations!"))
        .unwrap();
    println!("Loaded datasets and orgs");
    std::io::stdout().flush().unwrap();

    let tantivy_map = Arc::new(RwLock::new(TantivyIndexMap::new()));
    tantivy_map.write().unwrap().load_tantivy_indexes().unwrap();

    println!("Found {} datasets.", datasets_and_orgs.len());
    std::io::stdout().flush().unwrap();

    use crate::data::schema::card_metadata::dsl as card_columns;
    let datasets: Vec<Vec<DatasetAndOrg>> = datasets_and_orgs
        .into_iter()
        .chunks(2)
        .into_iter()
        .map(|c| c.collect())
        .collect();
    let mut handlers = vec![];

    for chunked_datasets in datasets {
        let tantivy_index = tantivy_map.clone();
        let pool1 = pool.clone();

        let handler = std::thread::spawn(move || {
            let mut conn = pool1
                .get()
                .map_err(|_| println!("Failed getting pool connection"))
                .unwrap();

            for dataset_and_org in chunked_datasets {
                println!("Recovering dataset {}.", dataset_and_org.dataset.name);
                let card_metadatas = card_columns::card_metadata
                    .filter(card_columns::dataset_id.eq(dataset_and_org.dataset.id))
                    .filter(card_columns::qdrant_point_id.is_not_null())
                    .load::<CardMetadata>(&mut conn)
                    .map_err(|_| println!("Error loading card metadata!"))
                    .unwrap();

                println!("Found {} cards", card_metadatas.len());

                tantivy_index
                    .write()
                    .unwrap()
                    .create_index(&dataset_and_org.dataset.id.to_string())
                    .map_err(|_| println!("Error creating index!"))
                    .unwrap();

                for card_metadata in card_metadatas {
                    tantivy_index
                        .read()
                        .unwrap()
                        .add_card(&dataset_and_org.dataset.id.to_string(), card_metadata)
                        .map_err(|_| println!("Error adding card!"))
                        .unwrap();
                }
                tantivy_index
                    .read()
                    .unwrap()
                    .commit(&dataset_and_org.dataset.id.to_string())
                    .map_err(|_| println!("Error committing index!"))
                    .unwrap();
            }
        });
        handlers.push(handler);
    }
    handlers.into_iter().for_each(|h| h.join().unwrap());
}
