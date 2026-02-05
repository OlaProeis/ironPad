use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::services::search;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub fn router() -> Router {
    Router::new().route("/", get(search_notes))
}

async fn search_notes(Query(params): Query<SearchQuery>) -> impl IntoResponse {
    match search::search_notes(&params.q) {
        Ok(results) => Json(results).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Search failed: {}", err),
        )
            .into_response(),
    }
}
