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
        collision_id -> Uuid,
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
        oc_file_path -> Nullable<Text>,
        card_html -> Nullable<Text>,
        private -> Bool,
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
    }
}

diesel::table! {
    invitations (id) {
        id -> Uuid,
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
    }
}

diesel::joinable!(card_collection -> users (author_id));
diesel::joinable!(card_collection_bookmarks -> card_collection (collection_id));
diesel::joinable!(card_collection_bookmarks -> card_metadata (card_metadata_id));
diesel::joinable!(card_metadata -> users (author_id));
diesel::joinable!(card_votes -> card_metadata (card_metadata_id));
diesel::joinable!(card_votes -> users (voted_user_id));
diesel::joinable!(messages -> topics (topic_id));
diesel::joinable!(topics -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    card_collection,
    card_collection_bookmarks,
    card_collisions,
    card_metadata,
    card_votes,
    invitations,
    messages,
    password_resets,
    stripe_customers,
    topics,
    user_plans,
    users,
);
