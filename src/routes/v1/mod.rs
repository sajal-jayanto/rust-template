mod sample;

use axum::Router;

pub fn routes() -> Router {
  Router::new().nest("/sample", sample::routes())
}
