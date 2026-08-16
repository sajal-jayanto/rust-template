mod v1;

use crate::middleware;
use crate::services::health::get_health;
use axum::{Router, extract::Extension, http::StatusCode, response::IntoResponse, routing::get};
use sqlx::PgPool;
use std::time::Instant;

pub fn routes(db_pool: PgPool) -> Router {
  let router = Router::new()
    .nest(
      "/api",
      Router::new()
        .route("/health", get(get_health))
        .nest("/v1", v1::routes()),
    )
    .layer(Extension(db_pool))
    .layer(Extension(Instant::now()))
    .fallback(handler_404);

  middleware::with_request_logging(router)
}

async fn handler_404(uri: axum::http::Uri) -> impl IntoResponse {
  (StatusCode::NOT_FOUND, format!("No end point found for path: {}", uri))
}
