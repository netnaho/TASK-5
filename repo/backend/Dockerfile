FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Copy manifests and lockfile for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Remove the dummy build artifacts for the main crate only
RUN rm -rf src target/release/campus-learn-backend target/release/deps/campus_learn*

# Copy real source and build
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/campus-learn-backend /app/campus-learn-backend
COPY --from=builder /app/Rocket.toml /app/Rocket.toml
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8000

CMD ["./campus-learn-backend"]
