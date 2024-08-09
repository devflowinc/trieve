# Use an official Python runtime as a parent image
FROM rust:1.78

# Set the working directory in the container
WORKDIR /app

COPY src/ src/
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release

# Run script.py when the container launches
CMD ["./target/release/clustering-script"]
