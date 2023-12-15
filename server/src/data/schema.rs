// @generated automatically by Diesel CLI.

diesel::table! {
    card_collection (id) {
        id -> Uuid,
        author_id -> Uuid,
        name -> Text,
        is_public -> Bool,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        dataset_id -> Uuid,
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
        private -> Bool,
        metadata -> Nullable<Jsonb>,
        tracking_id -> Nullable<Text>,
        time_stamp -> Nullable<Timestamp>,
        dataset_id -> Uuid,
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
    datasets (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        organization_id -> Uuid,
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
        dataset_id -> Uuid,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        user_id -> Uuid,
        file_name -> Text,
        private -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        size -> Int8,
        tag_set -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        link -> Nullable<Text>,
        time_stamp -> Nullable<Timestamp>,
        dataset_id -> Uuid,
    }
}

diesel::table! {
    invitations (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        dataset_id -> Uuid,
        used -> Bool,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        dataset_id -> Uuid,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        configuration -> Jsonb,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        registerable -> Nullable<Bool>,
    }
}

diesel::table! {
    stripe_plans (id) {
        id -> Uuid,
        stripe_id -> Text,
        card_count -> Int4,
        file_storage -> Int4,
        user_count -> Int4,
        dataset_count -> Int4,
        message_count -> Int4,
        amount -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stripe_subscriptions (id) {
        id -> Uuid,
        stripe_id -> Text,
        plan_id -> Uuid,
        organization_id -> Uuid,
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
        dataset_id -> Uuid,
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
    user_organizations (id) {
        id -> Uuid,
        user_id -> Uuid,
        organization_id -> Uuid,
        role -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        username -> Nullable<Text>,
        website -> Nullable<Text>,
        visible_email -> Bool,
        api_key_hash -> Nullable<Text>,
        name -> Nullable<Text>,
    }
}

diesel::joinable!(card_collection -> datasets (dataset_id));
diesel::joinable!(card_collection -> users (author_id));
diesel::joinable!(card_collection_bookmarks -> card_collection (collection_id));
diesel::joinable!(card_collection_bookmarks -> card_metadata (card_metadata_id));
diesel::joinable!(card_files -> card_metadata (card_id));
diesel::joinable!(card_files -> files (file_id));
diesel::joinable!(card_metadata -> datasets (dataset_id));
diesel::joinable!(card_metadata -> users (author_id));
diesel::joinable!(collections_from_files -> card_collection (collection_id));
diesel::joinable!(collections_from_files -> files (file_id));
diesel::joinable!(cut_cards -> users (user_id));
diesel::joinable!(datasets -> organizations (organization_id));
diesel::joinable!(file_upload_completed_notifications -> card_collection (collection_uuid));
diesel::joinable!(file_upload_completed_notifications -> datasets (dataset_id));
diesel::joinable!(files -> datasets (dataset_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(messages -> datasets (dataset_id));
diesel::joinable!(messages -> topics (topic_id));
diesel::joinable!(stripe_subscriptions -> organizations (organization_id));
diesel::joinable!(stripe_subscriptions -> stripe_plans (plan_id));
diesel::joinable!(topics -> datasets (dataset_id));
diesel::joinable!(topics -> users (user_id));
diesel::joinable!(user_collection_counts -> users (user_id));
diesel::joinable!(user_notification_counts -> users (user_uuid));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(user_organizations -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    card_collection,
    card_collection_bookmarks,
    card_collisions,
    card_files,
    card_metadata,
    collections_from_files,
    cut_cards,
    datasets,
    file_upload_completed_notifications,
    files,
    invitations,
    messages,
    organizations,
    stripe_plans,
    stripe_subscriptions,
    topics,
    user_collection_counts,
    user_notification_counts,
    user_organizations,
    users,
);
