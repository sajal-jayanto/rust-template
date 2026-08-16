// Create a migration (up + down pair):  sqlx migrate add -r <description>
// Run all pending "up" migrations:      sqlx migrate run
// Revert the last applied migration:    sqlx migrate revert

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgRow};
use sqlx::{Executor, FromRow, Postgres, QueryBuilder};
use std::error::Error;
use std::time::Duration;

fn env(key: &str) -> Result<String, Box<dyn Error>> {
  std::env::var(key).map_err(|_| format!("{key} must be set").into())
}

async fn init_pool() -> Result<PgPool, Box<dyn Error>> {
  let options = PgConnectOptions::new()
    .host(&env("DB_HOST")?)
    .port(env("DB_PORT")?.parse()?)
    .username(&env("DB_USER")?)
    .password(&env("DB_PASSWORD")?)
    .database(&env("DB_NAME")?);

  let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(5))
    .connect_with(options)
    .await?;

  Ok(pool)
}

pub async fn setup() -> PgPool {
  let pool = match init_pool().await {
    Ok(pool) => pool,
    Err(err) => {
      eprintln!("\x1b[31m❌ failed to connect to the database: {err}\x1b[0m");
      std::process::exit(1);
    }
  };

  if let Err(err) = sqlx::migrate!("./migrations").run(&pool).await {
    eprintln!("\x1b[31m❌ failed to run pending migrations: {err}\x1b[0m");
    std::process::exit(1);
  }
  println!("\x1b[32m✅ migrations up to date\x1b[0m");

  return pool;
}

/// A dynamically typed bind value for `insert`. Add a variant (and a `From` impl)
/// when a service needs to insert a column type not covered here.
pub enum SqlParam {
  Text(String),
  Int(i64),
  Bool(bool),
  Float(f64),
  Timestamp(DateTime<Utc>),
  Null,
}

impl From<String> for SqlParam {
  fn from(value: String) -> Self {
    SqlParam::Text(value)
  }
}

impl From<&str> for SqlParam {
  fn from(value: &str) -> Self {
    SqlParam::Text(value.to_string())
  }
}

impl From<i64> for SqlParam {
  fn from(value: i64) -> Self {
    SqlParam::Int(value)
  }
}

impl From<i32> for SqlParam {
  fn from(value: i32) -> Self {
    SqlParam::Int(value as i64)
  }
}

impl From<bool> for SqlParam {
  fn from(value: bool) -> Self {
    SqlParam::Bool(value)
  }
}

impl From<f64> for SqlParam {
  fn from(value: f64) -> Self {
    SqlParam::Float(value)
  }
}

impl From<DateTime<Utc>> for SqlParam {
  fn from(value: DateTime<Utc>) -> Self {
    SqlParam::Timestamp(value)
  }
}

impl<T: Into<SqlParam>> From<Option<T>> for SqlParam {
  fn from(value: Option<T>) -> Self {
    match value {
      Some(value) => value.into(),
      None => SqlParam::Null,
    }
  }
}

/// Builds and runs `INSERT INTO <table> (<columns>) VALUES (...) RETURNING <returning>`,
/// so any service can insert a row without hand-writing the query and bind chain.
///
/// `executor` accepts either a `&PgPool` or a transaction (`&mut *tx`), so the insert
/// can optionally run as part of a larger transaction.
///
/// `columns` and `values` must be the same length and in the same order, e.g.
/// `db::insert::<Sample>(&pool, "sample_table", &["name"], vec![name.into()], "id, name, created_at")`.
pub async fn insert<'c, T, E>(
  executor: E,
  table: &str,
  columns: &[&str],
  values: Vec<SqlParam>,
  returning: &str,
) -> Result<T, sqlx::Error>
where
  T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
  E: Executor<'c, Database = Postgres>,
{
  assert_eq!(columns.len(), values.len(), "columns and values must be the same length");

  let mut builder: QueryBuilder<Postgres> =
    QueryBuilder::new(format!("INSERT INTO {table} ({}) VALUES (", columns.join(", ")));

  let mut separated = builder.separated(", ");
  for value in values {
    match value {
      SqlParam::Text(v) => separated.push_bind(v),
      SqlParam::Int(v) => separated.push_bind(v),
      SqlParam::Bool(v) => separated.push_bind(v),
      SqlParam::Float(v) => separated.push_bind(v),
      SqlParam::Timestamp(v) => separated.push_bind(v),
      SqlParam::Null => separated.push_bind(None::<String>),
    };
  }

  builder.push(format!(") RETURNING {returning}"));

  builder.build_query_as::<T>().fetch_one(executor).await
}

