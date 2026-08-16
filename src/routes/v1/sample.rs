use axum::{Router, routing::{get, post, put}};

use crate::services::sample::{create_sample, get_all_sample, get_sample_by_id, update_sample};

pub fn routes() -> Router {
  Router::new()
    .route("/", get(get_all_sample))
    .route("/{id}", get(get_sample_by_id))
    .route("/", post(create_sample))
    .route("/{id}", put(update_sample))
}