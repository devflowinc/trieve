# Self hosting guide

We currently offer 3 ways to self host Trieve.

- [Docker Compose](#docker-compose)
- [Kubernetes (kind/minikube)](#local-kubernetes)
- [Kubernetes (AWS EKS)](#aws-eks)
- [Kubernetes (GCP GKE)](#gcp-gke)

We reccomend GKE for production deployments, EKS is not able to have the embedding servers exist within the kubernetes cluster due to issues with the fractional GPU usage driver.

## Docker Compose

The Docker Compose self hosted option is the easiest way to get started self hosting Trieve.

Things you need

- Domain name
- System with at least 4 CPU cores and 8GB of RAM (excluding the cpu embedding servers)
- System with at least 4 CPU cores and >25GB of RAM (including the cpu embedding servers)

### Install Docker

```bash
curl https://get.docker.com | sh
```

### Clone Trieve repository

```sh
git clone https://github.com/devflowinc/trieve
cd trieve
```

### Create `.env` file

```sh
cp .env.example .env
```

### Start Trieve Services

```sh
docker compose up -d
```

### Start Embedding servers

We offer 2 docker-compose files for embedding servers. One for GPU and one for CPU.

```sh
docker compose -f docker-compose-cpu-embeddings.yml up -d
```

or

```sh
docker compose -f docker-compose-cpu-embeddings.yml up -d
```

\* Note on embedding servers. If you want to use a separate GPU enabled device for embedding servers you will need to update the following parameters

```sh
SPARSE_SERVER_QUERY_ORIGIN
SPARSE_SERVER_DOC_ORIGIN
EMBEDDING_SERVER_ORIGIN
SPARSE_SERVER_QUERY_ORIGIN
```

### Setup Caddy reverse proxy

Install Caddy

Edit the Caddyfile

```sh
nano /etc/caddy/Caddyfile
```

Add the following configuration

```Caddyfile
dashboard.yourdomain.com {
    reverse_proxy localhost:5173
}

chat.yourdomain.com {
    reverse_proxy localhost:5175
}

search.yourdomain.com {
    reverse_proxy localhost:5174
}

api.yourdomain.com {
    reverse_proxy localhost:8090
}

auth.yourdomain.com {
    reverse_proxy localhost:8080
}
```

Start Caddy, you may also need to reload the service

```sh
sudo systemctl reload caddy.service
```

### Set the following A records for your domain to point to the server IP address.

```
A dashboard.yourdomain.com your-server-ip
A chat.yourdomain.com your-server-ip
A search.yourdomain.com your-server-ip
A auth.yourdomain.com your-server-ip
A api.yourdomain.com your-server-ip
```

### Edit .env

Most values can be left as default, the ones you do need to edit are

```
KC_HOSTNAME="auth.yourdomain.com"
KC_PROXY=edge

VITE_API_HOST=https://api.yourdomain.com/api
VITE_SEARCH_UI_URL=https://search.yourdomain.com
VITE_CHAT_UI_URL=https://chat.yourdomain.com
VITE_ANALYTICS_UI_URL=https://analytics.yourdomain.com
VITE_DASHBOARD_URL=https://dashboard.yourdomain.com

OIDC_AUTH_REDIRECT_URL="https://auth.yourdomain.com/realms/trieve/protocol/openid-connect/auth"
OIDC_ISSUER_URL="https://auth.yourdomain.com/realms/trieve"
BASE_SERVER_URL="https://api.yourdomain.com"
```

### Authorize keycloak redirect URLs

Go to auth.yourdomain.com and login with the default credentials (user: admin password: aintsecure)

1. Change the Realm from master to trieve
2. Go to Clients -> vault -> Settings
3. Add the following to the Valid Redirect URIs and Valid Post Logout Redirect URIs

```
https://api.yourdomain.com/*
https://dashboard.yourdomain.com/*
https://chat.yourdomain.com/*
https://search.yourdomain.com/*
```

### Testing

The fastest way to test is using the trieve cli

```
trieve init
trieve dataset example
```

And there you have it. Your very own trieve stack.
Happy hacking 🚀

## Local Kuberntes

We reccomend using `kind`

1) Create kind cluster with a local image registry

```sh

./scripts/kind-with-registry.sh

```

2) Install Trieve Dependencies

Trieve depends on the following:
- keycloak (or some other oidc compliant X)
- postgres 
- Redis
- clickhouse
- qdrant

We preconfigure all of these except `qdrant` within `test/`.

Install with `kubectl`, you will need to run this command twice as it contains CRD's

```sh

kubectl apply -f ./helm/test

```

3) Install Qdrant via helm chart

For updated install instructions follow qdrant's guide. https://github.com/qdrant/qdrant-helm/blob/main/charts/qdrant/README.md

```sh

helm repo add qdrant https://qdrant.github.io/qdrant-helm
helm repo update
helm upgrade -i qdrant qdrant/qdrant --set apiKey="qdrant-api-key"

```


4) Ensure all Dependencies are installed.

