#!/bin/bash

# Default connection details
REDIS_HOST="localhost"
REDIS_PORT="6379"
REDIS_PASSWORD=""
CLICKHOUSE_HOST="localhost"
CLICKHOUSE_PORT="8123"
CLICKHOUSE_USER="default"
CLICKHOUSE_PASSWORD="password"
CLICKHOUSE_DB="default"

# Function to print usage
usage() {
    echo "Usage: $0 -d <dataset_id> [-rh <redis_host>] [-rp <redis_port>] [-rw <redis_password>] [-ch <clickhouse_host>] [-cp <clickhouse_port>] [-cu <clickhouse_user>] [-cw <clickhouse_password>] [-cd <clickhouse_db>]"
    exit 1
}

# Parse command line arguments
while getopts "d:rh:rp:rw:ch:cp:cu:cw:cd:" opt; do
    case $opt in
        d) DATASET_ID="$OPTARG" ;;
        rh) REDIS_HOST="$OPTARG" ;;
        rp) REDIS_PORT="$OPTARG" ;;
        rw) REDIS_PASSWORD="$OPTARG" ;;
        ch) CLICKHOUSE_HOST="$OPTARG" ;;
        cp) CLICKHOUSE_PORT="$OPTARG" ;;
        cu) CLICKHOUSE_USER="$OPTARG" ;;
        cw) CLICKHOUSE_PASSWORD="$OPTARG" ;;
        cd) CLICKHOUSE_DB="$OPTARG" ;;
        *) usage ;;
    esac
done

# Check if dataset_id is provided
if [ -z "$DATASET_ID" ]; then
    echo "Error: dataset_id is required"
    usage
fi

# Construct Redis CLI command
REDIS_CMD="redis-cli -h $REDIS_HOST -p $REDIS_PORT"
if [ -n "$REDIS_PASSWORD" ]; then
    REDIS_CMD="$REDIS_CMD -a $REDIS_PASSWORD"
fi

# Delete key from Redis
echo "Deleting key *$DATASET_ID from Redis..."
$REDIS_CMD DEL "*$DATASET_ID"

# Delete row from ClickHouse
echo "Deleting row with dataset_id=$DATASET_ID from ClickHouse..."
clickhouse-client \
    --host "$CLICKHOUSE_HOST" \
    --port "$CLICKHOUSE_PORT" \
    --user "$CLICKHOUSE_USER" \
    --password "$CLICKHOUSE_PASSWORD" \
    --database "$CLICKHOUSE_DB" \
    --query "ALTER TABLE dataset_words_last_processed DELETE WHERE dataset_id = '$DATASET_ID'"

echo "Cleanup completed for dataset_id: $DATASET_ID"