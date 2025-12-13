//! Gathering of indexable values for the `Patient` entity.

use std::{collections::HashMap, str::FromStr as _};

use pgrx::{datum::Date, warning};
use serde_json::Value;

use crate::{index::IndexedKeyType, models::Patient};

pub fn find_search_index_for_key(key: &str) -> Option<IndexedKeyType> {
    Some(match key {
        "birth_date" => IndexedKeyType::Date,
        "gender" | "name" => IndexedKeyType::Text,
        _ => return None,
    })
}

pub fn date_index_values_for(data: &Value) -> HashMap<&'static str, Vec<Date>> {
    let mut keys = HashMap::new();

    let patient: Patient = serde_json::from_value(data.clone()).expect("invalid patient data");

    if let Some(v) = patient.birth_date {
        if let Ok(date) = Date::from_str(v.as_str()) {
            keys.insert("birth_date", vec![date]);
        } else {
            warning!("invalid date value for birth_date: {v}");
        }
    }

    keys
}

pub fn text_index_values_for(data: &Value) -> HashMap<&'static str, Vec<String>> {
    let mut keys = HashMap::new();

    let patient: Patient = serde_json::from_value(data.clone()).expect("invalid patient data");

    if let Some(v) = patient.gender {
        keys.insert("gender", vec![v.to_string()]);
    }

    // re-construct the full name of the patient
    let mut full_name = String::new();
    for name in patient.name.unwrap_or_default() {
        let parts = name
            .prefix
            .iter()
            .flatten()
            .chain(name.given.iter().flatten())
            .chain(name.family.iter())
            .chain(name.suffix.iter().flatten());

        for part in parts {
            full_name.push_str(part);
            full_name.push(' ');
        }
    }

    if !full_name.trim().is_empty() {
        keys.insert("name", vec![full_name.trim().to_lowercase()]);
    }

    keys
}
