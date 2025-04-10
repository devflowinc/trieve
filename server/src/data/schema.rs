// @generated automatically by Diesel CLI.

diesel::table! {
    chunk_boosts (chunk_id) {
        chunk_id -> Uuid,
        fulltext_boost_phrase -> Nullable<Text>,
        fulltext_boost_factor -> Nullable<Float8>,
        semantic_boost_phrase -> Nullable<Text>,
        semantic_boost_factor -> Nullable<Float8>,
    }
}

diesel::table! {
    chunk_group (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        dataset_id -> Uuid,
        tracking_id -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        tag_set -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    chunk_group_bookmarks (id) {
        id -> Uuid,
        group_id -> Uuid,
        chunk_metadata_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    chunk_metadata (id) {
        id -> Uuid,
        link -> Nullable<Text>,
        qdrant_point_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        chunk_html -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        tracking_id -> Nullable<Text>,
        time_stamp -> Nullable<Timestamp>,
        dataset_id -> Uuid,
        weight -> Float8,
        location -> Nullable<Jsonb>,
        image_urls -> Nullable<Array<Nullable<Text>>>,
        num_value -> Nullable<Float8>,
    }
}

diesel::table! {
    chunk_metadata_tags (id) {
        id -> Uuid,
        chunk_metadata_id -> Uuid,
        tag_id -> Uuid,
    }
}

diesel::table! {
    crawl_requests (id) {
        id -> Uuid,
        url -> Text,
        status -> Text,
        interval -> Nullable<Int4>,
        next_crawl_at -> Nullable<Timestamp>,
        scrape_id -> Uuid,
        dataset_id -> Uuid,
        created_at -> Timestamp,
        crawl_options -> Jsonb,
        crawl_type -> Text,
    }
}

diesel::table! {
    dataset_event_counts (id) {
        id -> Uuid,
        notification_count -> Int4,
        dataset_uuid -> Uuid,
    }
}

diesel::table! {
    dataset_group_counts (id) {
        id -> Uuid,
        group_count -> Int4,
        dataset_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    dataset_tags (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        tag -> Text,
    }
}

diesel::table! {
    dataset_usage_counts (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        chunk_count -> Int4,
    }
}

diesel::table! {
    datasets (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        organization_id -> Uuid,
        server_configuration -> Jsonb,
        tracking_id -> Nullable<Text>,
        deleted -> Int4,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        file_name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        metadata -> Nullable<Jsonb>,
        link -> Nullable<Text>,
        time_stamp -> Nullable<Timestamp>,
        dataset_id -> Uuid,
        tag_set -> Nullable<Array<Nullable<Text>>>,
        size -> Int8,
    }
}

diesel::table! {
    groups_from_files (id) {
        id -> Uuid,
        group_id -> Uuid,
        file_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    invitations (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        organization_id -> Uuid,
        used -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Int4,
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
    organization_api_key (id) {
        id -> Uuid,
        organization_id -> Uuid,
        api_key_hash -> Text,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Int4,
        dataset_ids -> Nullable<Array<Nullable<Text>>>,
        scopes -> Nullable<Array<Nullable<Text>>>,
        params -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organization_usage_counts (id) {
        id -> Uuid,
        org_id -> Uuid,
        dataset_count -> Int4,
        user_count -> Int4,
        message_count -> Int4,
        chunk_count -> Int4,
        file_storage -> Int8,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        registerable -> Nullable<Bool>,
        deleted -> Int4,
        partner_configuration -> Jsonb,
    }
}

diesel::table! {
    stripe_invoices (id) {
        id -> Uuid,
        org_id -> Uuid,
        total -> Int4,
        created_at -> Timestamp,
        status -> Text,
        hosted_invoice_url -> Text,
        stripe_id -> Nullable<Text>,
    }
}

diesel::table! {
    stripe_plans (id) {
        id -> Uuid,
        stripe_id -> Text,
        chunk_count -> Int4,
        user_count -> Int4,
        dataset_count -> Int4,
        message_count -> Int4,
        amount -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Text,
        visible -> Bool,
        file_storage -> Int8,
        component_loads -> Nullable<Int4>,
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
        current_period_end -> Nullable<Timestamp>,
    }
}

diesel::table! {
    stripe_usage_based_plans (id) {
        id -> Uuid,
        name -> Text,
        visible -> Bool,
        ingest_tokens_price_id -> Text,
        bytes_ingested_price_id -> Text,
        search_tokens_price_id -> Text,
        message_tokens_price_id -> Text,
        analytics_events_price_id -> Text,
        ocr_pages_price_id -> Text,
        pages_crawls_price_id -> Text,
        datasets_price_id -> Text,
        users_price_id -> Text,
        chunks_stored_price_id -> Text,
        files_storage_price_id -> Text,
        created_at -> Timestamp,
        platform_price_id -> Nullable<Text>,
        platform_price_amount -> Nullable<Int4>,
    }
}

diesel::table! {
    stripe_usage_based_subscriptions (id) {
        id -> Uuid,
        organization_id -> Uuid,
        stripe_subscription_id -> Text,
        usage_based_plan_id -> Uuid,
        created_at -> Timestamp,
        last_recorded_meter -> Timestamp,
        last_cycle_timestamp -> Timestamp,
        last_cycle_dataset_count -> Int8,
        last_cycle_users_count -> Int4,
        last_cycle_chunks_stored_mb -> Int8,
        last_cycle_files_storage_mb -> Int8,
        current_period_end -> Nullable<Timestamp>,
    }
}

diesel::table! {
    topics (id) {
        id -> Uuid,
        name -> Text,
        deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        dataset_id -> Uuid,
        owner_id -> Text,
    }
}

diesel::table! {
    user_api_key (id) {
        id -> Uuid,
        user_id -> Uuid,
        api_key_hash -> Nullable<Text>,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Int4,
        blake3_hash -> Nullable<Text>,
        dataset_ids -> Nullable<Array<Nullable<Text>>>,
        organization_ids -> Nullable<Array<Nullable<Text>>>,
        scopes -> Nullable<Array<Nullable<Text>>>,
        params -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamp>,
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
        name -> Nullable<Text>,
        #[max_length = 255]
        oidc_subject -> Varchar,
    }
}

diesel::joinable!(chunk_boosts -> chunk_metadata (chunk_id));
diesel::joinable!(chunk_group -> datasets (dataset_id));
diesel::joinable!(chunk_group_bookmarks -> chunk_group (group_id));
diesel::joinable!(chunk_group_bookmarks -> chunk_metadata (chunk_metadata_id));
diesel::joinable!(chunk_metadata -> datasets (dataset_id));
diesel::joinable!(chunk_metadata_tags -> chunk_metadata (chunk_metadata_id));
diesel::joinable!(chunk_metadata_tags -> dataset_tags (tag_id));
diesel::joinable!(crawl_requests -> datasets (dataset_id));
diesel::joinable!(dataset_event_counts -> datasets (dataset_uuid));
diesel::joinable!(dataset_tags -> datasets (dataset_id));
diesel::joinable!(dataset_usage_counts -> datasets (dataset_id));
diesel::joinable!(datasets -> organizations (organization_id));
diesel::joinable!(files -> datasets (dataset_id));
diesel::joinable!(groups_from_files -> chunk_group (group_id));
diesel::joinable!(groups_from_files -> files (file_id));
diesel::joinable!(messages -> datasets (dataset_id));
diesel::joinable!(messages -> topics (topic_id));
diesel::joinable!(organization_api_key -> organizations (organization_id));
diesel::joinable!(organization_usage_counts -> organizations (org_id));
diesel::joinable!(stripe_invoices -> organizations (org_id));
diesel::joinable!(stripe_subscriptions -> organizations (organization_id));
diesel::joinable!(stripe_subscriptions -> stripe_plans (plan_id));
diesel::joinable!(stripe_usage_based_subscriptions -> organizations (organization_id));
diesel::joinable!(stripe_usage_based_subscriptions -> stripe_usage_based_plans (usage_based_plan_id));
diesel::joinable!(topics -> datasets (dataset_id));
diesel::joinable!(user_api_key -> users (user_id));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(user_organizations -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    chunk_boosts,
    chunk_group,
    chunk_group_bookmarks,
    chunk_metadata,
    chunk_metadata_tags,
    crawl_requests,
    dataset_event_counts,
    dataset_group_counts,
    dataset_tags,
    dataset_usage_counts,
    datasets,
    files,
    groups_from_files,
    invitations,
    messages,
    organization_api_key,
    organization_usage_counts,
    organizations,
    stripe_invoices,
    stripe_plans,
    stripe_subscriptions,
    stripe_usage_based_plans,
    stripe_usage_based_subscriptions,
    topics,
    user_api_key,
    user_organizations,
    users,
);
