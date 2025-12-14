//! The create FHIR resource route.

use axum::{
    Json,
    extract::{Path, State},
};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, error::Result};

/// Entity successfully created.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateResponse {
    id: Uuid,
}

/// Insert a new FHIR entity
///
/// The resource type path parameter will be inserted into the body as the `resourceType` key.
/// If an existing `resourceType` field already exists in the data, the value will be overwritten.
#[utoipa::path(
    post,
    path = "/fhir/{resource}",
    request_body(description = "The FHIR entity data"),
    params(
        ("resource", description = "The FHIR resource type to insert"),
    ),
    responses(
        (status = 201, description = "Entity inserted successfully", body = inline(CreateResponse))
    )
)]
#[instrument(skip(db))]
#[axum::debug_handler]
pub async fn fhir_create(
    State(AppState { db, .. }): State<AppState>,
    Path(resource): Path<String>,
    Json(mut body): Json<serde_json::Map<String, Value>>,
) -> Result<Json<CreateResponse>> {
    body.insert("resourceType".to_string(), resource.into());

    let inserted = query!("SELECT fhir_put($1) as id", Value::Object(body))
        .fetch_one(&db)
        .await?;

    let Some(id) = inserted.id else {
        return Err(eyre!("`fhir_put` did not return a row").into());
    };

    Ok(Json(CreateResponse { id }))
}
