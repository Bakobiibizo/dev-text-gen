FROM rust:1.83-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 app
COPY --from=builder /build/target/release/dev-text-gen /usr/local/bin/dev-text-gen
USER app
ENV API_HOST=0.0.0.0 \
    API_PORT=7103 \
    OLLAMA_URL=http://ollama:11434 \
    MODEL_NAME=gemma3:1b \
    PRELOAD=false
EXPOSE 7103
ENTRYPOINT ["/usr/local/bin/dev-text-gen"]
