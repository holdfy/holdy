# Build stage - Rust
FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release (sem feature pulsar por padrão; para Pulsar: docker build --build-arg PULSAR=1)
ARG PULSAR=0
RUN if [ "$PULSAR" = "1" ]; then cargo build --release --features pulsar; else cargo build --release; fi

# Final stage - imagem mínima
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app

WORKDIR /app

COPY --from=builder /app/target/release/gatebox-rust .

RUN chown app:app /app/gatebox-rust

USER app

EXPOSE 8080 2112

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/gatebox-rust"]
