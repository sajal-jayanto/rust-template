use axum::{
  extract::{FromRequest, FromRequestParts, Path, Query, Request},
  http::{request::Parts, StatusCode},
  Json,
};
use serde::{de::DeserializeOwned, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Serialize)]
pub struct ErrorResponse {
  pub error: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<ValidationErrors>,
}

impl ErrorResponse {
  pub fn message(message: impl Into<String>) -> Self {
    Self { error: message.into(), details: None }
  }
}

pub type ValidationRejection = (StatusCode, Json<ErrorResponse>);

fn bad_request(message: impl Into<String>) -> ValidationRejection {
  (StatusCode::BAD_REQUEST, Json(ErrorResponse::message(message)))
}

/// Builds a 500 response with a plain error message. Shared so any service can
/// map a fallible operation to a consistent error shape without redefining it.
pub fn internal_error(message: impl Into<String>) -> ValidationRejection {
  (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::message(message)))
}

/// Logs the underlying error (e.g. a `sqlx::Error`) via `tracing::error!` before
/// building the generic 500 response, so the client-facing message can stay vague
/// without losing the real cause from the logs. Use in place of
/// `map_err(|_| internal_error(...))`, e.g. `map_err(|err| log_internal_error(err, "failed to fetch samples"))`.
pub fn log_internal_error(err: impl std::fmt::Display, message: &str) -> ValidationRejection {
  tracing::error!(%err, "{message}");
  internal_error(message)
}

/// Builds a 404 response with a plain error message. Shared so any service can
/// map a missing row to a consistent error shape without redefining it.
pub fn not_found(message: impl Into<String>) -> ValidationRejection {
  (StatusCode::NOT_FOUND, Json(ErrorResponse::message(message)))
}

fn invalid(errors: ValidationErrors) -> ValidationRejection {
  (
    StatusCode::BAD_REQUEST,
    Json(ErrorResponse { error: "validation failed".into(), details: Some(errors) }),
  )
}

/// Extracts and validates a JSON body: `ValidatedJson(payload): ValidatedJson<T>`.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ValidationRejection;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|err| bad_request(err.to_string()))?;
    value.validate().map_err(invalid)?;
    Ok(ValidatedJson(value))
  }
}

/// Extracts and validates query parameters: `ValidatedQuery(params): ValidatedQuery<T>`.
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ValidationRejection;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Query(value) = Query::<T>::from_request_parts(parts, state)
      .await
      .map_err(|err| bad_request(err.to_string()))?;
    value.validate().map_err(invalid)?;
    Ok(ValidatedQuery(value))
  }
}

/// Extracts and validates path parameters: `ValidatedPath(params): ValidatedPath<T>`.
pub struct ValidatedPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
  T: DeserializeOwned + Validate + Send,
  S: Send + Sync,
{
  type Rejection = ValidationRejection;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Path(value) = Path::<T>::from_request_parts(parts, state)
      .await
      .map_err(|err| bad_request(err.to_string()))?;
    value.validate().map_err(invalid)?;
    Ok(ValidatedPath(value))
  }
}
