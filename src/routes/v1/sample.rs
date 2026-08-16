use axum::{Router, routing::{get, post}};

use crate::services::sample::{create_sample, get_all_sample, get_sample_by_id};

pub fn routes() -> Router {
  Router::new()
    .route("/", get(get_all_sample))
    .route("/{id}", get(get_sample_by_id))
    .route("/", post(create_sample))
}