//! Wrappers around the [`Spi`] functions.
//!
//! Functions are wrapped to allow recording of traces using [`fasttrace`].

use fastrace::trace;
use pgrx::{datum::DatumWithOid, Spi};

pub use pgrx::spi::Result;

#[trace]
pub fn run_with_args<'mcx>(query: &str, args: &[DatumWithOid<'mcx>]) -> Result<()> {
    Spi::run_with_args(query, args)
}
