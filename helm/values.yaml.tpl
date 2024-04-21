region: $AWS_REGION
environment: local
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
    sentryChatDsn: ""
    dashboardUrl: http://localhost:5173
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
    unlimited: "true"
    cookieSecure: false
    baseServerUrl: http://server-service.default.svc.cluster.local:8090
    gpuServerOrigin: http://localhost:7070
    sparseServerQueryOrigin: ""
    sparseServerDocOrigin: ""
    embeddingServerOrigin: ""
    embeddingServerOriginBGEM3: ""
    rerankerServerOrigin: ""
    salt: $SALT
    secretKey: $SECRET_KEY
    apiAdminKey: $ADMIN_API_KEY
  oidc:
    clientSecret: $OIDC_CLIENT_SECRET
    clientId: vault
    issuerUrl: http://keycloak-service.default.svc.cluster.local:8080/realms/trieve
    authRedirectUrl: http://keycloak-service.default.svc.cluster.local:8080/realms/trieve/protocol/openid-connect/auth
    redirectUrl: http://keycloak-service.default.svc.cluster.local:8080/realms/trieve/protocol/openid-connect/auth
  
  smtp:
    relay: $SMTP_RELAY
    username: $SMTP_USERNAME
    passworD: $SMTP_PASSWORD
    emailAddress: $SMTP_EMAIL_ADDRESS
  llm:
    apiKey: $OPENAI_API_KEY
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
    webhookSecret: $STRIPE_WEBHOOK_SERCRET
    adminDashboardUrl: http://keycloak.default.svc.cluster.local:8080/admin
embeddings:
  - name: jina
    revision: main
    port: 9999
    model: jinaai/jina-embeddings-v2-small-en
    revision: main
  - name: reranker
    model: BAAI/bge-reranker-large
    revision: refs/pr/4
    port: 7777
  - name: splade-doc
    model: naver/efficient-splade-VI-BT-large-doc
    revision: main
    port: 7070
  - name: splade-query
    model: naver/efficient-splade-VI-BT-large-query
    revision: main
    port: 7071
redis:
  enabled: true
