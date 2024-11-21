# Deploying 

### Docker Compose

Use the docker-compose-prod.yaml file to deploy the application.

```bash
docker compose up -f docker-compose-prod.yaml -d
```

You can either chose to build locally or pull the pre-built images from the docker hub.

#### Build Options
##### Build On Machine:

```bash
docker compose up -f docker-compose-prod.yaml -d --build
```

##### Use Pre-built Images:
```bash
docker compose up -f docker-compose-prod.yaml -d --pull always
```

#### Setup Caddy reverse proxy (optional)

Setup a Caddyfile with the following content:

```bash
# Global options
{
    email developer@example.com
}

# Define a site block for pdftomd.example.com
pdftomd.example.com {
    reverse_proxy localhost:8081
}
```

Start the caddy reverse proxy. This should also handle your ssl

```bash
sudo systemctl reload caddy.service
```

### Kubernetes

```bash
kubectl apply -f k8s/
```

You can now access pdf2md within the kubernetes cluster at `http://pdf2md.default.svc.cluster.local`
To access it from outside the cluster:
- You can use a service of type `LoadBalancer` or `NodePort`.
- You can setup an Ingress (by default, the ingress is enabled in the k8s files).

#### Setup Ingress (optional)

```bash
kubectl get ingress
```

##### GKE Ingress

For gke ingress, you need to set add `kubernetes.io/ingress.class` annotation to `gce` in the ingress yaml file.

Here is an example of how it looks:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: pdf2md-ingress
  annotations:
    kubernetes.io/ingress.class: "gce"
spec:
  defaultBackend:
    service:
      name: pdf2md-api
      port:
        number: 80
```

NAME             CLASS    HOSTS   ADDRESS          PORTS   AGE
pdf2md-ingress   <none>   *       34.107.134.128   80      4h33m
```

##### EKS Ingress

For eks you need to set kubernetes.io/ingress.class to `alb` and set `spec.ingressClassName` to `alb` in the ingress yaml file.

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: pdf2md-ingress
  annotations:
    kubernetes.io/ingress.class: "alb"
spec:
  ingressClassName: "alb"
  defaultBackend:
    service:
      name: pdf2md-api
      port:
        number: 80
```
