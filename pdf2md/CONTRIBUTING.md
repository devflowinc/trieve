# Contributing to PDF2MD

## Setup ENV's

```bash
cd server
cp .env.dist ./server/.env
```

You will need to replace `LLM_API_KEY` with your key for OpenRouter, OpenAI, LiteLLM, or whichever OpenAI compliant API you are using with the `LLM_BASE_URL`.

If you want to support Chunkr then you can get an API key for their service from [chunkr.ai](https://chunkr.ai) and set it as the value for `CHUNKR_API_KEY`.

## Run dependency services

This will start MinIO S3, Clickhouse, and Redis.

```bash
docker compose --profile dev up -d
```

## Run Server + Workers

Strongly recommend using tmux or another multiplex system to handle the different proceses.

```bash
cargo watch -x run
cargo run --bin supervisor-worker
cargo run --bin chunk-worker
```

## CLI

Make your changes then use the following to run:

```bash
cd cli
cargo run -- help
```

## Run tailwindcss server for demo UI

```
npx tailwindcss -i ./static/in.css -o ./static/output.css --watch
```
