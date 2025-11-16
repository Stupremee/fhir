use fastrace::trace;
use pgrx::{prelude::*, JsonB, Uuid};
use serde_json::Value;

use crate::{api::common::fhir_generate_id, index::collect_index_values_for, spi};

/// Inserts a new FHIR resource into the database.
///
/// This function will only insert a new entity, and will not update existing entities.
#[pg_extern]
#[trace]
pub fn fhir_put(mut entity: JsonB) -> Uuid {
    let entity_obj = entity.0.as_object_mut().expect("Entity must be an object");

    let Some(Value::String(resource_type)) = entity_obj.remove("resourceType") else {
        panic!("the given entity does not have a 'resourceType'");
    };

    entity_obj.remove("id");

    let indexable_values = collect_index_values_for(&resource_type, &entity.0);

    let id = fhir_generate_id();
    spi::run_with_args(
        r#"
        INSERT INTO "fhir"."entity" ("id", "resource_type", "data") VALUES ($1, $2, $3);
        "#,
        &[id.into(), resource_type.into(), entity.into()],
    )
    .expect("Failed to insert entity");

    indexable_values
        .insert(id)
        .expect("Failed to insert index values");

    id
}
