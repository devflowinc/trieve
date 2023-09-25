# Use Ubuntu 22.04 as the base image
FROM ubuntu:22.04

# Update package list and install required dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gcc \
    g++ \
    make \
    pkg-config \
    python3 \
    python3-pip

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Clean up APT cache
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . /app/
COPY .env.docker /app/.env

# Install Node.js and Yarn
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g yarn && \
    yarn --cwd ./vault-nodejs install

RUN apt-get update && \
  apt-get -y upgrade && \
  apt-get -y install libpq-dev libssl-dev pkg-config openssl libreoffice

RUN rustup default nightly

RUN pip install -r ./vault-python/requirements.txt

RUN rm -rf ./tmp

RUN mkdir tmp

RUN cargo build --release

EXPOSE 8090

ENTRYPOINT ["/bin/bash", "-c", "cargo run --release"]
