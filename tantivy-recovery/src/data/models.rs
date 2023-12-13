use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema::*;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone)]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: uuid::Uuid,
    pub name: String,
    pub configuration: serde_json::Value,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    pub registerable: Option<bool>,
}

impl Organization {
    pub fn from_details(name: String, configuration: serde_json::Value) -> Self {
        Organization {
            id: uuid::Uuid::new_v4(),
            name,
            configuration,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            registerable: Some(true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone)]
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
}

impl Dataset {
    pub fn from_details(name: String, organization_id: uuid::Uuid) -> Self {
        Dataset {
            id: uuid::Uuid::new_v4(),
            name,
            organization_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone)]
#[diesel(table_name = card_metadata)]
pub struct CardMetadata {
    pub id: uuid::Uuid,
    pub content: String,
    pub link: Option<String>,
    pub author_id: uuid::Uuid,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub card_html: Option<String>,
    pub private: bool,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
}

impl CardMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        content: S,
        card_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<String>,
        author_id: T,
        qdrant_point_id: Option<uuid::Uuid>,
        private: bool,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        dataset_id: uuid::Uuid,
    ) -> Self {
        CardMetadata {
            id: uuid::Uuid::new_v4(),
            content: content.into(),
            card_html: card_html.clone(),
            link: link.clone(),
            author_id: author_id.into(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            private,
            metadata,
            tracking_id,
            time_stamp,
            dataset_id,
        }
    }
}

impl CardMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details_with_id<S: Into<String>, T: Into<uuid::Uuid>>(
        id: T,
        content: S,
        card_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<String>,
        author_id: T,
        qdrant_point_id: Option<uuid::Uuid>,
        private: bool,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        dataset_id: uuid::Uuid,
    ) -> Self {
        CardMetadata {
            id: id.into(),
            content: content.into(),
            card_html: card_html.clone(),
            link: link.clone(),
            author_id: author_id.into(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            private,
            metadata,
            tracking_id,
            time_stamp,
            dataset_id,
        }
    }
}
