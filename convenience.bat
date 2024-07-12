@echo off

if "%1"=="q" (
    call :reset_qdrant_database
)

if "%1"=="p" (
    call :setup_python_environment
)

if "%1"=="3" (
    call :reset_s3_service
)

if "%1"=="s" (
    call :reset_script_redis
)

if "%1"=="l" (
    call :start_local_services
)

if "%1"=="c" (
    call :build_ts_client
)

if "%1"=="" (
    echo "Usage: ./convenience.bat [q|p|3|s|l|c]"
    echo "q - reset the Qdrant database"
    echo "p - set up the Python environment"
    echo "3 - reset the S3 service"
    echo "s - reset the script Redis database"
    echo "l - start local services"
    echo "c - build the TypeScript client"
)

EXIT /B %ERRORLEVEL%

rem Function to reset the Qdrant database
:reset_qdrant_database
echo "Resetting the Qdrant database..."
docker "compose" "stop" "qdrant-database"
docker "compose" "rm" "-f" "qdrant-database"
docker "volume" "rm" "trieve_qdrant_data"
docker "compose" "up" "-d" "qdrant-database"
diesel "db" "reset"
EXIT /B 0

rem Function to reset the s3 serivce
:reset_s3_service
echo "Resetting the S3 service..."
docker "compose" "stop" "s3"
docker "compose" "rm" "-f" "s3"
docker "volume" "rm" "vault_s3-data"
docker "compose" "up" "-d" "s3"
EXIT /B 0

rem Function to reset the script database
:reset_script_redis
echo "Resetting the script Redis database..."
docker "compose" "stop" "script-redis"
docker "compose" "rm" "-f" "script-redis"
docker "volume" "rm" "vault_script-redis-data"
docker "compose" "up" "-d" "script-redis"
EXIT /B 0

:start_local_services
echo "Starting local services..."
docker "compose" "up" "-d" "db"
docker "compose" "up" "-d" "redis"
docker "compose" "up" "-d" "qdrant-database"
docker "compose" "up" "-d" "s3"
docker "compose" "up" "-d" "s3-client"
EXIT /B 0

:build_ts_client
echo "Building the TypeScript client..."
cargo run --features runtime-env --manifest-path server/Cargo.toml --bin redoc_ci > ./frontends/client/openapi.json
cd ./frontends/client/; yarn build:clean;
EXIT /B 0
