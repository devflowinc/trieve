use crate::data::models::CardMetadata;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use tantivy::doc;
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
    pub schema: Schema,
}

pub struct TantivyIndexMap {
    pub indices: HashMap<String, TantivyIndex>,
}

impl Default for TantivyIndexMap {
    fn default() -> Self {
        Self::new()
    }
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

        let path_name = format!("../server/tantivy/{}", index_name);
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
            schema,
        };

        self.indices.insert(index_name.to_string(), new_index);

        Ok(())
    }

    pub fn load_tantivy_indexes(&mut self) -> tantivy::Result<()> {
        let path = Path::new("../server/tantivy");
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }

        let paths = std::fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for path in paths {
            let index_name = path.file_name().unwrap().to_str().ok_or_else(|| {
                tantivy::TantivyError::InvalidArgument("Invalid index name".to_string())
            })?;

            self.create_index(index_name)?;
        }

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

        Ok(())
    }

    pub fn commit(&self, index_name: &str) -> tantivy::Result<()> {
        let tantivy_index = self.get_tantivy_index(index_name)?;

        tantivy_index.index_writer.write().unwrap().commit()?;

        Ok(())
    }
}
