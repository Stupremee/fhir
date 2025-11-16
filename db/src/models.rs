//! JSON models for FHIR entities.
//!
//! These models are not meant to be complete representations of the FHIR specification.
//! They are only used in places where access to certain data is required.

use serde::{Deserialize, Serialize};

use crate::enum_display_serde;

/// [NameUse](<https://hl7.org/fhir/valueset-name-use.html>)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NameUse {
    Usual,
    Official,
    Temp,
    Nickname,
    Anonymous,
    Old,
    Maiden,
}
enum_display_serde!(NameUse);

/// [AdministrativeGender](<https://hl7.org/fhir/valueset-administrative-gender.html>)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdministrativeGender {
    Male,
    Female,
    Other,
    Unknown,
}
enum_display_serde!(AdministrativeGender);

/// [HumanName](<https://hl7.org/fhir/datatypes.html#HumanName>)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanName {
    #[serde(rename = "use")]
    pub use_: Option<NameUse>,
    pub text: Option<String>,
    pub family: Option<String>,
    pub given: Option<Vec<String>>,
    pub prefix: Option<Vec<String>>,
    pub suffix: Option<Vec<String>>,
}

/// [Patient](<https://hl7.org/fhir/patient.html>)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub gender: Option<AdministrativeGender>,
    pub name: Option<Vec<HumanName>>,
    pub birth_date: Option<String>,
}
