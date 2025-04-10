
$env.NEW_EMBEDDING_MODEL_NAME
$env.NEW_EMBEDDING_BASE_URL
$env.NEW_EMBEDDING_SIZE

let apiKey = "admin"
let url = "http://localhost:8090"

###

$env.NEW_EMBEDDING_SIZE | into int

$env.DATASET_IDS | split row "," | each {
    $in
    let headers = {
        "TR-Dataset": $in
        "Authorization": $apiKey
    }
    let body = {
        "dataset_id": $in,
        "server_configuration": {
            "EMBEDDING_MODEL_NAME": $env.NEW_EMBEDDING_MODEL_NAME
            "EMBEDDING_BASE_URL": $env.NEW_EMBEDDING_BASE_URL
            "EMBEDDING_SIZE": ($env.NEW_EMBEDDING_SIZE | into int)
        }
    }
    http put  --content-type application/json $"($url)/api/dataset" $body --headers $headers -e
}

