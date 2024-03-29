#!/bin/bash
set -e

# Perform all actions as $POSTGRES_USER
export PGPASSWORD="${POSTGRES_PASSWORD}"

# Wait for PostgreSQL to start
# Optional: Remove this block if not needed
until pg_isready -U "${POSTGRES_USER}"; do
  echo "Waiting for PostgreSQL to become available..."
  sleep 1
done
echo "PostgreSQL is available."

# Create the Keycloak user and database
psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname "${POSTGRES_DB}" <<-EOSQL
    CREATE USER ${KC_DB_USERNAME} WITH PASSWORD '${KC_DB_PASSWORD}';
    CREATE DATABASE ${KC_DB} OWNER ${KC_DB_USERNAME};
EOSQL

echo "Keycloak database setup completed."
