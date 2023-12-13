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
    datasets (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        organization_id -> Uuid,
    }
}

diesel::joinable!(card_metadata -> datasets (dataset_id));
diesel::joinable!(datasets -> organizations (organization_id));

diesel::allow_tables_to_appear_in_same_query!(card_metadata, datasets, organizations,);
