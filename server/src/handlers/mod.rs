cfg_if::cfg_if! {
    if #[cfg(feature = "monolith")] {
        pub mod auth_handler;
        pub mod card_handler;
        pub mod collection_handler;
        pub mod file_handler;
        pub mod invitation_handler;
        pub mod message_handler;
        pub mod notification_handler;
        pub mod password_reset_handler;
        pub mod register_handler;
        pub mod topic_handler;
        pub mod user_handler;
        pub mod vote_handler;
    } else if #[cfg(feature = "ingest-server")] {
        pub mod ingest_handler;
    }
}
