use axum::{Extension, Json, http::StatusCode};
use sqlx::PgPool;

use crate::db;
use crate::middleware::validation::{internal_error, not_found, ErrorResponse, ValidatedJson, ValidatedPath};
use crate::models::sample::{CreateSample, Sample, SampleIdParams};

pub async fn get_all_sample(
  Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<Sample>>, (StatusCode, Json<ErrorResponse>)> {
  let samples = db::find_all::<Sample>(
    &pool, 
    "sample_table", 
    "id, name, created_at"
  )
  .await
  .map_err(|_| internal_error("failed to fetch samples"))?;

  Ok(Json(samples))
}

pub async fn get_sample_by_id(
  Extension(pool): Extension<PgPool>,
  ValidatedPath(params): ValidatedPath<SampleIdParams>,
) -> Result<Json<Sample>, (StatusCode, Json<ErrorResponse>)> {
  let sample = db::find_one::<Sample>(
    &pool,
    "sample_table",
    "id, name, created_at",
    "id",
    params.id.into(),
  )
  .await
  .map_err(|_| internal_error("failed to fetch sample"))?
  .ok_or_else(|| not_found("sample not found"))?;

  Ok(Json(sample))
}

pub async fn create_sample(
  Extension(pool): Extension<PgPool>,
  ValidatedJson(payload): ValidatedJson<CreateSample>,
) -> Result<(StatusCode, Json<Sample>), (StatusCode, Json<ErrorResponse>)> {
  let name = payload.name.trim().to_string();

  let sample = db::insert::<Sample>(
    &pool,
    "sample_table",
    &["name"],
    vec![name.into()],
    "id, name, created_at",
  )
  .await
  .map_err(|_| internal_error("failed to create sample"))?;

  Ok((StatusCode::CREATED, Json(sample)))
}
