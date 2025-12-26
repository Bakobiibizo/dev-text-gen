# dev-text-gen

Text generation proxy (e.g., Ollama/vLLM/OpenAI-compatible).

## Requirements
- Docker + GPU recommended
- Built/tested on **aarch64**. For x86_64, choose the matching base image tag and rebuild locally.

## Build
```bash
docker build -t inference/text-gen:local .
```

## Run (standalone)
```bash
docker run --gpus all -d -p 7103:7103 inference/text-gen:local
```

## Run with docker-compose (root of repo)
```bash
docker compose up text-gen
```

## Test
Health:
```bash
curl http://localhost:7103/health
```

Chat completion (example payload depends on backend; see proxy docs):
```bash
curl -X POST http://localhost:7103/v1/chat/completions ...
```***
