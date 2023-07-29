#!/bin/bash

# Function to reset the Qdrant database
reset_qdrant_database() {
    echo "Resetting the Qdrant database..."
    sudo docker compose stop qdrant-database
    sudo docker compose rm -f qdrant-database
    sudo docker volume rm vault-server_qdrant_data
    sudo docker compose up -d qdrant-database
    diesel db reset
}

reset_s3_service() {
    echo "Resetting the S3 service..."
    sudo docker compose stop s3
    sudo docker compose rm -f s3
    sudo docker volume rm vault_s3-data
    sudo docker compose up -d s3
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
    sudo docker compose stop script-redis
    sudo docker compose rm -f script-redis
    sudo docker volume rm vault_script-redis-data
    sudo docker compose up -d script-redis
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

