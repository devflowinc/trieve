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

postgres:
  useSubchart: true
  dbURI: postgres://postgres:password@trieve-postgresql.default.svc.cluster.local:5432 # Only used if useSubchart is false
config:
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
    useSubchart: true
    qdrantUrl: http://trieve-qdrant.default.svc.cluster.local:6334 # Only used if useSubchart is false
    collection: collection
    apiKey: "qdrant_password"
    quantizeVectors: false # If set to true will binary quantize
    replicationFactor: 2
    vectorSizes:
      - 384
      - 512
      - 768
      - 1024
      - 1536
      - 3072
  ingest:
    num_threads: 1
  trieve:
    unlimited: true
    cookieSecure: false
    baseServerUrl: https://api.$EXTERNAL_DOMAIN
    gpuServerOrigin: http://localhost:7070
    embeddingServerOrigin: http://embedding-jina.default.cluster.local
    sparseServerQueryOrigin: http://embedding-splade-query.default.cluster.local
    sparseServerDocOrigin: http://embedding-splade-doc.default.cluster.local
    embeddingServerOriginBGEM3: http://embedding-bgem3.default.cluster.local
    rerankerServerOrigin: http://embedding-reranker.default.cluster.local
    salt: $SALT
    secretKey: $SECRET_KEY
    adminApiKey: $ADMIN_API_KEY 
  oidc:
    clientSecret: $OIDC_CLIENT_SECRET
    clientId: trieve
    issuerUrl: $ISSUER_URL
    authRedirectUrl: $AUTH_REDIRECT_URL
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
    model: jinaai/jina-embeddings-v2-base-en
    revision: main
    args: []
  - name: reranker
    model: BAAI/bge-reranker-large
    revision: refs/pr/4
    args: []
  - name: splade-doc
    model: naver/efficient-splade-VI-BT-large-doc
    revision: main
    args: ["--pooling", "splade"]
  - name: splade-query
    model: naver/efficient-splade-VI-BT-large-query
    revision: main
    args: ["--pooling", "splade"]
redis:
  enabled: true
