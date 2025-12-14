use fastrace::{prelude::*, trace};
use pgrx::{prelude::*, JsonB, Uuid};
use serde_json::Value;

/// Gets a FHIR resource for a certain id.
#[pg_extern]
#[trace]
pub fn fhir_get(entity: String, id: Uuid) -> Option<JsonB> {
    let mut data = {
        let _guard = LocalSpan::enter_with_local_parent("spi");

        Spi::connect(|client| {
            client.select(
                r#"SELECT "data" FROM "fhir"."entity" WHERE "id" = $1 AND "resource_type" = $2;"#,
                Some(1),
                &[id.into(), entity.as_str().into()],
            )
            .expect("Failed to get entity")
            .next()?["data"]
            .value::<JsonB>().expect("data of fhir entity must be JsonB")
        })?
    };

    let obj = data.0.as_object_mut().expect("Entity must be an object");

    obj.insert("id".to_string(), Value::String(id.to_string()));
    obj.insert("resourceType".to_string(), Value::String(entity));

    Some(data)
}
