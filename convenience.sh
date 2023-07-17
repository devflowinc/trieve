#!/bin/bash

# Function to reset the Qdrant database
reset_qdrant_database() {
    echo "Resetting the Qdrant database..."
    sudo docker-compose stop qdrant-database
    sudo docker-compose rm -f qdrant-database
    sudo docker volume rm ai-editor_qdrant_data
    sudo docker compose up -d qdrant-database
    diesel db reset
}

# Function to set up the Python environment
setup_python_environment() {
    echo "Setting up the Python environment..."
    virtualenv venv
    source venv/bin/activate
    pip install -r ./python-scripts/requirements.txt
}

# Function to reset the script database
reset_script_redis() {
    echo "Resetting the script Redis database..."
    sudo docker compose stop script-redis
    sudo docker compose rm -f script-redis
    sudo docker volume rm ai-editor_script-redis-data
    sudo docker compose up -d script-redis
}

# Main script logic
while getopts ":qps" opt; do
    case $opt in
        q)
            reset_qdrant_database
            ;;
        p)
            setup_python_environment
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

