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
RUN cargo build --release --no-default-features

# Use Ubuntu 22.04 as the base image
FROM node:18-alpine3.18 as runtime

COPY --from=builder /app/target/release/vault-server /usr/local/bin
RUN apk upgrade --no-cache && apk add --no-cache ca-certificates

WORKDIR /app
COPY . /app/
COPY .env.dist /app/.env

# Install yarn
RUN yarn --cwd ./server-nodejs install

RUN rm -rf ./tmp
RUN mkdir tmp

EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/vault-server"]
