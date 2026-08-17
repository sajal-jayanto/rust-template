use axum::{Router, routing::{delete, get, post, put}};

use crate::services::sample::{
  create_sample, 
  delete_sample, 
  get_all_sample, 
  get_sample_by_id, 
  swap_sample_names,
  update_sample,
};

pub fn routes() -> Router {
  Router::new()
    .route("/", get(get_all_sample))
    .route("/", post(create_sample))
    .route("/{id}", get(get_sample_by_id))
    .route("/{id}", put(update_sample))
    .route("/{id}", delete(delete_sample))
    .route("/swap", post(swap_sample_names))
}