# Text-Gen Proxy + Backend
# Bundles the Rust proxy with Ollama backend

FROM rust:1.83-bookworm AS builder

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Runtime image with CUDA support
FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

WORKDIR /app

# Copy Rust binary
COPY --from=builder /build/target/release/dev-text-gen /app/proxy

# Environment defaults
ENV API_HOST=0.0.0.0
ENV API_PORT=7103
ENV OLLAMA_URL=http://localhost:11434
ENV MODEL_NAME=gemma3:27b
ENV BACKEND_CMD=ollama
ENV BACKEND_ARGS="serve" --host 0.0.0.0 --port 11434""
ENV BACKEND_WORKDIR=/app
ENV BACKEND_PORT=11434
ENV BACKEND_HEALTH_PATH=/api/tags
ENV PRELOAD=true

EXPOSE 7103 11434

CMD ["/app/proxy"]
