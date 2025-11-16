use json_diff_ng::{compare_serde_values, PathElement};
use pgrx::{prelude::*, spi::SpiError, JsonB, Uuid};
use thiserror::Error;

use crate::spi;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Null Trigger Tuple found")]
    NullTriggerTuple,

    #[error("\"data\" column is null")]
    DataNull,

    #[error("failed to diff JSONB values")]
    JsonDiff(#[from] json_diff_ng::Error),

    #[error("{0}")]
    PgTrigger(#[from] PgTriggerError),

    #[error("{0}")]
    Spi(#[from] SpiError),

    #[error("{0}")]
    TryFromDatum(#[from] pgrx::datum::TryFromDatumError),
}

#[pg_trigger]
pub fn fhir_log_entity_history<'t>(
    trigger: &'t pgrx::PgTrigger<'t>,
) -> Result<Option<PgHeapTuple<'t, impl WhoAllocated>>, TriggerError> {
    let op = trigger.op()?;

    let new = trigger
        .new()
        .map(PgHeapTuple::into_owned)
        .ok_or(TriggerError::NullTriggerTuple);
    let old = trigger
        .old()
        .map(PgHeapTuple::into_owned)
        .ok_or(TriggerError::NullTriggerTuple);

    match op {
        PgTriggerOperation::Insert => {
            let new = new?;

            let entity_id = new.get_by_name::<Uuid>("id")?;
            let data = new.get_by_name::<JsonB>("data")?;

            spi::run_with_args(
                r#"
                INSERT INTO "fhir"."entity_history"
                    ("entity_id", "timestamp", "operation", "data")
                VALUES
                    ($1, now(), 'insert', $2);
                "#,
                &[entity_id.into(), data.into()],
            )?;

            Ok(Some(new))
        }
        PgTriggerOperation::Update => {
            let new = new?;
            let old = old?;

            let entity_id = new.get_by_name::<Uuid>("id")?;
            let old_data = old
                .get_by_name::<JsonB>("data")?
                .ok_or(TriggerError::DataNull)?;
            let new_data = new
                .get_by_name::<JsonB>("data")?
                .ok_or(TriggerError::DataNull)?;

            if new_data.0 == old_data.0 {
                return Ok(Some(new));
            }

            let diff = compare_serde_values(&old_data.0, &new_data.0, true, &[])?;

            let mut changed_values = serde_json::Map::new();
            for v in diff.unequal_values.get_diffs() {
                let value = v.values.map(|v| v.1).cloned().unwrap_or_default();
                changed_values.insert(path_to_string(&v.path), value);
            }

            let mut added_values = serde_json::Map::new();
            for v in diff.right_only.get_diffs() {
                let value = v.values.map(|v| v.1).cloned().unwrap_or_default();
                added_values.insert(path_to_string(&v.path), value);
            }

            let mut removed_values = serde_json::Map::new();
            for v in diff.left_only.get_diffs() {
                let value = v.values.map(|v| v.1).cloned().unwrap_or_default();
                removed_values.insert(path_to_string(&v.path), value);
            }

            spi::run_with_args(
                r#"
                INSERT INTO "fhir"."entity_history"
                    (
                        "entity_id",
                        "timestamp",
                        "operation",
                        "update_removed_values",
                        "update_changed_values",
                        "update_added_values"
                    )
                VALUES
                    ($1, now(), 'update', $2, $3, $4);
                "#,
                &[
                    entity_id.into(),
                    JsonB(removed_values.into()).into(),
                    JsonB(changed_values.into()).into(),
                    JsonB(added_values.into()).into(),
                ],
            )?;

            Ok(Some(new))
        }
        PgTriggerOperation::Delete => {
            let old = old?;

            let entity_id = old.get_by_name::<Uuid>("id")?;
            let data = old.get_by_name::<JsonB>("data")?;

            spi::run_with_args(
                r#"
                INSERT INTO "fhir"."entity_history"
                    ("entity_id", "timestamp", "operation", "data")
                VALUES
                    ($1, now(), 'delete', $2);
                "#,
                &[entity_id.into(), data.into()],
            )?;

            Ok(Some(old))
        }
        PgTriggerOperation::Truncate => Ok(None),
    }
}

/// Converts a list of [`PathElement`] into a string.
fn path_to_string(elements: &[PathElement<'_>]) -> String {
    let mut path = String::new();

    for (i, p) in elements.iter().enumerate() {
        if i > 0 {
            path.push('.');
        }

        path.push_str(&p.to_string());
    }

    path
}
