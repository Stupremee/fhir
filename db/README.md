# FHIR PostgreSQL Extension Docker Image

This directory contains the Dockerfile for building a PostgreSQL 18 image with the FHIR extension pre-installed.

## Building the Image

From the repository root:

```bash
docker build -t fhir-postgres -f db/Dockerfile .
```

## Running the Container

### Standalone

```bash
docker run -d \
  --name fhir-db \
  -p 5432:5432 \
  -e POSTGRES_USER=fhir \
  -e POSTGRES_PASSWORD=fhir \
  -e POSTGRES_DB=fhir \
  -v fhir_data:/var/lib/postgresql/data \
  fhir-postgres
```

### With Docker Compose

Use the provided compose file:

```bash
docker compose -f docker-compose.db.yaml up -d
```

This starts the complete stack:
- PostgreSQL 18 with FHIR extension
- FHIR API server
- Jaeger tracing UI

## Connecting to the Database

```bash
# Connect using psql in the container
docker exec -it fhir-db psql -U fhir -d fhir

# Or from your host (if you have psql installed)
psql -h localhost -p 5432 -U fhir -d fhir
```

## Testing the Extension

Once connected, test that the extension is loaded:

```sql
-- List installed extensions
\dx

-- You should see 'fhir' in the list

-- Test the FHIR functions
SELECT fhir_put('{"resourceType":"Patient","gender":"female"}'::jsonb);
```

## Image Details

The image is built in two stages:

1. **Builder stage**:
   - Installs Rust and cargo-pgrx
   - Compiles the FHIR extension in release mode
   - Installs the extension to the PostgreSQL directories

2. **Runtime stage**:
   - Based on official `postgres:18` image
   - Copies the compiled extension files
   - Automatically creates the extension on first startup

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_USER` | `fhir` | PostgreSQL username |
| `POSTGRES_PASSWORD` | `fhir` | PostgreSQL password |
| `POSTGRES_DB` | `fhir` | Default database name |

## Volumes

- `/var/lib/postgresql/data` - PostgreSQL data directory (should be mounted for persistence)

## Extension Configuration

The extension supports configuration via PostgreSQL GUCs:

```sql
-- Enable Jaeger tracing
ALTER SYSTEM SET fhir.jaeger_enabled = 'true';
ALTER SYSTEM SET fhir.jaeger_host = '127.0.0.1:6831';
SELECT pg_reload_conf();
```

See the main README for more configuration options.

## Build Arguments

The Dockerfile uses `cargo-pgrx` version 0.16.1 to match the extension's dependencies. Make sure this matches the version in `Cargo.toml`.

## Troubleshooting

### Extension not found

If you get "extension not found" errors:

```bash
# Check if extension files are present
docker exec fhir-db ls -la /usr/share/postgresql/18/extension/fhir*
docker exec fhir-db ls -la /usr/lib/postgresql/18/lib/fhir.so
```

### Build failures

If the build fails during compilation:

1. Ensure you have enough memory allocated to Docker (at least 4GB recommended)
2. Check that the workspace Cargo.toml is correctly structured
3. Verify that the pgrx version matches between Cargo.toml and the Dockerfile

### Performance

The builder stage can take 10-20 minutes depending on your system. Subsequent builds with cached layers will be much faster.

## Production Considerations

1. **Security**: Change default passwords in production
2. **Persistence**: Always mount a volume for `/var/lib/postgresql/data`
3. **Resources**: The build requires significant CPU and memory
4. **Updates**: Rebuild the image when updating the extension code

## Size Optimization

The final image size is approximately 400-500MB, which includes:
- PostgreSQL 18 runtime (~200MB)
- System libraries (~100MB)
- FHIR extension (~50-100MB)

To reduce size further, consider:
- Using Alpine-based PostgreSQL images (requires additional setup)
- Stripping debug symbols from the extension
- Multi-stage builds to minimize layer count
