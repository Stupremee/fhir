# Dockerfile for building PostgreSQL 18 with the FHIR extension
# This creates a PostgreSQL image with the pgrx-based FHIR extension installed

FROM postgres:18 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    postgresql-server-dev-18 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# Install cargo-pgrx
RUN cargo install cargo-pgrx --version 0.16.1 --locked

# Initialize pgrx for PostgreSQL 18
RUN cargo pgrx init --pg18=/usr/lib/postgresql/18/bin/pg_config

# Set working directory
WORKDIR /build

# Copy the db extension source
COPY . .

# Build the extension in release mode
WORKDIR /build/db
RUN cargo pgrx install --release --pg-config=/usr/lib/postgresql/18/bin/pg_config

# Runtime stage - PostgreSQL with extension installed
FROM postgres:18

# Copy the installed extension from builder
COPY --from=builder /usr/share/postgresql/18/extension/fhir* /usr/share/postgresql/18/extension/
COPY --from=builder /usr/lib/postgresql/18/lib/fhir.so /usr/lib/postgresql/18/lib/

# Set environment variables
ENV POSTGRES_USER=fhir \
    POSTGRES_PASSWORD=fhir \
    POSTGRES_DB=fhir

# Copy initialization script to create the extension
RUN echo "CREATE EXTENSION IF NOT EXISTS pg_trgm;\nCREATE EXTENSION IF NOT EXISTS fhir;" > /docker-entrypoint-initdb.d/01-create-extension.sql


# Expose PostgreSQL port
EXPOSE 5432

# Use the default postgres entrypoint
CMD ["postgres"]