/// Runs `SELECT <columns> FROM <table>`, so any service can fetch every row without
/// hand-writing the query, e.g. `db::find_all::<Sample>(&pool, "sample_table", "id, name, created_at")`.
///
/// `executor` accepts either a `&PgPool` or a transaction (`&mut *tx`), so the query
/// can optionally run as part of a larger transaction.
///
/// `table` and `columns` are developer-supplied identifiers, not user input, so the
/// built string is asserted safe rather than bound as a value.
pub async fn find_all<'c, T, E>(executor: E, table: &str, select: &str) -> Result<Vec<T>, sqlx::Error>
where
  T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
  E: Executor<'c, Database = Postgres>,
{
  let query = sqlx::AssertSqlSafe(format!("SELECT {select} FROM {table}"));
  sqlx::query_as::<_, T>(query).fetch_all(executor).await
}

/// Binds a single dynamically typed value onto a query builder in place.
fn bind_param(builder: &mut QueryBuilder<Postgres>, value: SqlParam) {
  match value {
    SqlParam::Text(v) => builder.push_bind(v),
    SqlParam::Int(v) => builder.push_bind(v),
    SqlParam::Bool(v) => builder.push_bind(v),
    SqlParam::Float(v) => builder.push_bind(v),
    SqlParam::Timestamp(v) => builder.push_bind(v),
    SqlParam::Null => builder.push_bind(None::<String>),
  };
}

/// Runs `SELECT <select> FROM <table> WHERE <search_by> = <value>`, returning at most
/// one row. Lets any service look up a single record by a unique column (e.g. `id`)
/// without hand-writing the query, e.g.
/// `db::find_one::<Sample>(&pool, "sample_table", "id, name, created_at", "id", id.into())`.
///
/// `executor` accepts either a `&PgPool` or a transaction (`&mut *tx`), so the query
/// can optionally run as part of a larger transaction.
pub async fn find_one<'c, T, E>(
  executor: E,
  table: &str,
  select: &str,
  search_by: &str,
  value: SqlParam,
) -> Result<Option<T>, sqlx::Error>
where
  T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
  E: Executor<'c, Database = Postgres>,
{
  let mut builder: QueryBuilder<Postgres> =
    QueryBuilder::new(format!("SELECT {select} FROM {table} WHERE {search_by} = "));

  bind_param(&mut builder, value);

  builder.build_query_as::<T>().fetch_optional(executor).await
}

/// Builds and runs `UPDATE <table> SET <columns> = <values> WHERE <search_by> = <search_value>
/// RETURNING <returning>`, returning the updated row, or `None` when no row matched.
/// Lets any service update a row without hand-writing the query and bind chain, e.g.
/// `db::update::<Sample>(&pool, "sample_table", &["name"], vec![name.into()], "id", id.into(), "id, name, created_at")`.
///
/// `executor` accepts either a `&PgPool` or a transaction (`&mut *tx`), so the update
/// can optionally run as part of a larger transaction.
pub async fn update<'c, T, E>(
  executor: E,
  table: &str,
  columns: &[&str],
  values: Vec<SqlParam>,
  search_by: &str,
  search_value: SqlParam,
  returning: &str,
) -> Result<Option<T>, sqlx::Error>
where
  T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
  E: Executor<'c, Database = Postgres>,
{
  assert_eq!(columns.len(), values.len(), "columns and values must be the same length");

  let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(format!("UPDATE {table} SET "));

  for (i, (column, value)) in columns.iter().zip(values).enumerate() {
    if i > 0 {
      builder.push(", ");
    }
    builder.push(format!("{column} = "));
    bind_param(&mut builder, value);
  }

  builder.push(format!(" WHERE {search_by} = "));
  bind_param(&mut builder, search_value);

  builder.push(format!(" RETURNING {returning}"));

  builder.build_query_as::<T>().fetch_optional(executor).await
}
