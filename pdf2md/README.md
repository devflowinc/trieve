# Contributing to PDF2MD

## Project Setup

### Setup ENV's

```bash
cd server
cp .env.dist .env
```

### Start docker dependency services

- redis
- s3
- clickhouse-db

```bash
docker compose up -d
```

### Run Server + Workers

Strongly recommend using tmux or another multiplex system to handle the different proceses.

```bash
cargo watch -x run #HTTP server
cargo run --bin supervisor-worker
cargo run --bin chunk-worker
```

### Run tailwindcss server for demo UI

```
npx tailwindcss -i ./static/in.css -o ./static/output.css --watch
```

### Testing using the CLI

Make your changes then use the following to run:

```bash
cd cli
cargo run -- help #or other command instead of help
```

## Deploying 

### docker-compose

```bash
docker compose up -f docker-compose-prod.yaml -d
```

You can either chose to build locally or pull the pre-built images from the docker hub.

#### Build Options
##### Build On Machine:

```bash
docker compose up -f docker-compose-prod.yaml -d --build
```

##### Use Pre-built Images:
```bash
docker compose up -f docker-compose-prod.yaml -d --pull always
```

#### Setup Caddy reverse proxy (optional)

Setup a Caddyfile with the following content:

```bash
# Global options
{
    email developer@example.com
}

# Define a site block for pdftomd.example.com
pdftomd.example.com {
    reverse_proxy localhost:8081
}
```

Start the caddy reverse proxy. This should also handle your ssl

```bash
sudo systemctl reload caddy.service
```
