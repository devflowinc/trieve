# Self hosting guide

We currently offer 3 ways to self host Trieve.
- 1 Docker Compose
- 2 Kubernetes (AWS EKS)
- 3 Kubernetes (GCP GKE)

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
KEYCLOAK_HOST="auth.yourdomain.com"

VITE_API_HOST=https://api.yourdomain.com/api
VITE_SEARCH_UI_URL=https://search.yourdomain.com
VITE_CHAT_UI_URL=https://chat.yourdomain.com
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

## AWS EKS

Things you need
- Domain name
- An allowance for at least 8vCPU for G and VT instances
- helm cli
- aws cli
- kubectl
- k9s (optional)

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

### 3 Create values.yaml

```sh
export AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query "Account" --output text)
export AWS_REGION=us-east-1

export SENTRY_CHAT_DSN=https://********************************@sentry.trieve.ai/6
export ENVIRONMENT=aws
export DOMAIN=ansear.ai # Only used for local
export EXTERNAL_DOMAIN=ansear.ai
export DASHBOARD_URL=https://dashboard.trieve.ai
export KEYCLOAK_ADMIN=admin
export KEYCLOAK_PASSWORD=admin
export MINIO_ROOT_USER=admin
export MINIO_ROOT_PASSWORD=admin
export SALT=goodsaltisveryyummy
export SECRET_KEY=1234512345123451234512345123451234512345123451234512345123451234512345123451234h
export ADMIN_API_KEY=asdasdasdasdasd
export OIDC_CLIENT_SECRET=YllmLDTy67MbsUBrUAWvQ7z9aMq0QcKx
export ISSUER_URL=https://oidc.trieve.ai
export AUTH_REDIRECT_URL=https://oidc.trieve.ai/realms/trieve/protocol/openid-connect/auth
export REDIRECT_URL=https://oidc.trieve.ai/realms/trieve/protocol/openid-connect/auth
export SMTP_RELAY=smtp.gmail.com
export SMTP_USERNAME=trieve@gmail.com
export SMTP_PASSWORD=pass************
export SMTP_EMAIL_ADDRESS=triever@gmail.com
export LLM_API_KEY=sk-or-v1-**************************************************************** # Open Router API KEY
export OPENAI_API_KEY=sk-************************************************ # OPENAI API KEY
export OPENAI_BASE_URL=https://api.openai.com/v1
export AWS_REGION=us-west-1
export S3_ENDPOINT=http://minio.default.svc.cluster.local:9000
export S3_ACCESS_KEY=ZaaZZaaZZaaZZaaZZaaZ
export S3_SECRET_KEY=ssssssssssssssssssssTTTTTTTTTTTTTTTTTTTT
export S3_BUCKET=trieve
export STRIPE_API_KEY=sk_test_***************************************************************************************************
export STRIPE_WEBHOOK_SECRET=sk_test_***************************************************************************************************

helm/from-env.sh
```

This step generates a file in `helm/values.yaml`. It alllows you to modify the environment variables

### 4. Deploy the helm chart

```sh
aws eks update-kubeconfig --region us-west-1 --name trieve-cluster
helm install -f helm/values.yaml trieve helm/
```

Ensure everything has been deployed with

```sh
kubectl get pods
```

Notice the services that require ecr don't stand up. We need to build and push ecr docker images

### 4. Build and push Dockerimages 

```sh
bash scripts/login-docker.sh
bash scripts/docker-build-eks.sh
```

After this all other services should be working and verifieid by. 

```
kubectl get pods
```

`server` isn't up because CNAME and ssl are needed.

### 6. Set CNAME links

First get the ingress addresses using

```sh
kubectl get ingress
```

You will get output that looks like this

```
NAME                CLASS   HOSTS   ADDRESS                                                                  PORTS   AGE
ingress-chat        alb     *       k8s-default-ingressc-033d55ca4e-253753726.us-west-1.elb.amazonaws.com    80      9s
ingress-dashboard   alb     *       k8s-default-ingressd-50d3c72d7f-2039058451.us-west-1.elb.amazonaws.com   80      9s
ingress-keycloak    alb     *       k8s-default-ingressk-d51c2ca03e-1603083207.us-west-1.elb.amazonaws.com   80      9s
ingress-s3          alb     *       k8s-default-ingresss-42f1c8646f-1588406796.us-west-1.elb.amazonaws.com   80      9s
ingress-search      alb     *       k8s-default-ingresss-2007ac265d-1873944939.us-west-1.elb.amazonaws.com   80      9s
ingress-server      alb     *       k8s-default-ingresss-cb909f388e-797764884.us-west-1.elb.amazonaws.com    80      9s
```

Set CNAME's accordingly, we recommend using Cloudflare to CNAME and provision the SSL Cert

```
ingress-chat            chat.domain.
ingress-dashboard       dashboard.domain.
ingress-keycloak        keycloak.domain.
ingress-s3              data.domain.
ingress-search          search.domain.
ingress-server          api.domain.
```

