region: $AWS_REGION
accountId: $AWS_ACCOUNT_ID
environment: local
domain: $DOMAIN
externalDomain: $EXTERNAL_DOMAIN
useGpu: false
containers:
  keycloak:
    tag: 0.1.0
  minio:
    tag: 0.1.0
  postgres:
    tag: 0.1.0
  redis:
    tag: 0.1.0
  tika:
    tag: 0.1.0
  mc:
    tag: 0.1.0
  server:
    tag: 0.1.0
  ingest:
    tag: 0.1.0
  search:
    tag: 0.1.0
  chat:
    tag: 0.1.0
  dashboard:
    tag: 0.1.0
config:
  quantizeVectors: false
  vite:
    apiHost: http://server.default.svc.cluster.local:8090/api
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
  trieve:
    unlimited: true
    cookieSecure: false
    baseServerUrl: http://server-service.default.svc.cluster.local:8090
    gpuServerOrigin: http://localhost:7070
    embeddingServerOrigin: http://embedding-jina-service.default.cluster.local:80
    sparseServerQueryOrigin: http://embedding-splade-query-service.default.cluster.local:80
    sparseServerDocOrigin: http://embedding-splade-doc-service.default.cluster.local:80
    embeddingServerOriginBGEM3: http://embedding-bgem3-service.default.cluster.local:80
    rerankerServerOrigin: http://embedding-reranker-service.default.cluster.local:80
    salt: $SALT
    secretKey: $SECRET_KEY
    apiAdminKey: $API_ADMIN_KEY
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
    region: $AWS_REGION
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
    port: 9999
    model: jinaai/jina-embeddings-v2-small-en
    revision: main
    args: []
  - name: reranker
    model: BAAI/bge-reranker-large
    revision: refs/pr/4
    port: 7777
    args: []
  - name: splade-doc
    model: naver/efficient-splade-VI-BT-large-doc
    revision: main
    port: 7070
    args: []
  - name: splade-query
    model: naver/efficient-splade-VI-BT-large-query
    revision: main
    port: 7071
    args: ["--pooling", "splade"]
redis:
  enabled: true
