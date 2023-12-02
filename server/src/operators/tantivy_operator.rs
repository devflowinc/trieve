use std::error::Error;
use std::path::Path;

use itertools::Itertools;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::BooleanQuery;
use tantivy::query::QueryParser;
use tantivy::query::TermSetQuery;
use tantivy::query_grammar::Occur;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;

use crate::data::models::CardMetadata;

use super::search_operator::SearchResult;

pub struct TantivyIndex {
    pub index: Index,
    pub schema: Schema,
}

impl TantivyIndex {
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut schema_builder = Schema::builder();

        schema_builder.add_text_field("doc_id", TEXT | STORED);
        schema_builder.add_text_field("card_html", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_dir(&path, schema.clone())?;
        Ok(Self { index, schema })
    }

    pub fn add_card(&self, card: CardMetadata) -> Result<(), Box<dyn Error>> {
        let doc_id = self.schema.get_field("doc_id").unwrap();
        let card_html = self.schema.get_field("card_html").unwrap();
        let mut index_writer = self.index.writer(10_000_000)?;

        index_writer.add_document(doc!(
            doc_id => card.qdrant_point_id.expect("Card needs a qdrant id").to_string(),
            card_html => card.card_html.unwrap_or("".to_string())
        ))?;

        index_writer.commit()?;
        Ok(())
    }

    pub fn search_cards(
        &self,
        query: &str,
        filtered_ids: Option<Vec<uuid::Uuid>>,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let searcher = reader.searcher();

        let doc_id = self.schema.get_field("doc_id").unwrap();
        let card_html = self.schema.get_field("card_html").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![card_html]);

        let query = query_parser.parse_query(query)?;

        let filters = filtered_ids
            .unwrap_or(vec![])
            .iter()
            .map(|x| Term::from_field_text(doc_id, x.to_string().as_str()))
            .collect_vec();

        let filter = TermSetQuery::new(filters);
        let final_query = vec![(Occur::Must, query), (Occur::Must, Box::new(filter))];
        let filters_and_query = BooleanQuery::new(final_query);

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
                    .parse()?,
                score,
            });
        }

        Ok(cards)
    }

    pub fn delete_card(&self, card_id: uuid::Uuid) -> Result<(), Box<dyn Error>> {
        let doc_id = self.schema.get_field("doc_id").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![doc_id]);

        query_parser.parse_query(card_id.to_string().as_str())?;

        let mut index_writer = self.index.writer(10_000_000)?;
        index_writer.delete_term(Term::from_field_text(doc_id, card_id.to_string().as_str()));

        index_writer.commit()?;
        Ok(())
    }

    pub fn update_card(&self, card: CardMetadata) -> Result<(), Box<dyn Error>> {
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

        let mut index_writer = self.index.writer(10_000_000)?;

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
