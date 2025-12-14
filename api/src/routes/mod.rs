use axum::Router;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable as _};

use crate::AppState;

mod create;
mod get;
mod history;
mod list;

pub fn build_router() -> Router<AppState> {
    let (router, openapi) = OpenApiRouter::<AppState>::new()
        .routes(routes!(create::fhir_create, list::fhir_list))
        .routes(routes!(history::fhir_get_history))
        .routes(routes!(get::fhir_get))
        .split_for_parts();

    router.merge(Scalar::with_url("/docs", openapi))
}
