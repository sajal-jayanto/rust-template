mod db;
mod middleware;
mod models;
mod routes;
mod services;

#[tokio::main]
async fn main() {
  dotenvy::dotenv().ok();
  middleware::init_request_logging();

  let db_pool = db::setup().await;
  let app = routes::routes(db_pool);

  let addr = "127.0.0.1:3000";
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
  axum::serve(listener, app).await.unwrap();
}
