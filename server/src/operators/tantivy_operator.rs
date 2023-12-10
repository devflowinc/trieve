use super::search_operator::SearchResult;
use crate::data::models::CardMetadata;
use actix::Arbiter;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::RwLock;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::BooleanQuery;
use tantivy::query::QueryParser;
use tantivy::query::TermSetQuery;
use tantivy::query_grammar::Occur;
use tantivy::schema::*;
use tantivy::tokenizer::LowerCaser;
use tantivy::tokenizer::NgramTokenizer;
use tantivy::tokenizer::RawTokenizer;
use tantivy::tokenizer::RemoveLongFilter;
use tantivy::tokenizer::TextAnalyzer;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::ReloadPolicy;

pub struct TantivyIndex {
    pub index_name: String,
    pub index: Index,
    pub index_writer: Arc<RwLock<IndexWriter>>,
    pub index_reader: Arc<IndexReader>,
    pub commit_queue: Arc<CommitQueue>,
    pub schema: Schema,
}

pub struct TantivyIndexMap {
    pub indices: HashMap<String, TantivyIndex>,
}

impl TantivyIndexMap {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub fn create_index(&mut self, index_name: &str) -> tantivy::Result<()> {
        if self.indices.contains_key(index_name) {
            return Ok(());
        }

        let path_name = format!("./tantivy/{}", index_name);
        let path = Path::new(&path_name);

        let mut schema_builder = Schema::builder();

        let id_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw_id")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_fast(Some("raw_id"))
            .set_stored();