It may take a while for all resources to be available, kind is typically a bit slower than a production cluster.

To verify the installation is correct run `kubectl get all` and verify the output matches that below.

Easiest way to do this is run

```sh

kubectl get all

```

```
NAME                                       READY   STATUS      RESTARTS   AGE
pod/chi-trieve-clickhouse-cluster1-0-0-0   1/1     Running     0          30m
pod/keycloak-operator-5d45b88d94-kclw2     1/1     Running     0          31m
pod/keycloak-postgres-0                    1/1     Running     0          31m
pod/postgresql-db-0                        1/1     Running     0          31m
pod/qdrant-0                               1/1     Running     0          22s
pod/redis-68d8d5bddf-4p5df                 1/1     Running     0          31m
pod/trieve-keycloak-0                      1/1     Running     0          6m51s
pod/trieve-realm-xv7hp                     0/1     Completed   0          7m2s

NAME                                         TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)                      AGE
service/chi-trieve-clickhouse-cluster1-0-0   ClusterIP   None            <none>        9000/TCP,8123/TCP,9009/TCP   30m
service/clickhouse-trieve-clickhouse         ClusterIP   None            <none>        8123/TCP,9000/TCP            30m
service/keycloak-operator                    ClusterIP   10.96.152.103   <none>        80/TCP                       31m
service/keycloak-postgres                    ClusterIP   10.96.214.23    <none>        5432/TCP                     31m
service/kubernetes                           ClusterIP   10.96.0.1       <none>        443/TCP                      31m
service/postgres                             ClusterIP   10.96.217.15    <none>        5432/TCP                     31m
service/qdrant                               ClusterIP   10.96.44.173    <none>        6333/TCP,6334/TCP,6335/TCP   22s
service/qdrant-headless                      ClusterIP   None            <none>        6333/TCP,6334/TCP,6335/TCP   22s
service/redis                                ClusterIP   10.96.172.177   <none>        6379/TCP                     31m
service/trieve-keycloak-discovery            ClusterIP   None            <none>        7800/TCP                     7m42s
service/trieve-keycloak-service              ClusterIP   10.96.136.207   <none>        8080/TCP,9000/TCP            7m42s

NAME                                READY   UP-TO-DATE   AVAILABLE   AGE
deployment.apps/keycloak-operator   1/1     1            1           31m
deployment.apps/redis               1/1     1            1           31m

NAME                                           DESIRED   CURRENT   READY   AGE
replicaset.apps/keycloak-operator-5d45b88d94   1         1         1       31m
replicaset.apps/redis-68d8d5bddf               1         1         1       31m

NAME                                                  READY   AGE
statefulset.apps/chi-trieve-clickhouse-cluster1-0-0   1/1     30m
statefulset.apps/keycloak-postgres                    1/1     31m
statefulset.apps/postgresql-db                        1/1     31m
statefulset.apps/qdrant                               1/1     22s
statefulset.apps/trieve-keycloak                      1/1     7m42s

NAME                     STATUS     COMPLETIONS   DURATION   AGE
job.batch/trieve-realm   Complete   1/1           10s        7m2s

```

6) Setup Trieve `values.yaml`


7) Install the Trieve helm chart

The `helm/values.local.yaml` file is preconfigured to work with the dependencies as installed by `test/`.
They should also be namespace agnostic if you choose a different namespace. Edit any values if needed.

```sh

helm upgrade -i trieve-local -f helm/values.local.yaml helm/

```

