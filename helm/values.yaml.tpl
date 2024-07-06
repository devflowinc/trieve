environment: $ENVIRONMENT # Set to aws if deploying to 
domain: $DOMAIN # Only used if environment = local
externalDomain: $EXTERNAL_DOMAIN
useGpu: true
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
    tag: latest
  ingest:
    tag: latest
  bulk_ingest:
    tag: latest
  file_worker:
    tag: latest
  delete_worker:
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
    searchUiUrl: https://search.$EXTERNAL_DOMAIN
    frontmatterVals: "link,tag_set,time_stamp"
    sentryChatDsn: $SENTRY_CHAT_DSN
    dashboardUrl: $DASHBOARD_URL
  redis:
    connections: 2
    useSubchart: true
    uri: "redis://:redis@trieve-redis-master.default.svc.cluster.local:6379" # Only used if useSubchart is false
  qdrant:
    useSubchart: true
    qdrantUrl: http://trieve-qdrant.default.svc.cluster.local:6334 # Only used if useSubchart is false
    collection: collection
    apiKey: "qdrant_password"
    quantizeVectors: false # If set to true will binary quantize
    replicationFactor: 2
    initCollections: true
    vectorSizes:
      - 384
      - 512
      - 768
      - 1024
      - 1536
      - 3072
  trieve:
    unlimited: true
    cookieSecure: false
    baseServerUrl: https://api.$EXTERNAL_DOMAIN
    embeddingServerOrigin: http://embedding-jina.default.cluster.local
    sparseServerQueryOrigin: http://embedding-splade-query.default.cluster.local
    sparseServerDocOrigin: http://embedding-splade-doc.default.cluster.local
    embeddingServerOriginBGEM3: http://embedding-bgem3.default.cluster.local
    rerankerServerOrigin: http://embedding-reranker.default.cluster.local
    embeddingServerOriginJinaCode: https://api.jina.ai/v1
    jinaCodeApiKey: jina_************************************************************
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
    region: $AWS_REGION
  stripe:
    secret: $STRIPE_API_KEY
    webhookSecret: $STRIPE_WEBHOOK_SECRET
    adminDashboardUrl: https://dashboard.$EXTERNAL_DOMAIN
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