        let ngram_tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 10, false).unwrap())
            .filter(RemoveLongFilter::limit(255))
            .filter(LowerCaser)
            .build();

        let raw_tokenizer = TextAnalyzer::builder(RawTokenizer::default())
            .filter(RemoveLongFilter::limit(255))
            .build();

        let card_html_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("ngram")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        schema_builder.add_text_field("doc_id", id_options);
        schema_builder.add_text_field("card_html", card_html_options);
        let schema = schema_builder.build();
        let index = if !path.exists() {
            std::fs::create_dir_all(path)?;
            Index::create_in_dir(path, schema.clone())?
        } else {
            Index::open_in_dir(path)?
        };

        index.tokenizers().register("ngram", ngram_tokenizer);
        index.tokenizers().register("raw_id", raw_tokenizer.clone());
        index
            .fast_field_tokenizer()
            .register("raw_id", raw_tokenizer);

        let index_writer = Arc::new(RwLock::new(index.writer(30_000_000)?));
        let commit_queue = Arc::new(CommitQueue::new(index_writer.clone()));

        commit_queue.run();
        let commit_queue_inner = commit_queue.clone();
        Arbiter::new().spawn(async move {
            commit_queue_inner.wait_for_job();
        });

        let index_reader = Arc::new(
            index
                .reader_builder()
                .reload_policy(ReloadPolicy::OnCommit)
                .try_into()?,
        );

        let new_index = TantivyIndex {
            index_name: index_name.to_string(),
            index,
            index_writer,
            index_reader,
            commit_queue,
            schema,
        };

        self.indices.insert(index_name.to_string(), new_index);

        Ok(())
    }

    fn get_tantivy_index(&self, index_name: &str) -> tantivy::Result<&TantivyIndex> {
        match self.indices.get(index_name) {
            Some(index) => Ok(index),
            None => Err(tantivy::TantivyError::InvalidArgument(
                "Index not found".to_string(),
            )),
        }
    }

    pub fn add_card(&self, index_name: &str, card: CardMetadata) -> tantivy::Result<()> {
        let tantivy_index = self.get_tantivy_index(index_name)?;

        let doc_id = tantivy_index.schema.get_field("doc_id").unwrap();
        let card_html = tantivy_index.schema.get_field("card_html").unwrap();

        tantivy_index
            .index_writer
            .read()
            .unwrap()
            .add_document(doc!(
                doc_id => card.qdrant_point_id.expect("Card needs a qdrant id").to_string(),
                card_html => card.card_html.unwrap_or("".to_string())
            ))?;

        //add to some sort of WAL which commits after a certain number of writes
        tantivy_index
            .commit_queue
            .add_commit(card.qdrant_point_id.unwrap());

        Ok(())
    }

    pub fn search_cards(
        &self,
        index_name: &str,
        query: &str,
        page: u64,
        filtered_ids: Option<Vec<uuid::Uuid>>,
    ) -> tantivy::Result<Vec<SearchResult>> {
        let tantivy_index = self.get_tantivy_index(index_name)?;

        let searcher = tantivy_index.index_reader.searcher();

        let doc_id = tantivy_index.schema.get_field("doc_id").unwrap();
        let card_html = tantivy_index.schema.get_field("card_html").unwrap();

        let query_parser = QueryParser::for_index(&tantivy_index.index, vec![card_html]);

        let query = query_parser.parse_query_lenient(query).0;
        let filters = filtered_ids
            .unwrap_or_default()
            .iter()
            .map(|x| Term::from_field_text(doc_id, x.to_string().as_str()))
            .collect_vec();

        let filters_and_query = if !filters.is_empty() {
            let filter = TermSetQuery::new(filters);
            let final_query = vec![(Occur::Must, query), (Occur::Must, Box::new(filter))];
            BooleanQuery::new(final_query)
        } else {
            let final_query = vec![(Occur::Must, query)];
            BooleanQuery::new(final_query)
        };
        let top_docs = searcher.search(
            &filters_and_query,
            &TopDocs::with_limit(10).and_offset(((page - 1) * 10) as usize),
        )?;

        let mut cards = vec![];

        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            cards.push(SearchResult {
                point_id: retrieved_doc
                    .get_first(doc_id)
                    .unwrap()
                    .as_text()
                    .expect("Value should be text")
                    .parse()
                    .map_err(|_| {
                        tantivy::TantivyError::InvalidArgument("Could not parse uuid".to_string())
                    })?,
                score,
            });
        }

        log::info!("Found {:?}", cards);

        Ok(cards)
    }

    pub fn delete_card(&self, index_name: &str, card_id: uuid::Uuid) -> tantivy::Result<()> {
        let tantivy_index = self.get_tantivy_index(index_name)?;

        let doc_id = tantivy_index.schema.get_field("doc_id").unwrap();

        let query_parser = QueryParser::for_index(&tantivy_index.index, vec![doc_id]);

        query_parser.parse_query(card_id.to_string().as_str())?;

        tantivy_index
            .index_writer
            .read()
            .unwrap()
            .delete_term(Term::from_field_text(doc_id, card_id.to_string().as_str()));

        tantivy_index.index_writer.write().unwrap().commit()?;

        Ok(())
    }

    pub fn update_card(&self, index_name: &str, card: CardMetadata) -> tantivy::Result<()> {
        let tantivy_index = self.get_tantivy_index(index_name)?;

        if card.qdrant_point_id.is_none() {
            return Ok(());
        }
        let doc_id = tantivy_index.schema.get_field("doc_id").unwrap();
        let card_html = tantivy_index.schema.get_field("card_html").unwrap();

        //each of these index_writers allocates 30mb of memory -- can lead to lockup if too many are open

        tantivy_index
            .index_writer
            .read()
            .unwrap()
            .delete_term(Term::from_field_text(
                doc_id,
                card.qdrant_point_id
                    .expect("Card needs a qdrant id")
                    .to_string()
                    .as_str(),
            ));

        tantivy_index
            .index_writer
            .read()
            .unwrap()
            .add_document(doc!(
                doc_id => card.qdrant_point_id.expect("Card needs a qdrant id").to_string(),
                card_html => card.card_html.unwrap_or("".to_string())
            ))?;

        tantivy_index.index_writer.write().unwrap().commit()?;

        Ok(())
    }
}

pub struct CommitQueue {
    jobs: Mutex<VecDeque<uuid::Uuid>>,
    index_writer: Arc<RwLock<IndexWriter>>,
    cvar: Arc<Condvar>,
}

impl CommitQueue {
    pub fn new(index_writer: Arc<RwLock<IndexWriter>>) -> Self {
        CommitQueue {
            jobs: Mutex::new(VecDeque::new()),
            index_writer,
            cvar: Arc::new(Condvar::new()),
        }
    }
    pub fn add_commit(&self, card_id: uuid::Uuid) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.append(&mut vec![card_id].into_iter().collect());
    }

    pub fn wait_for_job(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        loop {
            if !jobs.is_empty() {
                let mut index_writer = self.index_writer.write().unwrap();
                index_writer.commit().unwrap();
                *jobs = VecDeque::new();
            } else {
                jobs = self.cvar.wait(jobs).unwrap();
            }
        }
    }

    pub fn run(&self) {
        let cvar = self.cvar.clone();
        Arbiter::new().spawn(async move {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(10));
                cvar.notify_all();
            }
        });
    }
}
