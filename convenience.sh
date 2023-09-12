#!/bin/bash

# Function to reset the Qdrant database
reset_qdrant_database() {
    echo "Resetting the Qdrant database..."
    docker compose stop vault-server
    docker compose stop qdrant-database
    docker compose rm -f qdrant-database
    docker volume rm vault-server_qdrant_data
    docker compose up -d qdrant-database
    diesel db reset
    docker compose up -d vault-server
}

reset_s3_service() {
    echo "Resetting the S3 service..."
    docker compose stop s3
    docker compose rm -f s3
    docker volume rm vault_s3-data
    docker compose up -d s3
}

# Function to set up the Python environment
setup_python_environment() {
    echo "Setting up the Python environment..."
    virtualenv venv
    source venv/bin/activate
    pip install -r ./vault-python/requirements.txt
}

# Function to reset the script database
reset_script_redis() {
    echo "Resetting the script Redis database..."
    docker compose stop script-redis
    docker compose rm -f script-redis
    docker volume rm vault_script-redis-data
    docker compose up -d script-redis
}

# Main script logic
while getopts ":qps3" opt; do
    case $opt in
        q)
            reset_qdrant_database
            ;;
        p)
            setup_python_environment
            ;;
        3)
            reset_s3_service
            ;;
        s)
            reset_script_redis
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

