# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a FHIR (Fast Healthcare Interoperability Resources) server implementation with two main components:
1. **db**: A PostgreSQL extension written in Rust using pgrx that provides FHIR-specific functions directly in Postgres
2. **api**: An Axum-based REST API server that exposes the database functionality via HTTP

The architecture follows a pattern where the database extension handles core FHIR operations (validation, storage, indexing) while the API provides HTTP endpoints.

## Development Environment

The project uses Nix flakes for reproducible development environments. Enter the dev shell with:
```bash
nix develop
```

This provides:
- Rust toolchain via rust-overlay
- PostgreSQL 18 (`PG_VERSION=pg18`)
- cargo-pgrx for extension development
- bacon for live-reload development
- sqlx-cli for database migrations

## Building and Testing

### Database Extension (db)

Initialize pgrx (first time only):
```bash
cargo pgrx init
```

Run the extension in a pgrx-managed Postgres instance with psql:
```bash
cargo pgrx run --package fhir
```

Run extension tests:
```bash
cargo pgrx test --package fhir
```

Install the extension to a specific Postgres instance:
```bash
cargo pgrx install --package fhir
```

### API Server (api)

Run the API server with live-reload:
```bash
bacon run-api
```

Or run directly:
```bash
cargo run --package api
```

The API server requires these environment variables:
- `FHIR_DATABASE_URL`: PostgreSQL connection string (default from flake: `postgresql://localhost:28818/fhir`)
- `PORT`: HTTP server port (default: 3100)
- `FHIR_ENV`: Environment mode (`development` or `production`)

Configuration can also be loaded from a TOML file specified by `FHIR_CONFIG_FILE`.

### Running Tests

Run all tests:
```bash
cargo test
```

Run tests for specific package:
```bash
cargo test --package fhir
cargo test --package api
```

## Architecture Details

### Database Extension Architecture

The extension is structured around these key concepts:

1. **Core Tables** (db/src/schema.rs):
   - `fhir.entity`: Stores FHIR resources with UUID, resource_type, and JSONB data
   - `fhir.entity_history`: Audit log with operation tracking (insert/update/delete) and JSON diffs
   - `fhir.entity_index_text`: Full-text search index using pg_trgm
   - `fhir.entity_index_date`: Date-based search index

2. **FHIR Functions** (db/src/api/):
   - `fhir_put(jsonb)`: Insert FHIR resource, returns UUID
   - `fhir_get(entity_type, uuid)`: Retrieve FHIR resource by ID
   - `fhir_search(entity, key, operator, value)`: Search using indexed parameters
   - All functions are exposed as PostgreSQL functions callable via SQL

3. **Indexing System** (db/src/index/):
   - Resource-specific index extraction (currently Patient)
   - Automatically extracts searchable values during `fhir_put`
   - Supports text (with trigram) and date indexes
   - Each resource type has its own module (e.g., `index/patient.rs`) defining which fields to index

4. **Schema Validation** (db/src/fhir/mod.rs):
   - Full FHIR JSON schema validation using `jsonschema` crate
   - Schema is compiled once per thread and cached in thread-local storage
   - Validation is enforced via Postgres CHECK constraint on the entity table

5. **History Tracking** (db/src/hooks.rs):
   - PostgreSQL trigger automatically logs all changes
   - Captures JSON diffs showing removed/changed/added values
   - Uses fastrace for distributed tracing integration

### API Server Architecture

1. **Router** (api/src/routes/mod.rs):
   - Uses utoipa-axum for OpenAPI documentation generation
   - Scalar UI available at `/docs`
   - All routes are type-safe with utoipa annotations

2. **Database Integration**:
   - Uses sqlx for async Postgres queries
   - Calls the extension functions via raw SQL
   - Connection pool configured in main.rs

3. **Observability**:
   - OpenTelemetry tracing with init-tracing-opentelemetry
   - Custom error logging middleware
   - Structured logging with tracing crate

### Tracing System

Both components support distributed tracing via Jaeger:

**Extension tracing** (enabled in postgresql.conf):
```
fhir.jaeger_enabled = 'true'
fhir.jaeger_host = '127.0.0.1:6831'
```

**API tracing**: Configured via environment (production vs development mode)

Start Jaeger for local development:
```bash
docker compose up -d
```

Access Jaeger UI at http://localhost:16686

### Adding New FHIR Resource Types

To support a new resource type (e.g., Observation):

1. Add Rust model in `db/src/models.rs` (only fields you need to access in Rust)
2. Create index module `db/src/index/observation.rs` with:
   - `text_index_values_for()`: Extract searchable text fields
   - `date_index_values_for()`: Extract searchable date fields
   - `find_search_index_for_key()`: Map search parameter names to index types
3. Wire up in `db/src/index/mod.rs` to route to your new module
4. The schema validation already supports all FHIR resources

## Code Style and Patterns

- The db crate enforces `#![deny(clippy::pedantic)]` but allows `needless_pass_by_value` because pgrx requires value passing
- All database operations in the extension use the SPI (Server Programming Interface) via the custom `spi` module wrapper
- Extension functions use `#[pg_extern]` attribute and are automatically exposed to Postgres
- Use `#[trace]` attribute from fastrace on functions you want to trace
- API routes use `#[utoipa::path]` annotations for OpenAPI documentation
- Error handling: Extension uses `panic!` for errors (Postgres convention), API uses `eyre::Result`

## Performance Notes

- First FHIR entity insert is slow (~600ms) due to JSON schema compilation
- Subsequent inserts take ~3-4ms
- The schema is compiled lazily on first validation call
- Consider pre-compiling during extension initialization for production use
- Trigram indexes provide fast partial-match text searches but require pg_trgm extension
