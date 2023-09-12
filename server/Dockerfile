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
RUN cargo build --release

# Use Ubuntu 22.04 as the base image
FROM node:18 as runtime

COPY --from=builder /app/target/release/vault-server /usr/local/bin
# Update package list and install required dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gcc \
    g++ \
    make \
    pkg-config \
    python3 \
    python3-pip \
    python-is-python3 \
    libpq-dev \
    libssl-dev \
    pkg-config \
    openssl \
    libreoffice \
    ca-certificates \
    curl \
    gnupg

RUN apt-get update && apt-get -y upgrade

WORKDIR /app
COPY . /app/
COPY .env.dist /app/.env

# Install yarn
RUN yarn --cwd ./vault-nodejs install

RUN pip install -r ./vault-python/requirements.txt --break-system-packages
RUN rm -rf ./tmp
RUN mkdir tmp

EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/vault-server"]
