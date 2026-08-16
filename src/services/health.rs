use axum::{extract::Extension, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::time::Instant;

#[derive(Serialize)]
pub struct HealthStatus {
  pub status: &'static str,
  pub time: DateTime<Utc>,
  pub uptime_seconds: u64,
  pub database: &'static str,
}

pub async fn get_health(
  Extension(pool): Extension<PgPool>,
  Extension(started_at): Extension<Instant>,
) -> (StatusCode, Json<HealthStatus>) {
  let db_ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();

  let status = HealthStatus {
    status: if db_ok { "ok" } else { "degraded" },
    time: Utc::now(),
    uptime_seconds: started_at.elapsed().as_secs(),
    database: if db_ok { "connected" } else { "unreachable" },
  };

  let code = if db_ok {
    StatusCode::OK
  } else {
    StatusCode::SERVICE_UNAVAILABLE
  };

  (code, Json(status))
}
