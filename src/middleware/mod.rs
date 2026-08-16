pub mod validation;

use axum::Router;
use tower_http::{
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
  LatencyUnit,
};
use tracing::Level;

pub fn init_request_logging() {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into()),
    ).init();
}

pub fn with_request_logging(router: Router) -> Router {
  router.layer(
    TraceLayer::new_for_http()
      .make_span_with(DefaultMakeSpan::new()
        .level(Level::INFO))
      .on_request(DefaultOnRequest::new()
        .level(Level::INFO))
      .on_response(
        DefaultOnResponse::new()
          .level(Level::INFO)
          .latency_unit(LatencyUnit::Millis),
      ),
  )
}
