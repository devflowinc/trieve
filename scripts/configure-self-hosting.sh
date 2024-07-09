#!/bin/bash

# Function to check GPU and system resources
check_gpu_and_resources() {
    if command -v nvidia-smi &> /dev/null; then
        architecture=$(nvidia-smi -q | grep "Product Architecture" | awk '{print $NF}')
        if [[ "$architecture" == "Turing" ]]; then
            echo "Turing GPU detected. Running GPU-Embeddings(docker-compose-gpu-embeddings.yml) and Trieve(docker-compose.yml)."
            echo "*** Note on embedding servers. If you want to use a separate GPU enabled device for embedding servers you will need to update the following parameters:"
            echo "SPARSE_SERVER_QUERY_ORIGIN"
            echo "SPARSE_SERVER_DOC_ORIGIN"
            echo "EMBEDDING_SERVER_ORIGIN"
            echo "SPARSE_SERVER_QUERY_ORIGIN"
            docker compose -f docker-compose-gpu-embeddings.yml up -d && sleep 3 && docker compose up -d
        elif [[ "$architecture" =~ ^(Ampere|Ada Lovelace|Hopper)$ ]]; then
            echo "Advanced GPU architecture detected ($architecture)."
            echo "Running Trieve(docker-compose.yml). To run GPU Embeddings:"
            echo "Go to https://github.com/huggingface/text-embeddings-inference?tab=readme-ov-file#docker-images and check docker image tags."
            docker compose up -d
        else
            check_cpu_and_ram
        fi
    else
        check_cpu_and_ram
    fi
}

# Function to check CPU and RAM
check_cpu_and_ram() {
    cpu_cores=$(nproc)
    total_ram=$(free -g | awk '/^Mem:/{print $2}')

    if [[ $cpu_cores -ge 4 && $total_ram -ge 24 ]]; then
        echo "Sufficient CPU and RAM detected. Running CPU-Embeddings(docker-compose-cpu-embeddings.yml) and Trieve(docker-compose.yml)."
        docker compose -f docker-compose-cpu-embeddings.yml up -d && sleep 3 && docker compose up -d
    else
        echo "Insufficient resources. Running Trieve(docker-compose.yml) wihout Embeddings"
        sudo docker compose up -d
    fi
}

### CHECK DEPENDENCY

for cmd in curl awk jq; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd is not installed. Please install it and try again."
        exit 1
    fi
done

### CHECK IF DOMAIN SET

if [[ -z "$DOMAIN" ]]; then
    echo "Error: DOMAIN environment variable is not set. Type 'DOMAIN=yourdomain.com ./configure.sh'"
    exit 1
fi

### CHECK IF PWD IS SET TO TRIEVE DIR

is_in_trieve_or_subfolder() {
    local current_path="$PWD"
    while [[ "$current_path" != "/" ]]; do
        if [[ $(basename "$current_path") == "trieve" ]]; then
            return 0
        fi
        current_path=$(dirname "$current_path")
    done
    return 1
}

if is_in_trieve_or_subfolder; then
    if [[ $(basename "$PWD") == "trieve" ]]; then
        echo "Already in the trieve directory."
    else
        echo "In a subdirectory of trieve. Moving to the main trieve directory."
        cd "$(pwd | sed -e 's/trieve.*/trieve/')"
        echo "Successfully changed to the trieve directory: $PWD"
    fi
else
    if cd trieve 2>/dev/null; then
        echo "Successfully changed to the trieve directory."
    else
        echo "Error: Unable to change to the trieve directory. It may not exist or you may not have permission to access it." >&2
        exit 1
    fi
fi
echo $PWD

### PORTS FOR CADDYFILE

declare -A URL_ARRAY=(
    ["dashboard"]="5173"
    ["chat"]="5175"
    ["search"]="5174"
    ["api"]="8090"
    ["auth"]="8080"
    ["analytics"]="5176"
)

### GENERATE AND COPY CADDYFILE

if ! sudo tee Caddyfile > /dev/null << EOF
$(for subdomain in "${!URL_ARRAY[@]}"; do
echo "${subdomain}.${DOMAIN} {
    reverse_proxy localhost:${URL_ARRAY[$subdomain]}
}"
done)
EOF
then
    echo "Error: Failed to generate Caddyfile."
    exit 1
fi

if ! sudo cp -f Caddyfile /etc/caddy/Caddyfile; then
    echo "Error: Failed to copy Caddyfile to /etc/caddy/Caddyfile."
    exit 1
fi

### CHECK IF CADDY SERVICE WORKING AND MAKE RELOAD

if ! sudo systemctl is-enabled caddy.service &>/dev/null; then
    echo "Caddy service is not enabled. Enabling..."
    if ! sudo systemctl enable caddy.service; then
        echo "Error: Failed to enable Caddy service."
        exit 1
    fi
    echo "Caddy service has been enabled."
fi

