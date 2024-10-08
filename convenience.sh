#!/bin/bash

# Function to reset the Qdrant database
reset_qdrant_database() {
    echo "Resetting the Qdrant database..."
    docker compose stop qdrant-database
    docker compose rm -f qdrant-database
    docker volume rm trieve_qdrant_data
    docker compose up -d qdrant-database
    diesel db reset
}

reset_s3_service() {
    echo "Resetting the S3 service..."
    docker compose stop s3
    docker compose rm -f s3
    docker volume rm vault_s3-data
    docker compose up -d s3
}

start_local_services() {
    echo "Starting local services..."
    docker compose up -d db redis qdrant-database s3 s3-client keycloak keycloak-db tika clickhouse-db
}

build_typescript_client() {
    echo "Building the TypeScript client..."
    cargo run --features runtime-env --manifest-path server/Cargo.toml --bin redoc_ci > ./clients/ts-sdk/openapi.json
    cd ./clients/ts-sdk/; yarn && yarn build:clean;
    echo "Done building the TypeScript client."
}

start_firecrawl() {
    echo "Starting Firecrawl..."
    docker compose -f docker-compose-firecrawl.yml up -d firecrawl-worker firecrawl-api playwright-service redis
}

# Main script logic
while getopts ":qps3lcf" opt; do
    case $opt in
        q)
            reset_qdrant_database
            ;;
        3)
            reset_s3_service
            ;;
        l)
            start_local_services
            ;;
        f)
            start_firecrawl
            ;;
        c)
            build_typescript_client
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

