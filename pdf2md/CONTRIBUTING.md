# Contributing to PDF2MD

## Setup ENV's

```bash
cd server
cp .env.dist .env
```

## Run dep processes

```bash
docker compose --profile dev up -d
```

## Run Server + Workers

Strongly recommend using tmux or another multiplex system to handle the different proceses.

```bash
cargo watch -x run #HTTP server
cargo run --bin supervisor-worker
cargo run --bin chunk-worker
```

## CLI

Make your changes then use the following to run:

```bash
cd cli
cargo run -- help #or other command instead of help
```

## Run tailwindcss server for demo UI

```
npx tailwindcss -i ./static/in.css -o ./static/output.css --watch
```