if sudo systemctl is-active caddy.service &>/dev/null; then
    echo "Caddy service is running. Reloading..."
    if ! sudo systemctl reload caddy.service; then
        echo "Error: Failed to reload Caddy service."
        exit 1
    fi
    echo "Caddy service has been reloaded."
else
    echo "Caddy service is not running. Starting..."
    if ! sudo systemctl start caddy.service; then
        echo "Error: Failed to start Caddy service."
        exit 1
    fi
    echo "Caddy service has been started."
fi

echo "Caddy service has been successfully configured and is running."

### GET IP PUBLIC ADDR

IP_ADDR=$(curl -4s icanhazip.com)
if [[ -z "$IP_ADDR" ]]; then
    echo "Error: Failed to retrieve IP address."
    exit 1
fi

### DISPLAY DNS A RECORD

echo "Add DNS RECORD to your domain:"
for subdomain in "${!URL_ARRAY[@]}"; do
    echo "A ${subdomain}.${DOMAIN} $IP_ADDR"
done

read -p "Do you change DOMAIN A RECORDS? (y|N) " answer

case ${answer:0:1} in
    y|Y )
        echo "Continuing with the script..."
    ;;
    * )
        echo "Exiting without changes."
        exit 0
    ;;
esac

### EDIT ENV FILE

if ! awk -v domain="$DOMAIN" '
BEGIN {
    vars["KC_HOSTNAME"] = "auth." domain
    vars["VITE_API_HOST"] = "https://api." domain "/api"
    vars["VITE_SEARCH_UI_URL"] = "https://search." domain
    vars["VITE_CHAT_UI_URL"] = "https://chat." domain
    vars["VITE_ANALYTICS_UI_URL"] = "https://analytics." domain
    vars["VITE_DASHBOARD_URL"] = "https://dashboard." domain
    vars["OIDC_AUTH_REDIRECT_URL"] = "https://auth." domain "/realms/trieve/protocol/openid-connect/auth"
    vars["OIDC_ISSUER_URL"] = "https://auth." domain "/realms/trieve"
    vars["BASE_SERVER_URL"] = "https://api." domain
}
{
    if ($0 ~ "^KC_PROXY=") {
        print "KC_PROXY=edge " $2
    } else {
        for (var in vars) {
            if ($0 ~ "^" var "=") {
                $0 = var "=\"" vars[var] "\""
                break
            }
        }
        print $0
    }
}' .env.example > .env
then
    echo "Error: Failed to generate .env file."
    exit 1
fi

### EDIT DOCKER-COMPOSE FILE

INPUT_FILE="docker-compose.yml"
OUTPUT_FILE="docker-compose-modified.yml"

if ! awk '
{
    if ($0 ~ /^\s*-\s*KC_PROXY=/) {
        gsub(/KC_PROXY=.*/, "KC_PROXY=${KC_PROXY}")
    }
    print $0
}' "$INPUT_FILE" > "$OUTPUT_FILE"
then
    echo "Error: Failed to modify docker-compose.yml"
    exit 1
fi

mv "$OUTPUT_FILE" "$INPUT_FILE"

echo "Successfully modified docker-compose.yml"


### EDIT REALM-EXPORT FILE

INPUT_FILE="docker/keycloak/realm-export.json"
OUTPUT_FILE="docker/keycloak/realm-export-modified.json"

if ! awk -v DOMAIN="$DOMAIN" '
BEGIN {
    FS = OFS = "\""
    in_redirect_uris = 0
    printed_new_uris = 0
}

{
    if ($0 ~ /"redirectUris"/) {
        in_redirect_uris = 1
        print $0
    } else if (in_redirect_uris && $0 ~ /]/) {
        in_redirect_uris = 0
        printed_new_uris = 0
        print $0
    } else if (in_redirect_uris && $2 == "http://localhost:8090/*" && !printed_new_uris) {
        print "        \"https://api." DOMAIN "/*\","
        print "        \"https://analytics." DOMAIN "/*\","
        print "        \"https://search." DOMAIN "/*\","
        print "        \"https://chat." DOMAIN "/*\","
        print "        \"https://dashboard." DOMAIN "/*\","
        print $0
        printed_new_uris = 1
    } else if ($0 ~ /"post\.logout\.redirect\.uris":/) {
        gsub(/"http:\/\/localhost:8090\/\*"/, "\"http://localhost:8090/*##https://api." DOMAIN "/*##https://analytics." DOMAIN "/*##https://search." DOMAIN "/*##https://chat." DOMAIN "/*##https://dashboard." DOMAIN "/*\"")
        print $0
    } else {
        print $0
    }
}' "$INPUT_FILE" > "$OUTPUT_FILE"
then
    echo "Error: Failed to modify realm-export.json"
    exit 1
fi

mv "$OUTPUT_FILE" "$INPUT_FILE"

echo "Successfully modified realm-export.json"
# Check GPU and system resources
echo ""
check_gpu_and_resources

