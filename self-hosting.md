### 1 Setup AWS CLI

```sh
aws configure
```

Request Increase in on demand instances for G family

### 2 Provision Terraform

Using aws

```sh
cd terraform
terraform init
terraform apply
```

This should provision an eks cluster with elb and ebs drivers

### 3 Create values.yaml

```sh
export AWS_ACCOUNT_ID='"$(aws sts get-caller-identity --query "Account" --output text)"'
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
