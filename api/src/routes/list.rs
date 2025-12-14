//! FHIR resource list and search endpoint.

use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::query;
use tracing::instrument;
use utoipa::IntoParams;

use crate::{
    AppState,
    error::{AppError, Result},
};

const fn default_count() -> i64 {
    20
}

/// Query parameters for list and search operations.
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQueryParams {
    #[serde(rename = "_count")]
    #[serde(default = "default_count")]
    #[param(minimum = 1, maximum = 100, default = 20)]
    count: i64,

    #[serde(rename = "_offset")]
    #[serde(default)]
    #[param(minimum = 0, default = 0)]
    offset: i64,

    /// Search parameters as query string parameters.
    #[serde(flatten)]
    search_params: HashMap<String, String>,
}

/// Search FHIR entities
#[utoipa::path(
    get,
    path = "/fhir/{resource}",
    params(
        ("resource", description = "The FHIR resource type (e.g., Patient, Observation)"),
        ListQueryParams,
    ),
    responses(
        (status = 200, description = "Returns a paginated list of FHIR entities", body = Vec<Value>),
    )
)]
#[instrument(skip(db))]
#[axum::debug_handler]
pub async fn fhir_list(
    State(AppState { db, .. }): State<AppState>,
    Path(resource): Path<String>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<Vec<Value>>> {
    let (key, original_value) = params
        .search_params
        .iter()
        .next()
        .ok_or(AppError::BadRequest(Some(
            "exactly one search parameter must be provided",
        )))?;

    let (op, mut value) = original_value.split_at(2);
    let search_op = match op {
        "eq" => "=",
        "ne" => "!=",
        "gt" => ">",
        "ge" => ">=",
        "lt" => "<",
        "le" => "<=",
        // This is a non-standard operator, used for testing performance
        // and nicer usage
        "like" => "~",
        "trgm" => "%",
        _ => {
            value = original_value;
            "="
        }
    };

    let entities = query!(
        r#"
    SELECT
        fhir_get($1, id) as entity
    FROM
        fhir_search($1, $2, $3, $4)
    ORDER BY id
    LIMIT $5
    OFFSET $6
    "#,
        resource,
        key,
        search_op,
        value,
        params.count,
        params.offset,
    )
    .fetch_all(&db)
    .await?;

    Ok(Json(
        entities.into_iter().filter_map(|e| e.entity).collect(),
    ))
}
