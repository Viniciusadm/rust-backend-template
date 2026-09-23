# {{project-name}}

Backend Rust com **arquitetura hexagonal (Ports & Adapters)**: domínio puro,
casos de uso desacoplados de I/O via traits, e adaptadores concretos
(axum + sqlx/mysql + jwt + argon2) injetados nas bordas.

> Este projeto foi gerado a partir do template `rust-backend-template`.

## Gerar um novo projeto a partir deste template

```bash
# instalar a ferramenta uma vez
cargo install cargo-generate

# a partir de um caminho local
cargo generate --path /caminho/para/rust-backend-template --name meu-servico

# ou de um repositório Git
cargo generate --git <url-do-repo> --name meu-servico
```

`--name meu-servico` define `project-name` (kebab-case) e `crate_name`
(snake_case, ex.: `meu_servico`) automaticamente.

## Requisitos

- Rust **1.94+** (edition 2024; exigência do sqlx 0.9)
- MySQL 8 para os endpoints com persistência

Stack principal: axum 0.8, sqlx 0.9 (MySQL, rustls), jsonwebtoken 11
(backend `rust_crypto`), argon2 0.6, tower-http 0.7.

## Rodar

```bash
cp .env.example .env          # ajuste DATABASE_URL e JWT_SECRET
cargo test                    # testes de caso de uso (não precisam de banco)
cargo run                     # sobe a API (precisa de MySQL para persistência)
```

Endpoints do slice de exemplo:

- `GET  /health`
- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `GET  /api/v1/auth/me` (requer `Authorization: Bearer <token>`)

## Arquitetura

As dependências apontam **para dentro**: `api → application → domain`. A
infraestrutura implementa os traits (ports) definidos em `application`, e é
injetada nos handlers por request.

```
src/
├── domain/            Regras e tipos puros (sem axum/sqlx/serde)
│   ├── value_objects.rs   DomainId, Email, NonEmptyString, UtcDateTime
│   └── users/entities.rs  struct User
├── application/       Casos de uso + ports (traits)
│   └── auth/
│       ├── ports.rs       UserRepository, PasswordHasher, TokenIssuer, TokenVerifier
│       ├── commands.rs    DTOs de entrada/saída
│       └── use_cases.rs   AuthUseCases<R,H,T> (genérico sobre os ports)
├── infrastructure/    Adaptadores concretos (implementam os ports)
│   ├── auth/              Argon2PasswordHasher, JwtTokenService
│   └── repositories/      MySqlUserRepository + mapper (linha → domínio)
├── database/          pool + transaction
├── config/            AppConfig::from_env()
├── errors/            AppError (enum único) + códigos estáveis
└── api/               Camada HTTP (axum)
    ├── state.rs           AppState (só infra: pool + jwt_secret)
    ├── dto/               Request/Response + From para commands
    ├── extractors/        AuthenticatedUser (FromRequestParts)
    └── routes/            health + v1 (handlers + app_error)
```

Princípios que o template segue (e que você deve manter):

- **Sem estado global / service locator.** O `AppState` guarda só
  infraestrutura. Cada handler constrói seu caso de uso por request.
- **Domínio não conhece framework.** Nada de `serde`/`sqlx`/`axum` em `domain/`.
- **Erros num único `AppError`**, traduzido para HTTP em `app_error()`.
- **Casos de uso genéricos sobre os ports** → testáveis com fakes, sem banco.

## Como adicionar uma feature nova (ex.: `products`)

Replique o mesmo padrão, camada por camada:

1. **Application** — `src/application/products/`
   - `ports.rs`: trait `ProductRepository` (+ records de persistência)
   - `commands.rs`: DTOs de entrada/saída
   - `use_cases.rs`: `ProductUseCases<R>` genérico sobre o(s) port(s)
   - `mod.rs`: `pub use commands/ports/use_cases::*;`
   - registre `pub mod products;` em `src/application/mod.rs`
2. **Domain** — `src/domain/products/{mod.rs,entities.rs}`; value objects novos
   vão em `src/domain/value_objects.rs`. Sem dependências de framework.
3. **Infrastructure** — `src/infrastructure/repositories/mysql_products.rs`
   implementa o trait; conversões de linha → domínio em `mapper.rs`. Registre
   em `repositories/mod.rs`.
4. **API** — `src/api/routes/v1/products.rs` com `router()` + handlers;
   instancie o caso de uso por request e trate erros com `app_error()`. Faça
   `.merge(products::router())` em `src/api/routes/v1/mod.rs`.
5. **DTOs** — `src/api/dto/products.rs` com `From<Request> for Command`.
6. **Tests** — `tests/products_use_cases.rs` com fakes dos ports (veja
   `tests/auth_use_cases.rs` como modelo).

## Variáveis de ambiente

Veja `.env.example`. Resumo:

| Variável         | Default                | Descrição                              |
|------------------|------------------------|----------------------------------------|
| `HTTP_HOST`      | `127.0.0.1`            | Host de bind                           |
| `HTTP_PORT`      | `3001`                 | Porta                                  |
| `DATABASE_URL`   | —                      | Conexão MySQL (sem ela, sem persistência) |
| `JWT_SECRET`     | `development-only-secret` | Segredo de assinatura do JWT        |
| `RUN_MIGRATIONS` | `true`                 | Rodar migrations no boot               |
