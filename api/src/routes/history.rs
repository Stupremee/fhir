//! FHIR resource history endpoint.
//!
//! This module provides functionality to retrieve the complete history of a FHIR entity,
//! including all insert, update, and delete operations that have occurred over time.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query;
use time::OffsetDateTime;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
};

/// Represents a single operation that was performed on a FHIR entity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "operation", rename_all = "lowercase")]
pub enum EntityHistoryOperation {
    /// An insert operation that created the entity.
    #[serde(rename = "insert")]
    Insert { data: Value },

    /// An update operation that modified the entity.
    #[serde(rename = "update")]
    Update {
        /// Fields that were added to the entity during this update.
        added: Value,
        /// Fields that were changed during this update.
        changed: Value,
        /// Fields that were removed from the entity during this update.
        removed: Value,
    },

    /// A delete operation that removed the entity.
    #[serde(rename = "delete")]
    Delete,
}

/// A single entry in the entity's history timeline.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EntityHistoryEntry {
    /// The exact timestamp when this operation was performed.
    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,

    /// The operation that was performed at this timestamp.
    #[serde(flatten)]
    operation: EntityHistoryOperation,
}

/// The complete response for a FHIR entity history request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EntityHistoryResponse {
    /// The current, complete FHIR entity in its latest state.
    current: Value,

    /// The complete chronological history of all operations on this entity.
    history: Vec<EntityHistoryEntry>,
}

/// Get history of a FHIR entity
///
/// This endpoint returns both the current state of the entity and a chronological
/// list of all operations (insert, update, delete) that have been performed on it.
#[utoipa::path(
    get,
    path = "/fhir/{resource}/{id}/_history",
    params(
        ("resource", description = "The FHIR resource type"),
        ("id", description = "The unique UUID identifier of the entity"),
    ),
    responses(
        (status = 200, description = "Returns the found FHIR entity plus its history", body = EntityHistoryResponse),
        (status = 404, description = "The entity does not exist"),
    )
)]
#[instrument(skip(db))]
#[axum::debug_handler]
pub async fn fhir_get_history(
    State(AppState { db, .. }): State<AppState>,
    Path((resource, id)): Path<(String, Uuid)>,
) -> Result<Json<EntityHistoryResponse>> {
    let current_entity = query!("SELECT fhir_get($1, $2) as entity", resource, id)
        .fetch_one(&db)
        .await?;

    let current = current_entity.entity.ok_or(AppError::NotFound)?;

    let history_rows = query!(
        r#"
        SELECT
            timestamp,
            operation::text as "operation!",
            data,
            update_removed_values,
            update_changed_values,
            update_added_values
        FROM fhir.entity_history
        WHERE entity_id = $1
        ORDER BY timestamp ASC
        "#,
        id
    )
    .fetch_all(&db)
    .await?;

    let history = history_rows
        .into_iter()
        .map(|row| {
            let operation = match row.operation.as_str() {
                "insert" => EntityHistoryOperation::Insert {
                    data: row.data.unwrap_or(Value::Null),
                },
                "update" => EntityHistoryOperation::Update {
                    added: row.update_added_values.unwrap_or(Value::Null),
                    changed: row.update_changed_values.unwrap_or(Value::Null),
                    removed: row.update_removed_values.unwrap_or(Value::Null),
                },
                "delete" => EntityHistoryOperation::Delete,
                op => unreachable!("unknown history operation: {op}"),
            };

            EntityHistoryEntry {
                timestamp: row.timestamp,
                operation,
            }
        })
        .collect();

    Ok(Json(EntityHistoryResponse { current, history }))
}