kubectl get service | grep service
kubectl describe ingress/ingress-dashboard
kubectl get pods | grep dashboard

kubectl get deploy

After installing, `kubectl get deployments` should look like this

```
NAME                READY   UP-TO-DATE   AVAILABLE   AGE
bktree-worker       1/1     1            1           14m
chat                1/1     1            1           14m
crawl-worker        1/1     1            1           14m
dashboard           1/1     1            1           14m
delete-worker       1/1     1            1           14m
group-worker        1/1     1            1           14m
ingest              10/10   10           10          14m
keycloak-operator   1/1     1            1           46m
redis               1/1     1            1           46m
search              1/1     1            1           14m
server              0/3     1            0           14m
sync-qdrant         0/0     0            0           14m
word-worker         1/1     1            1           14m
```

8) Edit `/etc/hosts` and add the following entries here.

```
127.0.0.1  api.localtrieve.com
127.0.0.1  search.localtrieve.com
127.0.0.1  dashboard.localtrieve.com
127.0.0.1  chat.localtrieve.com
127.0.0.1  auth.localtrieve.com
127.0.0.1       trieve-keycloak-service
```

and port-forward keycloak

```sh
kubectl port-forward svc/trieve-keycloak-service 8080:8080
```


9) Trieve is installed and ready to use.

Navigate to `http://dashboard.localtrieve.com`, and make an acocount. (The email does not need to be real).

You should be use everything locally with a local CPU embedding server.

## AWS EKS

Things you need

- Domain name
- An allowance for at least 8vCPU for G and VT instances
- helm cli
- aws cli
- kubectl
- k9s (optional)

Run this to configure the cli's iam user and secret key
 
```sh
aws configure
```

### Provision Terraform

aws should be configured with your IAM credentails chosen. Run the following commands to create the EKS cluster

```sh
cd terraform/aws
terraform init
terraform apply
```

This should provision an eks cluster with elb and ebs drivers

### 2.5 Setup embedding servers

Due to many issues with the NVIDIA k8s-device-plugin, we have not figured out how to do fractional GPU usage for pods within kubernetes, meaning its not economically reasonable to have the GPU embedding server within Kubernetes.For this reason we have a `docker-compose.yml` that is recommended to be ran on the ec2 box provisioned within the Terraform. (Note it uses `~/.ssh/id_ed25519` as a default key). The user to login as is `dev`.

### 3 Create values.yaml

```sh
export SENTRY_CHAT_DSN=https://********************************@sentry.trieve.ai/6
export ENVIRONMENT=aws
export DOMAIN=example.com # Only used for local
export EXTERNAL_DOMAIN=example.com
export DASHBOARD_URL=https://dashboard.example.com
export SALT=goodsaltisveryyummy
export SECRET_KEY=1234512345123451234512345123451234512345123451234512345123451234512345123451234h
export ADMIN_API_KEY=asdasdasdasdasd
export OIDC_CLIENT_SECRET=YllmLDTy67MbsUBrUAWvQ7z9aMq0QcKx
export ISSUER_URL=https://oidc.example.com
export AUTH_REDIRECT_URL=https://oidc.example.com/realms/trieve/protocol/openid-connect/auth
export REDIRECT_URL=https://oidc.example.com/realms/trieve/protocol/openid-connect/auth
export SMTP_RELAY=smtp.gmail.com
export SMTP_USERNAME=trieve@gmail.com
export SMTP_PASSWORD=pass************
export SMTP_EMAIL_ADDRESS=triever@gmail.com
export LLM_API_KEY=sk-or-v1-**************************************************************** # Open Router API KEY
export OPENAI_API_KEY=sk-************************************************ # OPENAI API KEY
export OPENAI_BASE_URL=https://api.openai.com/v1
export S3_ENDPOINT=https://<bucket>.s3.amazonaws.com
export S3_ACCESS_KEY=ZaaZZaaZZaaZZaaZZaaZ
export S3_SECRET_KEY=ssssssssssssssssssssTTTTTTTTTTTTTTTTTTTT
export S3_BUCKET=trieve
export AWS_REGION=us-east-1
export STRIPE_API_KEY=sk_test_***************************************************************************************************
export STRIPE_WEBHOOK_SECRET=sk_test_***************************************************************************************************

helm/from-env.sh
```

