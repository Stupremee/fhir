//! This module contains the full FHIR JSON schema file.

use std::cell::OnceCell;

use fastrace::trace;
use jsonschema::Validator;
use serde_json::Value;

static FULL_SCHEMA: &str = include_str!("../../assets/fhir.schema.json");

thread_local! {
    // Cache the `Validator` for faster validation.
    //
    // With this we don't have to re-compile the schema every time.
    static VALIDATOR: OnceCell<Validator> = const { OnceCell::new() };
}

/// Compiles the FHIR json schema.
#[trace]
pub fn compile_schema() {
    is_valid(&Value::Null);
}

/// Checks if the given JSON value matches the FHIR schema.
#[trace]
pub fn is_valid(obj: &Value) -> bool {
    VALIDATOR.with(|raw| {
        let validator = raw.get_or_init(|| {
            let parsed: Value =
                serde_json::from_str(FULL_SCHEMA).expect("the included FHIR schema is invalid");

            jsonschema::validator_for(&parsed).expect("failed to compile FHIR schema")
        });

        validator.is_valid(obj)
    })
}
