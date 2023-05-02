FROM rust:slim-buster

RUN apt-get update && \
  apt-get -y upgrade && \
  apt-get -y install libpq-dev libssl-dev pkg-config

WORKDIR /app
COPY . /app/
COPY .env.docker /app/.env

RUN cargo build --release

EXPOSE 8090

ENTRYPOINT ["/bin/bash", "-c", "cargo run --release"]
