region: $AWS_REGION
environment: $ENVIRONMENT # Set to aws if deploying to 
domain: $DOMAIN # Only used if environment = local
externalDomain: $EXTERNAL_DOMAIN
useGpu: false
containers:
  keycloak:
    tag: 23.0.7
  minio:
    tag: RELEASE.2023-09-27T15-22-50Z
  tika:
    tag: 2.9.1.0-full
  mc:
    tag: latest
  server:
    tag: no-ocr
  ingest:
    tag: latest
  search:
    tag: latest
  chat:
    tag: latest
  dashboard:
    tag: latest
config:
  quantizeVectors: false
  vite:
    apiHost: https://api.$EXTERNAL_DOMAIN/api
    searchUiUrl: https://search.trieve.ai
    frontmatterVals: "link,tag_set,time_stamp"
    sentryChatDsn: $SENTRY_CHAT_DSN
    dashboardUrl: $DASHBOARD_URL
  keycloak:
    admin: $KEYCLOAK_ADMIN
    password: $KEYCLOAK_PASSWORD
  minio:
    rootUser: $MINIO_ROOT_USER
    rootPassword: $MINIO_ROOT_PASSWORD
  redis:
    connections: 30
    password: redis
  qdrant:
    collection: collection
    apiKey: "qdrant_password"
    quantizeVectors: false # If set to true will binary quantize
    replicationFactor: 2
  ingest:
    replicas: 5
  trieve:
    unlimited: true
    cookieSecure: false
    baseServerUrl: https://api.$EXTERNAL_DOMAIN
    gpuServerOrigin: http://localhost:7070
    embeddingServerOrigin: http://embedding-jina-service.default.cluster.local:80
    sparseServerQueryOrigin: http://embedding-splade-query-service.default.cluster.local:80
    sparseServerDocOrigin: http://embedding-splade-doc-service.default.cluster.local:80
    embeddingServerOriginBGEM3: http://embedding-bgem3-service.default.cluster.local:80
    rerankerServerOrigin: http://embedding-reranker-service.default.cluster.local:80
    salt: $SALT
    secretKey: $SECRET_KEY
    apiAdminKey: $ADMIN_API_KEY
  oidc:
    clientSecret: $OIDC_CLIENT_SECRET
    clientId: vault
    issuerUrl: $ISSUER_URL
    authRedirectUrl: $AUTH_REDIRECT_URL
    redirectUrl: $REDIRECT_URL
  
  smtp:
    relay: $SMTP_RELAY
    username: $SMTP_USERNAME
    passworD: $SMTP_PASSWORD
    emailAddress: $SMTP_EMAIL_ADDRESS
  llm:
    apiKey: $LLM_API_KEY
  openai:
    apiKey: $OPENAI_API_KEY
    baseUrl: $OPENAI_BASE_URL
  s3:
    endpoint: $S3_ENDPOINT
    accessKey: $S3_ACCESS_KEY
    secretKey: $S3_SECRET_KEY
    bucket: $S3_BUCKET
  stripe:
    secret: $STRIPE_API_KEY
    webhookSecret: $STRIPE_WEBHOOK_SECRET
    adminDashboardUrl: http://keycloak.default.svc.cluster.local:8080/admin
embeddings:
  - name: jina
    revision: main
    port: 80
    model: jinaai/jina-embeddings-v2-small-en
    revision: main
    args: []
  - name: reranker
    model: BAAI/bge-reranker-large
    revision: refs/pr/4
    port: 80
    args: []
  - name: splade-doc
    model: naver/efficient-splade-VI-BT-large-doc
    revision: main
    port: 80
    args: ["--pooling", "splade"]
  - name: splade-query
    model: naver/efficient-splade-VI-BT-large-query
    revision: main
    port: 80
    args: ["--pooling", "splade"]
redis:
  enabled: true
