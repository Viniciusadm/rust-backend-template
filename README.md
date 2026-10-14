<!-- {% comment %} -->
# rust-backend-template

A [cargo-generate](https://github.com/cargo-generate/cargo-generate) template for Rust HTTP backends with a hexagonal (ports and adapters) layout: a pure domain, use cases that reach I/O only through traits, and concrete adapters (axum, sqlx with MySQL, JWT, Argon2) wired at the edges. It ships with one working slice, user registration and login.

## Generate a project

Install cargo-generate once, then generate from this repository:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/Viniciusadm/rust-backend-template.git --name my-service
```

`--name my-service` sets `project-name` (`my-service`) and `crate_name` (`my_service`), which fill `Cargo.toml`, `.env.example`, `src/main.rs` and the tests. This README is also a template: in the generated project its title becomes the project name and this section is removed.
<!-- {% endcomment %} -->
<!-- {{ "--" }}>
# {{project-name}}

Rust HTTP backend generated from [rust-backend-template](https://github.com/Viniciusadm/rust-backend-template), with a hexagonal (ports and adapters) layout: a pure domain, use cases that reach I/O only through traits, and concrete adapters (axum, sqlx with MySQL, JWT, Argon2) wired at the edges.
<!-- -->

## Requirements

- Rust **1.94+** (edition 2024, required by sqlx 0.9)
- MySQL 8 for the endpoints that persist data

Main crates: axum 0.8, sqlx 0.9 (MySQL, rustls), jsonwebtoken 11 (`rust_crypto` backend), argon2 0.6, tower-http 0.7.

## Run locally

Copy the example environment and set `DATABASE_URL` and `JWT_SECRET`:

```bash
cp .env.example .env
```

Start the API. Without `DATABASE_URL` it still starts, but the endpoints that need the database answer 503. Migrations in [`migrations`](migrations) run on boot unless `RUN_MIGRATIONS` is off.

```bash
cargo run
```

Endpoints of the example slice:

| Method and path | What it does |
| --- | --- |
| `GET /health` | Health check |
| `POST /api/v1/auth/register` | Creates a user |
| `POST /api/v1/auth/login` | Returns a JWT |
| `GET /api/v1/auth/me` | Returns the current user (`Authorization: Bearer <token>`) |

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `HTTP_HOST` | `127.0.0.1` | Bind address |
| `HTTP_PORT` | `3001` | Port |
| `DATABASE_URL` | none | MySQL connection; without it there is no persistence |
| `JWT_SECRET` | `development-only-secret` | JWT signing secret; change it in production |
| `RUN_MIGRATIONS` | `true` | Runs migrations on boot; `0`, `false` or `no` turn it off |

## Architecture

Dependencies point inward: `api` depends on `application`, which depends on `domain`. `infrastructure` implements the traits (ports) declared in `application`, and handlers build their use case per request.

| Path | What it is |
| --- | --- |
| [`src/domain`](src/domain) | Pure types and rules, with no axum, sqlx or serde: value objects (`DomainId`, `Email`, `NonEmptyString`, `UtcDateTime`) and the `User` entity. |
| [`src/application`](src/application) | Use cases and ports. `auth` declares `UserRepository`, `PasswordHasher`, `TokenIssuer` and `TokenVerifier`, and `AuthUseCases` is generic over them. |
| [`src/infrastructure`](src/infrastructure) | Adapters that implement the ports: `Argon2PasswordHasher`, `JwtTokenService` and `MySqlUserRepository` with its row mapper. |
| [`src/database`](src/database) | Connection pool and transactions. |
| [`src/config`](src/config) | `AppConfig::from_env()`. |
| [`src/errors`](src/errors) | `AppError`, the single error type, with stable codes. |
| [`src/api`](src/api) | HTTP layer: `AppState` (only the pool and the JWT secret), DTOs, the `AuthenticatedUser` extractor and the routes. |

Rules the layout follows:

- No global state or service locator. `AppState` holds only infrastructure; each handler builds its use case.
- The domain does not know the framework.
- Every error goes through `AppError` and is mapped to HTTP in `app_error()`.
- Use cases are generic over their ports, so they are tested with fakes and no database.

## Adding a feature

A new feature (for example `products`) repeats the same layers:

1. Application: `src/application/products/` with `ports.rs` (the `ProductRepository` trait), `commands.rs` (input and output DTOs), `use_cases.rs` (`ProductUseCases<R>`) and `mod.rs`; register it in `src/application/mod.rs`.
2. Domain: `src/domain/products/` with its entities; new value objects go in `src/domain/value_objects.rs`.
3. Infrastructure: `src/infrastructure/repositories/mysql_products.rs` implements the trait, with row conversion in `mapper.rs`.
4. API: `src/api/routes/v1/products.rs` with `router()` and the handlers, merged in `src/api/routes/v1/mod.rs`; request DTOs in `src/api/dto/products.rs` with `From<Request> for Command`.
5. Tests: `tests/products_use_cases.rs` with fakes of the ports, following `tests/auth_use_cases.rs`.

## Tests

```bash
cargo test
```

[`tests/auth_use_cases.rs`](tests/auth_use_cases.rs) covers registration (success, duplicate email, short password) and login with in-memory fakes of the ports; it needs no database.
