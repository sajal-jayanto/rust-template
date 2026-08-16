use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Serialize, sqlx::FromRow)]
pub struct Sample {
  pub id: i64,
  pub name: String,
  pub created_at: DateTime<Utc>,
}

const NAME_MAX_LEN: u64 = 255;

#[derive(Deserialize, Validate)]
pub struct CreateSample {
  #[validate(custom(function = "validate_name"))]
  pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateSample {
  #[validate(custom(function = "validate_name"))]
  pub name: String,
}

#[derive(Deserialize, Validate)]
pub struct SampleIdParams {
  #[validate(range(min = 1, message = "id must be a positive integer"))]
  pub id: i64,
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
  if name.trim().is_empty() {
    return Err(
      ValidationError::new("empty")
        .with_message("name must not be empty".into())
    );
  }
  if name.chars().count() as u64 > NAME_MAX_LEN {
    return Err(
      ValidationError::new("too_long")
        .with_message(format!("name must be at most {NAME_MAX_LEN} characters").into()),
    );
  }
  Ok(())
}
