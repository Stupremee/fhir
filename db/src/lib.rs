#![deny(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)] // pgrx only allows value passing

use fastrace::{collector::Config, prelude::*};
use pgrx::prelude::*;

mod api;
mod fhir;
mod gucs;
mod hooks;
mod index;
mod macros;
mod models;
mod schema;
mod spi;

::pgrx::pg_module_magic!(name, version);

extension_sql!(
    r"CREATE SCHEMA IF NOT EXISTS fhir;",
    name = "schema",
    bootstrap
);

#[allow(non_snake_case)]
#[pg_guard]
pub unsafe extern "C-unwind" fn _PG_init() {
    gucs::init();

    let jaeger_enabled = gucs::JAEGER_ENABLED
        .get()
        .is_some_and(|s| s.to_str().unwrap() == "true");
    let jaeger_host = gucs::JAEGER_HOST.get();

    if jaeger_enabled {
        if let Some(host) = jaeger_host {
            // FIXME: do not crash the whole database if Jeager host is invalid

            let host = host.to_str().unwrap();
            let reporter =
                fastrace_jaeger::JaegerReporter::new(host.parse().unwrap(), "fhir_extension")
                    .unwrap();

            fastrace::set_reporter(reporter, Config::default());

            info!("starting tracing export to Jaeger {host}");
        } else {
            warning!("fhir.jaeger_enabled is set, but fhir.jaeger_host is not set");
        }
    }

    let root_span = Span::root("extension-init", SpanContext::random());
    let _guard = root_span.set_local_parent();

    hooks::register_hooks();
    fhir::compile_schema();
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::{prelude::*, JsonB, Uuid};

    const PATIENT: &str = r#"{"resourceType":"Patient","id":"66033","meta":{"profile":["http://hl7.org/fhir/uv/ips/StructureDefinition/Patient-uv-ips"]},"language":"en","identifier":[{"system":"urn:oid:1.3.182.4.4","value":"1998041799999"},{"system":"urn:ietf:rfc:3986","value":"urn:uuid:647515ed-0d5e-4c99-b23d-073fbc593f76"}],"name":[{"family":"Lux-Brennard","given":["Marie"]}],"gender":"female","birthDate":"1998-04-17"}"#;

    fn patient() -> JsonB {
        JsonB(serde_json::from_str(PATIENT).unwrap())
    }

    #[pg_test]
    fn insert_valid_patient() {
        let data = patient();
        let mut raw_data = data.0.clone();
        let id = Spi::get_one_with_args::<Uuid>("SELECT fhir_put($1)", &[data.into()]).unwrap();

        let mut got_data =
            Spi::get_one_with_args::<JsonB>("SELECT fhir_get('Patient', $1)", &[id.into()])
                .unwrap()
                .unwrap();

        // `id` wont match
        got_data.0.as_object_mut().unwrap().remove("id");
        raw_data.as_object_mut().unwrap().remove("id");

        assert_eq!(raw_data, got_data.0);

        let history = Spi::get_one_with_args::<JsonB>(
            "SELECT data FROM fhir.entity_history WHERE entity_id = $1",
            &[id.into()],
        )
        .unwrap()
        .unwrap();

        // `resourceType` is not present in history table
        got_data.0.as_object_mut().unwrap().remove("resourceType");

        assert_eq!(history.0, got_data.0);
    }

    #[pg_test(error = "the given entity does not have a 'resourceType'")]
    fn insert_without_resource_type() {
        let mut data = patient();
        data.0.as_object_mut().unwrap().remove("resourceType");

        Spi::run_with_args("SELECT fhir_put($1)", &[data.into()]).unwrap();
    }

    #[pg_test]
    fn ensure_no_id_in_data() {
        let data = patient();
        let id = Spi::get_one_with_args::<Uuid>("SELECT fhir_put($1)", &[data.into()]).unwrap();

        let data = Spi::get_one_with_args::<JsonB>(
            "SELECT data FROM fhir.entity WHERE id = $1",
            &[id.into()],
        )
        .unwrap()
        .unwrap();

        assert!(data.0.as_object().unwrap().get("id").is_none());
    }

    #[pg_test]
    fn fhir_search() {
        let data = patient();
        let id = Spi::get_one_with_args::<Uuid>("SELECT fhir_put($1)", &[data.into()]).unwrap();

        let _data = Spi::get_one_with_args::<Uuid>(
            "SELECT fhir_search('Patient', 'gender', '=', 'female')",
            &[id.into()],
        )
        .unwrap()
        .unwrap();
    }
}
