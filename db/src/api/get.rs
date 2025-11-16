use fastrace::trace;
use pgrx::{prelude::*, JsonB, Uuid};
use serde_json::Value;

/// Gets a FHIR resource for a certain id.
#[pg_extern]
#[trace]
pub fn fhir_get(entity: String, id: Uuid) -> Option<JsonB> {
    let mut data = Spi::get_one_with_args::<JsonB>(
        r#"
        SELECT "data" FROM "fhir"."entity" WHERE "id" = $1 AND "resource_type" = $2;
        "#,
        &[id.into(), entity.as_str().into()],
    )
    .expect("Failed to get entity")?;

    let obj = data.0.as_object_mut().expect("Entity must be an object");

    obj.insert("id".to_string(), Value::String(id.to_string()));
    obj.insert("resourceType".to_string(), Value::String(entity));

    Some(data)
}
