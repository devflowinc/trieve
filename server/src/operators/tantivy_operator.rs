use std::path::Path;

use itertools::Itertools;
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
use tantivy::ReloadPolicy;

use crate::data::models::CardMetadata;

use super::search_operator::SearchResult;

pub struct TantivyIndex {
    pub index: Index,
    pub schema: Schema,
}

impl TantivyIndex {
    pub fn new<P>(path: P) -> tantivy::Result<Self>
    where
        P: AsRef<Path>,
    {
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
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();

        schema_builder.add_text_field("doc_id", id_options);
        schema_builder.add_text_field("card_html", card_html_options);
        let schema = schema_builder.build();
        let index = if !path.as_ref().exists() {
            std::fs::create_dir_all(&path)?;
            let index = Index::create_in_dir(&path, schema.clone())?;
            index.tokenizers().register("ngram", ngram_tokenizer);
            index.tokenizers().register("raw_id", raw_tokenizer.clone());
            index
                .fast_field_tokenizer()
                .register("raw_id", raw_tokenizer);
            index
        } else {
            Index::open_in_dir(&path)?
        };

        Ok(Self { index, schema })
    }

    pub fn add_card(&self, card: CardMetadata) -> tantivy::Result<()> {
        let doc_id = self.schema.get_field("doc_id").unwrap();
        let card_html = self.schema.get_field("card_html").unwrap();
        let mut index_writer = self.index.writer(30_000_000)?;

        index_writer.add_document(doc!(
            doc_id => card.qdrant_point_id.expect("Card needs a qdrant id").to_string(),
            card_html => card.card_html.unwrap_or("".to_string())
        ))?;

        //add to some sort of WAL which commits after a certain number of writes
        index_writer.commit()?;
        Ok(())
    }

    pub fn search_cards(
        &self,
        query: &str,
        filtered_ids: Option<Vec<uuid::Uuid>>,
    ) -> tantivy::Result<Vec<SearchResult>> {
        log::info!("Searching for {:?}", query);
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let searcher = reader.searcher();

        let doc_id = self.schema.get_field("doc_id").unwrap();
        let card_html = self.schema.get_field("card_html").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![card_html]);

        let query = query_parser.parse_query_lenient(query).0;
        log::info!("Searching for {:?}", query);
        let filters = filtered_ids
            .unwrap_or(vec![])
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

        log::info!("Searching for {:?}", filters_and_query);

        let top_docs = searcher.search(&filters_and_query, &TopDocs::with_limit(10))?;

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
        log::info!("Found cards {:?}", cards);

        Ok(cards)
    }

    pub fn delete_card(&self, card_id: uuid::Uuid) -> tantivy::Result<()> {
        let doc_id = self.schema.get_field("doc_id").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![doc_id]);

        query_parser.parse_query(card_id.to_string().as_str())?;

        let mut index_writer = self.index.writer(30_000_000)?;
        index_writer.delete_term(Term::from_field_text(doc_id, card_id.to_string().as_str()));

        index_writer.commit()?;
        Ok(())
    }

    pub fn update_card(&self, card: CardMetadata) -> tantivy::Result<()> {
        if card.qdrant_point_id.is_none() {
            return Ok(());
        }
        let doc_id = self.schema.get_field("doc_id").unwrap();
        let card_html = self.schema.get_field("card_html").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![doc_id]);

        query_parser.parse_query(
            card.qdrant_point_id
                .expect("Card needs a qdrant id")
                .to_string()
                .as_str(),
        )?;

        //each of these index_writers allocates 30mb of memory -- can lead to lockup if too many are open
        let mut index_writer = self.index.writer(30_000_000)?;

        index_writer.delete_term(Term::from_field_text(
            doc_id,
            card.qdrant_point_id
                .expect("Card needs a qdrant id")
                .to_string()
                .as_str(),
        ));

        index_writer.add_document(doc!(
            doc_id => card.qdrant_point_id.expect("Card needs a qdrant id").to_string(),
            card_html => card.card_html.unwrap_or("".to_string())
        ))?;

        index_writer.commit()?;
        Ok(())
    }
}
