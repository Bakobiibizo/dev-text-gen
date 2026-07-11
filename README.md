# dev-text-gen

A small Rust control proxy in front of an official Ollama container. It provides a stable local port, model readiness, optional model preload, optional bearer authentication, native Ollama generation, and streamed OpenAI-compatible passthrough.

The repository does not vendor or rebuild Ollama.

## Requirements

- Docker with the Compose plugin
- Optional NVIDIA Container Toolkit for GPU execution

## Start on CPU

```bash
cp .env.example .env
docker compose up --build -d
```

## Start with NVIDIA GPU

```bash
cp .env.example .env
docker compose -f docker-compose.yml -f docker-compose.gpu.yml up --build -d
```

The API binds to `127.0.0.1:7103` by default. Set `API_TOKEN` in `.env` before publishing it through another interface or reverse proxy.

## Model lifecycle

The default model is `gemma3:1b`. Model data persists in the `ollama-models` volume.

Pull the configured model explicitly:

```bash
curl -X POST http://127.0.0.1:7103/pull
```

Set `PRELOAD=true` to pull it automatically at proxy startup. Large models can take considerable time and storage, so preload is disabled by default.

Readiness returns HTTP 200 only when Ollama is reachable and the configured model is installed:

```bash
curl http://127.0.0.1:7103/health
curl -i http://127.0.0.1:7103/ready
```

## Generate

Native Ollama request:

```bash
curl http://127.0.0.1:7103/generate \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"Write one sentence about rain.","stream":false}'
```

OpenAI-compatible request:

```bash
curl http://127.0.0.1:7103/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"gemma3:1b","messages":[{"role":"user","content":"Hello"}],"stream":true}'
```

When `API_TOKEN` is configured, add `Authorization: Bearer <token>` to `/pull`, `/generate`, and all `/v1/*` requests. Health and readiness remain unauthenticated for container orchestration.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `API_PORT` | `7103` | Host port for the proxy |
| `API_TOKEN` | empty | Optional bearer token |
| `MODEL_NAME` | `gemma3:1b` | Model used by pull/readiness/generate |
| `PRELOAD` | `false` | Pull the configured model at startup |
| `MAX_BODY_BYTES` | `10485760` | Maximum OpenAI request body size |
| `OLLAMA_IMAGE` | `ollama/ollama:0.31.1` | Official backend image |
| `OLLAMA_KEEP_ALIVE` | `5m` | Ollama model residency period |

## Verify

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
docker compose config
docker build -t dev-text-gen:test .
```
