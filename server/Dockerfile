FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --features "runtime-env"

FROM python:3.9.18-bookworm as runtime
WORKDIR /app
COPY --from=builder /app/target/release/vault-server /app/vault-server
COPY ./server-python/ /app/server-python
COPY ./migrations/ /app/migrations

RUN pip install -r /app/server-python/requirements.txt

RUN rm -rf tmp
RUN mkdir tmp

RUN apt-get update && apt-get install -y libpq-dev pkg-config build-essential

EXPOSE 8090
ENTRYPOINT ["/app/vault-server"]
