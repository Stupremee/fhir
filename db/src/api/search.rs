use std::str::FromStr;

use fastrace::prelude::*;
use pgrx::{datum::DatumWithOid, prelude::*, Uuid};
use thiserror::Error;

use crate::index::{self, IndexedKeyType};

/// Errors that can occurr in the [`fhir_search`] function.
#[derive(Debug, Error)]
pub enum SearchError {
    /// The provided operator string is not a valid search operator.
    #[error("unknown search operator: '{0}'")]
    UnknownOperator(String),

    /// The provided search key is not indexed.
    #[error("unknown search key: '{0}'")]
    UnknownSearchKey(String),

    /// The search value has wrong type.
    #[error("the search value is not valid for this search key")]
    InvalidValueType,

    #[error("{0}")]
    Spi(
        #[source]
        #[from]
        pgrx::spi::Error,
    ),
}

/// Search operators for filtering FHIR entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOperator {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    Trgm,
}

impl SearchOperator {
    /// Converts the search operator to its corresponding Postgres operator string.
    pub fn to_postgres_operator(self) -> &'static str {
        match self {
            SearchOperator::Eq => "=",
            SearchOperator::Ne => "!=",
            SearchOperator::Lt => "<",
            SearchOperator::Lte => "<=",
            SearchOperator::Gt => ">",
            SearchOperator::Gte => ">=",
            SearchOperator::Like => "ilike",
            SearchOperator::Trgm => "%",
        }
    }
}

impl FromStr for SearchOperator {
    type Err = SearchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eq" | "=" => Ok(SearchOperator::Eq),
            "ne" | "!=" | "<>" => Ok(SearchOperator::Ne),
            "lt" | "<" => Ok(SearchOperator::Lt),
            "lte" | "<=" => Ok(SearchOperator::Lte),
            "gt" | ">" => Ok(SearchOperator::Gt),
            "gte" | ">=" => Ok(SearchOperator::Gte),
            "like" | "~" => Ok(SearchOperator::Like),
            "%" => Ok(SearchOperator::Trgm),
            _ => Err(SearchError::UnknownOperator(s.to_string())),
        }
    }
}

/// The types that can be passed as the search value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchValue {
    Text(String),
    Date(Date),
}

/// [`fhir_search`] overload with string as search value.
#[pg_extern(name = "fhir_search")]
#[trace]
pub fn fhir_search_text(
    entity: &str,
    key: &str,
    op: &str,
    value: String,
) -> Result<TableIterator<'static, (name!(idx, i64), name!(id, Uuid))>, SearchError> {
    fhir_search(entity, key, op, SearchValue::Text(value))
}

/// [`fhir_search`] overload with date as search value.
#[pg_extern(name = "fhir_search")]
#[trace]
pub fn fhir_search_date(
    entity: &str,
    key: &str,
    op: &str,
    value: Date,
) -> Result<TableIterator<'static, (name!(idx, i64), name!(id, Uuid))>, SearchError> {
    fhir_search(entity, key, op, SearchValue::Date(value))
}

/// Searches for FHIR entities based on indexed search parameters.
///
/// This function performs searches against the FHIR entity index tables
/// to efficiently find entities that match the specified search criteria.
#[trace]
pub fn fhir_search(
    entity: &str,
    key: &str,
    op: &str,
    value: SearchValue,
) -> Result<TableIterator<'static, (name!(idx, i64), name!(id, Uuid))>, SearchError> {
    let op = SearchOperator::from_str(op)?;
    let psql_op = op.to_postgres_operator();

    let index_type = index::find_search_index_for_key(entity, key)
        .ok_or_else(|| SearchError::UnknownSearchKey(key.to_string()))?;

    // TODO: throw error on invalid operators for data type
    let (table_suffix, value): (&str, DatumWithOid<'_>) = match (index_type, value) {
        (IndexedKeyType::Text, SearchValue::Text(v)) => ("text", v.into()),
        (IndexedKeyType::Date, SearchValue::Date(v)) => ("date", v.into()),
        _ => return Err(SearchError::InvalidValueType),
    };

    let ids = {
        let _guard = LocalSpan::enter_with_local_parent("spi_select");

        Spi::connect(|conn| {
            conn.select(
                &format!(
                    r#"
                SELECT
                    "entity_id"
                FROM
                    "fhir"."entity_index_{table_suffix}"
                WHERE
                    "entity" = $1
                    and "key" = $2
                    and "value" {psql_op} $3
                "#,
                ),
                None,
                &[entity.into(), key.into(), value],
            )?
            .filter_map(|row| row["entity_id"].value::<Uuid>().transpose())
            .collect::<pgrx::spi::Result<Vec<_>>>()
        })?
    };

    Ok(TableIterator::new(ids.into_iter().enumerate().map(
        move |(idx, id)| {
            (
                i64::try_from(idx).expect("usize to i64 conversion failed"),
                id,
            )
        },
    )))
}
