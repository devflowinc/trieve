# Start with a small base image
FROM golang:1.24-alpine AS builder

# Install git and build essentials
RUN apk add --no-cache git gcc musl-dev

# Set working directory
WORKDIR /app

# Copy go.mod and go.sum first to leverage Docker cache
COPY go.mod go.sum* ./

# Download dependencies (if go.sum exists)
RUN if [ -f go.sum ]; then go mod download; else go mod tidy; fi

# Copy source code
COPY *.go ./

# Build the application with optimizations
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -ldflags="-s -w" -o collapse_queries .

# Use a minimal alpine image for the final container
FROM alpine:3.18

# Add CA certificates for HTTPS
RUN apk --no-cache add ca-certificates tzdata

# Set working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/collapse_queries .

# Copy the .env file if it exists (optional, can be mounted)
COPY .env* ./

# Create a non-root user and set permissions
RUN adduser -D -H -h /app appuser && \
    chown -R appuser:appuser /app

USER appuser

# Command to run the application
ENTRYPOINT ["/app/collapse_queries"]