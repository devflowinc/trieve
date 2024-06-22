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

# Main script logic
while getopts ":qps3l" opt; do
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
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

