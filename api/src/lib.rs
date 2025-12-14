use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Configuration;

pub mod config;
pub mod error;
pub mod routes;

/// Central application state that is shared across all parts of the API.
#[derive(Clone)]
pub struct AppState {
    /// The config data.
    pub config: Arc<Configuration>,

    /// The database connection pool.
    pub db: PgPool,
}
