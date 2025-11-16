//! Responsible for generating indexable values from FHIR entities.

use std::collections::HashMap;

use fastrace::trace;
use pgrx::{
    datum::{Date, DatumWithOid},
    Uuid,
};
use serde_json::Value;

use crate::spi;

mod patient;

/// Collection of values that must be inserted into the index tables.
///
/// This is separated into a struct, to allow for first collecting the values,
/// but then inserting them at a later point.
pub struct IndexableValues {
    text: Option<HashMap<&'static str, Vec<String>>>,
    date: Option<HashMap<&'static str, Vec<Date>>>,
}

impl IndexableValues {
    fn insert_values<'d, T: Into<DatumWithOid<'d>>>(
        suffix: &str,
        id: Uuid,
        vals: HashMap<&'static str, Vec<T>>,
    ) -> spi::Result<()> {
        for (key, values) in vals {
            for value in values {
                spi::run_with_args(
                    &format!(
                        r#"
                    INSERT INTO "fhir"."entity_index_{suffix}" ("entity_id", "key", "value")
                    VALUES ($1, $2, $3);
                    "#
                    ),
                    &[id.into(), key.into(), value.into()],
                )?;
            }
        }

        Ok(())
    }

    /// Inserts all indexable values into the database.
    #[trace]
    pub fn insert(self, id: Uuid) -> spi::Result<()> {
        if let Some(text_values) = self.text.filter(|v| !v.is_empty()) {
            Self::insert_values("text", id, text_values)?;
        }

        if let Some(date_values) = self.date.filter(|v| !v.is_empty()) {
            Self::insert_values("date", id, date_values)?;
        }

        Ok(())
    }
}

/// Generates a list of text index values for the given entity.
///
/// The returned value is a map that maps the name / parameter of the
/// indexable value, to a list of values that can be used for searching.
///
/// The list key + value combinations will then be inserted into the
/// `entity_index_text` table.
#[trace]
fn text_index_values_for<'d>(
    entity: &str,
    data: &Value,
) -> Option<HashMap<&'static str, Vec<String>>> {
    Some(match entity {
        "Patient" => patient::text_index_values_for(data),
        _ => return None,
    })
}

/// Generates a list of date index values for the given entity.
#[trace]
fn date_index_values_for<'d>(
    entity: &str,
    data: &Value,
) -> Option<HashMap<&'static str, Vec<Date>>> {
    Some(match entity {
        "Patient" => patient::date_index_values_for(data),
        _ => return None,
    })
}

/// Collects all indexable values for the given entity.
#[trace]
pub fn collect_index_values_for(entity: &str, data: &Value) -> IndexableValues {
    let text = text_index_values_for(entity, data);
    let date = date_index_values_for(entity, data);

    IndexableValues { text, date }
}
