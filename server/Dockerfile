FROM rust:1 AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --features "runtime-env"

FROM node:alpine as node-builder
WORKDIR /app
COPY ./server-nodejs/yarn.lock /app/server-nodejs/yarn.lock
COPY ./server-nodejs/package.json /app/server-nodejs/package.json
RUN yarn --cwd ./server-nodejs install

# Use Ubuntu 22.04 as the base image
FROM node:18-bookworm as runtime

WORKDIR /app
COPY --from=builder /app/target/release/vault-server /app/vault-server
COPY ./server-nodejs/yarn.lock /app/server-nodejs/yarn.lock
COPY ./server-nodejs/package.json /app/server-nodejs/package.json
# Install yarn
COPY ./server-nodejs/ /app/server-nodejs
COPY --from=node-builder /app/server-nodejs/node_modules /app/server-nodejs/node_modules
COPY ./migrations/ /app/migrations

RUN mkdir tmp

RUN apt-get update && apt-get install -y libpq-dev pkg-config build-essential

EXPOSE 8090
ENTRYPOINT ["/app/vault-server"]
