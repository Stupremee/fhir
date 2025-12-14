//! The create FHIR resource route.

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;
use sqlx::query;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
};

/// Gets a FHIR entity by it's UUID
#[utoipa::path(
    get,
    path = "/fhir/{resource}/{id}",
    params(
        ("resource", description = "The FHIR resource type to insert"),
    ),
    responses(
        (status = 200, description = "Returns the found FHIR entity"),
        (status = 404, description = "The entity does not exist"),
    )
)]
#[instrument(skip(db))]
#[axum::debug_handler]
pub async fn fhir_get(
    State(AppState { db, .. }): State<AppState>,
    Path((resource, id)): Path<(String, Uuid)>,
) -> Result<Json<Value>> {
    let entity = query!("SELECT fhir_get($1, $2) as entity", resource, id)
        .fetch_one(&db)
        .await?;

    entity.entity.ok_or(AppError::NotFound).map(Json)
}