Once you set the ingress rules properly, the server should be able to properly deploy.

### 7. Set Keycloak Authorized Redirect URL'S

Change the realm from `master` to `trieve`

Go to Realm Settings -> Themes and set the login theme as arguflow

Finally go into Clients -> vault and add the following redirect url's
- https://api.domain.com/*
- https://search.domain.com/*
- https://chat.domain.com/*
- https://dashboard.domain.com/*

### Testing

The fastest way to test is using the trieve cli
```
trieve init
trieve dataset example
```

And there you have it. Your very own trieve stack.
Happy hacking 🚀


## GCP GKE

Things you need
- Domain name
- helm cli
- google cloud cli
- kubectl
- k9s (optional)

```sh
aws configure
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
export AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query "Account" --output text)
export AWS_REGION=us-east-1

export SENTRY_CHAT_DSN=https://********************************@sentry.trieve.ai/6
export ENVIRONMENT=gcloud
export DOMAIN=ansear.ai # Only used for local
export EXTERNAL_DOMAIN=ansear.ai
export DASHBOARD_URL=https://dashboard.trieve.ai
export KEYCLOAK_ADMIN=admin
export KEYCLOAK_PASSWORD=admin
export MINIO_ROOT_USER=admin
export MINIO_ROOT_PASSWORD=admin
export SALT=goodsaltisveryyummy
export SECRET_KEY=1234512345123451234512345123451234512345123451234512345123451234512345123451234h
export ADMIN_API_KEY=asdasdasdasdasd
export OIDC_CLIENT_SECRET=YllmLDTy67MbsUBrUAWvQ7z9aMq0QcKx
export ISSUER_URL=https://oidc.trieve.ai
export AUTH_REDIRECT_URL=https://oidc.trieve.ai/realms/trieve/protocol/openid-connect/auth
export REDIRECT_URL=https://oidc.trieve.ai/realms/trieve/protocol/openid-connect/auth
export SMTP_RELAY=smtp.gmail.com
export SMTP_USERNAME=trieve@gmail.com
export SMTP_PASSWORD=pass************
export SMTP_EMAIL_ADDRESS=triever@gmail.com
export LLM_API_KEY=sk-or-v1-**************************************************************** # Open Router API KEY
export OPENAI_API_KEY=sk-************************************************ # OPENAI API KEY
export OPENAI_BASE_URL=https://api.openai.com/v1
export AWS_REGION=us-west-1
export S3_ENDPOINT=http://minio.default.svc.cluster.local:9000
export S3_ACCESS_KEY=ZaaZZaaZZaaZZaaZZaaZ
export S3_SECRET_KEY=ssssssssssssssssssssTTTTTTTTTTTTTTTTTTTT
export S3_BUCKET=trieve
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

Notice the services that require ecr don't stand up. We need to build and push ecr docker images

### 4. Build and push Dockerimages 

```sh
bash scripts/login-docker.sh
bash scripts/docker-build-eks.sh
```

After this all other services should be working and verifieid by. 

```
kubectl get pods
```

`server` isn't up because CNAME and ssl are needed.

### 6. Set CNAME links

First get the ingress addresses using

```sh
kubectl get ingress
```

You will get output that looks like this

```
NAME                CLASS   HOSTS				   ADDRESS        PORTS   AGE
ingress-chat        <none>  chat.domain.com        20.50.100.31   80      9s
ingress-dashboard   <none>  dashboard.domain.com   20.50.100.32   80      9s
ingress-keycloak    <none>  oidc.domain.com		   20.50.100.33   80      9s
ingress-s3          <none>  data.domain.com        20.50.100.34   80      9s
ingress-search      <none>  search.domain.co       20.50.100.35   80      9s
ingress-server      <none>  api.domain.com         20.50.100.36   80      9s
```

Set CNAME's accordingly, we recommend using Cloudflare to CNAME and provision the SSL Cert







ingress-chat            chat.domain.
ingress-dashboard       dashboard.domain.
ingress-keycloak        keycloak.domain.
ingress-s3              data.domain.
ingress-search          search.domain.
ingress-server          api.domain.

```
ingress-chat            chat.domain.
ingress-dashboard       dashboard.domain.
ingress-keycloak        oidc.domain.
ingress-s3              data.domain.
ingress-search          search.domain.
ingress-server          api.domain.
```

Once you set the ingress rules properly, the server should be able to properly deploy.

### 7. Set Keycloak Authorized Redirect URL'S

Change the realm from `master` to `trieve`

Go to Realm Settings -> Themes and set the login theme as arguflow

Finally go into Clients -> vault and add the following redirect url's
- https://api.domain.com/*
- https://search.domain.com/*
- https://chat.domain.com/*
- https://dashboard.domain.com/*

### Testing

The fastest way to test is using the trieve cli
```
trieve init
trieve dataset example
```

And there you have it. Your very own trieve stack.
Happy hacking 🚀
