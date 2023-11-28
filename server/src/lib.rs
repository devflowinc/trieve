
use tokio::sync::Semaphore;

cfg_if::cfg_if! {
    if #[cfg(feature = "monolith")] {
        #[macro_use]
        extern crate diesel;
        mod data;
    }
}

mod errors;
mod handlers;
mod operators;


#[macro_export]
#[cfg(not(feature = "runtime-env"))]
macro_rules! get_env {
    ($name:expr, $message:expr) => {
        env!($name, $message)
    };
}

pub struct AppMutexStore {
    pub embedding_semaphore: Option<Semaphore>,
}


#[macro_export]
#[cfg(feature = "runtime-env")]
macro_rules! get_env {
    ($name:expr, $message:expr) => {{
        lazy_static::lazy_static! {
            static ref ENV_VAR: String = {
                std::env::var($name).expect($message)
            };
        }
        ENV_VAR.as_str()
    }};
}

#[cfg(not(feature = "ingest-server"))]
mod server;

#[cfg(feature = "ingest-server")]
mod ingest;
#[cfg(feature = "ingest-server")]
use ingest as server;

pub use server::main;