This step generates a file in `helm/values.yaml`. It alllows you to modify the environment variables

Additionally, you will need to modify values.yaml in two ways,

First you will need to change all the embedding server origins to point to the embedding server url as follows.

```yaml
config:
---
trieve:
---
sparseServerDocOrigin: http://<ip>:5000
sparseServerQueryOrigin: http://<ip>:6000
embeddingServerOrigin: http://<ip>:7000
embeddingServerOriginBGEM3: http://<ip>:8000
rerankerServerOrigin: http://<ip>:9000
```

Since the embbedding servers are not included in the kubernetes cluster, remove all items in the embeddings list below and leave it empty as follows

```yaml

---
embeddings:
```

### SubChart usage

Postgres, Redis, and qdrant are installed within this helm chart via a subchart. You can opt out of using the helm chart installation and using a managed service can be toggled via. the `useSubChart` parameters and setting the `uri` to a managed service. We reccomend placing at least Postgres and Redis out side of the helm chart and keeping Qdrant in the helm chart for a production usecase.

### 4. Deploy the helm chart

```sh
aws eks update-kubeconfig --region us-west-1 --name trieve-cluster
helm install -f helm/values.yaml trieve helm/
```

Ensure everything has been deployed with

```sh
kubectl get pods
```

### 6. Set DNS records

First get the ingress addresses using

```sh
kubectl get ingress
```

You will get output that looks like this

```
NAME                CLASS   HOSTS   ADDRESS                                                                  PORTS   AGE
ingress-chat        alb     *       k8s-default-ingressc-033d55ca4e-253753726.us-west-1.elb.amazonaws.com    80      9s
ingress-dashboard   alb     *       k8s-default-ingressd-50d3c72d7f-2039058451.us-west-1.elb.amazonaws.com   80      9s
ingress-search      alb     *       k8s-default-ingresss-2007ac265d-1873944939.us-west-1.elb.amazonaws.com   80      9s
ingress-server      alb     *       k8s-default-ingresss-cb909f388e-797764884.us-west-1.elb.amazonaws.com    80      9s
```

Set CNAME's accordingly, we recommend using Cloudflare to CNAME and provision the SSL Cert

```
ingress-chat            chat.domain.
ingress-dashboard       dashboard.domain.
ingress-search          search.domain.
ingress-server          api.domain.
```

Once you set the ingress rules properly, the server should be able to properly deploy.

### 7. Setup/OIDC provider and Authorized Redirect URL'S

The last step is to setup an OIDC compliant server like `keycloak` for authentication and get an `issuerUrl` and `clientSecret`. This is how you do it within Keycloak.

A) Create a new realm called `trieve`
B) Go into Clients and create a new client called `trieve`.

Enable client authentication and set the following allowed redirect url's

- https://api.domain.com/*
- https://search.domain.com/*
- https://chat.domain.com/*
- https://dashboard.domain.com/*

You will get the client secret in the `Credentials` tab.

You will need to set the following values in the `helm/values.yaml` file, it should be prefilled already with default values

```
config:
  oidc:
    clientSecret: $OIDC_CLIENT_SECRET
    clientId: trieve
    issuerUrl: https://auth.domain.com/realms/trieve
    authRedirectUrl: https://auth.domain.com/realms/trieve/protocol/openid-connect/auth
```

### Testing

The fastest way to test is using the trieve cli

```
trieve init
trieve dataset example
```

And there you have it. Your very own Trieve stack.
Happy hacking 🚀

## GCP GKE

Things you need

