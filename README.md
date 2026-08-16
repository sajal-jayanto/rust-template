# rust_learning

A small Axum + sqlx (Postgres) API.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [PostgreSQL](https://www.postgresql.org/download/) running and reachable
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli), for running migrations manually:

  ```sh
  cargo install sqlx-cli --no-default-features --features postgres,rustls
  ```

## Install

1. Clone the repo and enter it:

   ```sh
   git clone <repo-url>
   cd rust_learning
   ```

2. Copy the env example and fill in your local Postgres credentials:

   ```sh
   cp .env.example .env
   ```

3. Create the database (if it doesn't exist yet):

   ```sh
   sqlx database create
   ```

4. Build the project:

   ```sh
   cargo build
   ```

## Running

```sh
cargo run
```

On startup the app connects to Postgres and runs any pending migrations automatically
(see `db::setup` in [src/db/mod.rs](src/db/mod.rs)) — a manual `sqlx migrate run` isn't
required to boot the app, but is still useful for inspecting/applying migrations ahead
of time.

The server listens on `127.0.0.1:3000`.

## Database migrations

Migrations live in [migrations/](migrations) and are managed with `sqlx-cli`. Run these
from the project root (`DATABASE_URL` is read from `.env`):

| Command | Purpose |
| --- | --- |
| `sqlx migrate add -r <description>` | Create a new up/down migration pair |
| `sqlx migrate run` | Apply all pending "up" migrations |
| `sqlx migrate revert` | Revert the most recently applied migration |
| `sqlx migrate info` | Show which migrations have been applied |

## API

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/health` | Health check (includes DB connectivity) |
| GET | `/api/v1/sample` | List all samples |
| GET | `/api/v1/sample/{id}` | Get a sample by id |
| POST | `/api/v1/sample` | Create a sample |
| PUT | `/api/v1/sample/{id}` | Update a sample |
