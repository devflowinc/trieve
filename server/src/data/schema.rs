// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tsvector", schema = "pg_catalog"))]
    pub struct Tsvector;
}

diesel::table! {
    card_collection (id) {
        id -> Uuid,
        author_id -> Uuid,
        name -> Text,
        is_public -> Bool,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    card_collection_bookmarks (id) {
        id -> Uuid,
        collection_id -> Uuid,
        card_metadata_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    card_collisions (id) {
        id -> Uuid,
        card_id -> Uuid,
        collision_qdrant_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    card_files (id) {
        id -> Uuid,
        card_id -> Uuid,
        file_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Tsvector;

    card_metadata (id) {
        id -> Uuid,
        content -> Text,
        link -> Nullable<Text>,
        author_id -> Uuid,
        qdrant_point_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        tag_set -> Nullable<Text>,
        card_html -> Nullable<Text>,
        card_metadata_tsvector -> Nullable<Tsvector>,
        private -> Bool,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    card_metadata_count (id) {
        id -> Uuid,
        total_rows -> Int8,
    }
}

diesel::table! {
    card_verification (id) {
        id -> Uuid,
        card_id -> Uuid,
        similarity_score -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    card_votes (id) {
        id -> Uuid,
        voted_user_id -> Uuid,
        card_metadata_id -> Uuid,
        vote -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted -> Bool,
    }
}

diesel::table! {
    collections_from_files (id) {
        id -> Uuid,
        collection_id -> Uuid,
        file_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    cut_cards (id) {
        id -> Uuid,
        user_id -> Uuid,
        cut_card_content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    file_upload_completed_notifications (id) {
        id -> Uuid,
        user_uuid -> Uuid,
        collection_uuid -> Uuid,
        user_read -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        user_id -> Uuid,
        file_name -> Text,
        mime_type -> Text,
        private -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        size -> Int8,
        tag_set -> Nullable<Text>,
    }
}

diesel::table! {
    invitations (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        referral_tokens -> Nullable<Text>,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        topic_id -> Uuid,
        sort_order -> Int4,
        content -> Text,
        #[max_length = 10]
        role -> Varchar,
        deleted -> Bool,
        prompt_tokens -> Nullable<Int4>,
        completion_tokens -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stripe_customers (id) {
        id -> Uuid,
        stripe_id -> Text,
        #[max_length = 100]
        email -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    topics (id) {
        id -> Uuid,
        user_id -> Uuid,
        resolution -> Text,
        side -> Bool,
        deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        normal_chat -> Bool,
    }
}

diesel::table! {
    user_collection_counts (id) {
        id -> Uuid,
        user_id -> Uuid,
        collection_count -> Int4,
    }
}

diesel::table! {
    user_notification_counts (id) {
        id -> Uuid,
        user_uuid -> Uuid,
        notification_count -> Int4,
    }
}

diesel::table! {
    user_plans (id) {
        id -> Uuid,
        stripe_customer_id -> Text,
        stripe_subscription_id -> Text,
        plan -> Text,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        hash -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        username -> Nullable<Text>,
        website -> Nullable<Text>,
        visible_email -> Bool,
        api_key_hash -> Nullable<Text>,
    }
}

diesel::table! {
    verification_notifications (id) {
        id -> Uuid,
        user_uuid -> Uuid,
        card_uuid -> Uuid,
        verification_uuid -> Uuid,
        similarity_score -> Int8,
        user_read -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(card_collection -> users (author_id));
diesel::joinable!(card_collection_bookmarks -> card_collection (collection_id));
diesel::joinable!(card_collection_bookmarks -> card_metadata (card_metadata_id));
diesel::joinable!(card_files -> card_metadata (card_id));
diesel::joinable!(card_files -> files (file_id));
diesel::joinable!(card_metadata -> users (author_id));
diesel::joinable!(card_verification -> card_metadata (card_id));
diesel::joinable!(card_votes -> card_metadata (card_metadata_id));
diesel::joinable!(card_votes -> users (voted_user_id));
diesel::joinable!(collections_from_files -> card_collection (collection_id));
diesel::joinable!(collections_from_files -> files (file_id));
diesel::joinable!(cut_cards -> users (user_id));
diesel::joinable!(file_upload_completed_notifications -> card_collection (collection_uuid));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(messages -> topics (topic_id));
diesel::joinable!(topics -> users (user_id));
diesel::joinable!(user_collection_counts -> users (user_id));
diesel::joinable!(user_notification_counts -> users (user_uuid));
diesel::joinable!(verification_notifications -> card_metadata (card_uuid));
diesel::joinable!(verification_notifications -> card_verification (verification_uuid));
diesel::joinable!(verification_notifications -> users (user_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    card_collection,
    card_collection_bookmarks,
    card_collisions,
    card_files,
    card_metadata,
    card_metadata_count,
    card_verification,
    card_votes,
    collections_from_files,
    cut_cards,
    file_upload_completed_notifications,
    files,
    invitations,
    messages,
    password_resets,
    stripe_customers,
    topics,
    user_collection_counts,
    user_notification_counts,
    user_plans,
    users,
    verification_notifications,
);