- Domain name
- helm cli (https://helm.sh/docs/intro/install)
- google cloud cli (https://cloud.google.com/sdk/docs/install)
- kubectl
- k9s (optional)

Run this to set your project-id

```sh
gcloud auth login
```

### Provision Terraform

aws should be configured with your IAM credentails chosen. Run the following commands to create the EKS cluster

```sh
cd terraform/gcloud
terraform init
terraform apply
```

This should provision an eks cluster with elb and ebs drivers

### 3 Create values.yaml

```sh
export SENTRY_CHAT_DSN=https://********************************@sentry.trieve.ai/6
export ENVIRONMENT=gcloud
export DOMAIN=example.com # Only used for local
export EXTERNAL_DOMAIN=example.com
export DASHBOARD_URL=https://dashboard.example.com
export SALT=goodsaltisveryyummy
export SECRET_KEY=1234512345123451234512345123451234512345123451234512345123451234512345123451234h
export ADMIN_API_KEY=asdasdasdasdasd
export OIDC_CLIENT_SECRET=YllmLDTy67MbsUBrUAWvQ7z9aMq0QcKx
export ISSUER_URL=https://oidc.example.com
export AUTH_REDIRECT_URL=https://oidc.example.com/realms/trieve/protocol/openid-connect/auth
export REDIRECT_URL=https://oidc.example.com/realms/trieve/protocol/openid-connect/auth
export SMTP_RELAY=smtp.gmail.com
export SMTP_USERNAME=trieve@gmail.com
export SMTP_PASSWORD=pass************
export SMTP_EMAIL_ADDRESS=triever@gmail.com
export LLM_API_KEY=sk-or-v1-**************************************************************** # Open Router API KEY
export OPENAI_API_KEY=sk-************************************************ # OPENAI API KEY
export OPENAI_BASE_URL=https://api.openai.com/v1
export S3_ENDPOINT=https://<bucket>.s3.amazonaws.com
export S3_ACCESS_KEY=ZaaZZaaZZaaZZaaZZaaZ
export S3_SECRET_KEY=ssssssssssssssssssssTTTTTTTTTTTTTTTTTTTT
export S3_BUCKET=trieve
export AWS_REGION=us-east-1 # Useful if your bucket is in s3
export STRIPE_API_KEY=sk_test_***************************************************************************************************
export STRIPE_WEBHOOK_SECRET=sk_test_***************************************************************************************************

helm/from-env.sh
```

This step generates a file in `helm/values.yaml`. It alllows you to modify the environment variables

### 4. Deploy the helm chart

```sh
gcloud container clusters get-credentials test-cluster
helm install -f helm/values.yaml trieve helm/
```

Ensure everything has been deployed with

```sh
kubectl get pods
```

### SubChart usage

Postgres, Redis, and qdrant are installed within this helm chart via a subchart. You can opt out of using the helm chart installation and using a managed service can be toggled via. the `useSubChart` parameters and setting the `uri` to a managed service. We reccomend placing at least Postgres and Redis out side of the helm chart and keeping Qdrant in the helm chart for a production usecase.

### 6. Set DNS records

First get the ingress addresses using

```sh
kubectl get ingress
```

You will get output that looks like this

```
NAME                CLASS   HOSTS                  ADDRESS        PORTS   AGE
ingress-chat        <none>  chat.domain.com        25.50.100.31   80      9s
ingress-dashboard   <none>  dashboard.domain.com   25.50.100.32   80      9s
ingress-search      <none>  search.domain.co       25.50.100.35   80      9s
ingress-server      <none>  api.domain.com         25.50.100.36   80      9s
```

Set CNAME's accordingly, we recommend using Cloudflare to CNAME and provision the SSL Cert
Once you set the ingress rules properly, the server should be able to properly deploy.

### 7. Setup/OIDC provider and Authorized Redirect URL'S

The last step is to setup an OIDC compliant server like `keycloak` for authentication and get an `issuerUrl` and `clientSecret`. This is how you do it within Keycloak.

A) Create a new realm called `trieve`
B) Go into Clients and create a new client called `trieve`.

Enable client authentication and set the following allowed redirect url's

- https://api.domain.com/*
- https://search.domain.com/*
- https://chat.domain.com/*
- https://dashboard.domain.com/*

You will get the client secret in the `Credentials` tab.

You will need to set the following values in the `helm/values.yaml` file, it should be prefilled already with default values

```
config:
  oidc:
    clientSecret: $OIDC_CLIENT_SECRET
    clientId: trieve
    issuerUrl: https://auth.domain.com/realms/trieve
    authRedirectUrl: https://auth.domain.com/realms/trieve/protocol/openid-connect/auth
```

### Testing

The fastest way to test is using the trieve cli

```
trieve login # Make sure to set the api url to https://api.domain.com
trieve dataset example
```

And there you have it. Your very own Trieve stack.
Happy hacking 🚀
