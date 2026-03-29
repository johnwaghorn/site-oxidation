# Error Taxonomy

This document defines how errors should be represented across the backend.

## Why This Exists

Different layers have different needs:

- Startup/wiring needs rich context and fast failure.
- API handlers need stable HTTP responses.
- Domain/internal modules need typed errors with clear semantics.

Using one error style everywhere makes code either too loose or too verbose.

## Layer 1: Startup / Bootstrap Errors

**Where:** `main`, config loading, database init, app bootstrap wiring.

**Type:** `anyhow::Result<T>`

**Goal:** fail fast with helpful context for operators/developers.

**Patterns:**

- Use `?` for propagation.
- Add `.context(...)` / `.with_context(...)` at important boundaries.
- Prefer returning `Result` over `expect`/`panic!` in startup paths.

**Example:**

```rust
let config = AppConfig::from_env().context("Failed to load app conf from env")?;
let pool = db::init_db(&config.database_path)
    .await
    .with_context(|| format!("Could not initialise database {}", config.database_path))?;
```

## Layer 2: API Boundary Errors

**Where:** request handlers, extractors, middleware responses.

**Type:** explicit API response type (`ApiErrorResponse`)

**Goal:** keep HTTP status codes and response shape stable and intentional.

**Patterns:**

- Return `Result<..., ApiErrorResponse>` from handlers.
- Convert internal errors into API-safe messages.
- Log internal details server-side; avoid leaking internals in response bodies.
- Do not replace API boundary types with `anyhow::Error`.

**Example:**

```rust
let sites = sqlx::query_as::<_, SiteResponse>("SELECT ...")
    .fetch_all(&state.pool)
    .await
    .map_err(|e| internal_err("Failed to fetch sites", e))?;
```

## Layer 3: Domain / Internal Typed Errors

**Where:** reusable modules (auth backend, networking helpers, services).

**Type:** typed enums/structs with `thiserror`.

**Goal:** model error cases explicitly and keep call sites clear.

**Patterns:**

- Define module-level error enums with `#[derive(thiserror::Error)]`.
- Use #[from] when the conversion is unambiguous (one source type per enum). Avoid it when multiple variants wrap the
same source type, or when you need to add context at the conversion point.
- Keep domain meanings explicit (avoid stringly-typed errors).

**Example:**

```rust
#[derive(Debug, thiserror::Error)]
enum ResolverError {
    #[error("resolved to private IP")]
    PrivateIpBlocked,
}
```
