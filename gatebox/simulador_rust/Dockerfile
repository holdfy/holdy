# Multi-stage Dockerfile para simulador_rust
# Uso: docker build --build-arg PACKAGE=seventrust -t seventrust-rust .
#      docker build --build-arg PACKAGE=sulcred -t sulcred-rust .
#      docker build --build-arg PACKAGE=client-simulator -t client-simulator-rust .
ARG PACKAGE=seventrust
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copiar manifests (Cargo.lock opcional - gerado se não existir)
COPY Cargo.toml ./
COPY seventrust/Cargo.toml ./seventrust/
COPY sulcred/Cargo.toml ./sulcred/
COPY client-simulator/Cargo.toml ./client-simulator/
RUN cargo generate-lockfile 2>/dev/null || true

# Copiar código fonte
COPY seventrust ./seventrust
COPY sulcred ./sulcred
COPY client-simulator ./client-simulator

# Build release
RUN cargo build --release -p ${PACKAGE}

# Imagem final
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

ARG PACKAGE=seventrust
COPY --from=builder /app/target/release/${PACKAGE} /usr/local/bin/

# Binário (ENV persiste no build)
ENV BINARY_NAME=${PACKAGE}

EXPOSE 7010 7020 7070

CMD ["/bin/sh", "-c", "exec /usr/local/bin/$BINARY_NAME"]
