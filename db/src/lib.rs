#![deny(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)] // pgrx only allows value passing

use fastrace::{collector::Config, prelude::*};
use pgrx::prelude::*;

mod api;
mod fhir;
mod gucs;
mod hooks;
mod schema;

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
