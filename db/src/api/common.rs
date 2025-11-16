use pgrx::{prelude::*, JsonB};
use serde_json::Value;

use crate::fhir;

/// Checks if the `data` matches the given JSON schema.
#[pg_extern]
pub fn fhir_is_valid(entity: &str, mut data: JsonB) -> bool {
    let Some(data_obj) = data.0.as_object_mut() else {
        return false;
    };

    match data_obj.get("resourceType").and_then(|v| v.as_str()) {
        Some(resource_type) if resource_type == entity => {}
        Some(_) => return false,
        None => {
            data_obj.insert(
                "resourceType".to_string(),
                Value::String(entity.to_string()),
            );
        }
    }

    fhir::is_valid(&data.0)
}

/// Generates a new UUID v7.
///
/// These ids are used for all FHIR resources as their identifier.
#[pg_extern]
pub fn fhir_generate_id() -> pgrx::Uuid {
    pgrx::Uuid::from_bytes(uuid::Uuid::now_v7().into_bytes())
}
