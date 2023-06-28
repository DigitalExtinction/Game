# DE Lobby Server

Multiplayer games are managed and initiated via DE Lobby Server. The lobby
server implements a simple HTTP API for user and game management.

The HTTP API is documented with Open API Specification:
[openapi.yaml](openapi.yaml).

## Configuration

The server is configured via environment variables:

* `DE_DB_URL` (required) – SQLite database URL passed to `SqliteConnectOptions`
  of `sqlx` Rust library.
* `DE_JWT_SECRET` (required) – A Base64 encoded secret used for signing and
  validation of JSON Web Tokens (JWT). The secret must have between 12 and 86
  characters.

  Make sure to invalidate all JWT by changing the secret after any changes or
  purges of the database.
* `DE_HTTP_PORT` (optional) – HTTP server port number. Defaults to `8080`.
* `RUST_LOG` (optional) – logging configuration, see [env_logger
  documentation](https://docs.rs/env_logger/latest/env_logger/#enabling-logging).
