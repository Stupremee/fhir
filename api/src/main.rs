use std::sync::Arc;

use api::{AppState, config::Configuration, error::AppError, routes};
use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::Response,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use eyre::{Context as _, Result};
use init_tracing_opentelemetry::TracingConfig;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_log_error::log_error;

async fn log_app_error(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if let Some(err) = response.extensions().get::<Arc<AppError>>() {
        match &**err {
            api::error::AppError::Internal(report) => log_error!(**report, "internal server error"),
            api::error::AppError::InternalBoxed(error) => {
                log_error!(**error, "internal server error")
            }
            api::error::AppError::Database(error) => log_error!(error, "database error"),
            _ => {}
        }
    }

    response
}

#[tokio::main]
async fn main() -> Result<()> {
    // load env variables, this is mainly useful for development
    let _ = dotenv::dotenv();

    let config = Configuration::load()?;

    // initialize tracing + opentelemetry
    let tracing_config = if config.is_production() {
        TracingConfig::production()
    } else {
        TracingConfig::development()
    };
    let _guard = tracing_config.init_subscriber()?;

    // connect to database
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .wrap_err("could not initialize database connection")?;

    sqlx::query("SELECT 1")
        .execute(&db)
        .await
        .wrap_err("health check to database failed")?;
    info!("database connection established");

    let state = AppState {
        config: Arc::new(config),
        db,
    };

    // construct the axum router
    let router = Router::new()
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .layer(TraceLayer::new_for_http())
        .merge(routes::build_router())
        // must be after route registration, in order to run correctly
        .layer(from_fn(log_app_error))
        .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3100);

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .wrap_err("failed to start listener")?;

    info!(port, "starting HTTP server");

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            warn!("shutting down server")
        })
        .await
        .wrap_err("could not start HTTP server")?;

    Ok(())
}
