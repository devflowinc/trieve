#!/bin/sh
# set -e - revisit, make idemopotent

# Define a function to wait for MinIO server to become ready
wait_for_minio() {
    echo "Waiting for MinIO to start..."
    while ! mc ready local; do
        sleep 1
    done
    echo "MinIO started."
}

# Start MinIO server in the background if the command matches
if [ "$1" = "minio" ] && [ "$2" = "server" ]; then
    "$@" &
    
    wait_for_minio
    
    # Set alias
    mc alias set trieve http://127.0.0.1:9000 "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD" 2>/dev/null
    # s3 mb
    mc mb trieve/$S3_BUCKET 2>/dev/null # bucket name regex (?!(^xn--|.+-s3alias$))^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$
    # add user
    mc admin user add trieve $S3_ACCESS_KEY $S3_SECRET_KEY 2>/dev/null
    # attach policy
    mc admin policy attach trieve readwrite --user $S3_ACCESS_KEY 2>/dev/null

    wait
else
    exec "$@"
fi
