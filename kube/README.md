# Setup Minicube locally

1. Set up configs
- Run `cat kube/env-vals/configmap.yaml.local < kube/env-vals/configmap.yaml`
- Fill in values with your api keys and secrets except for the keycloak values (we will get the keycloak ip later)

2. Download minikube and kubectl
- Use this [guide](https://minikube.sigs.k8s.io/docs/start/) to download minikube 
- Use this [guide](https://kubernetes.io/docs/tasks/tools/) to install kubectl
- Run `minikube start` to start up your local kube cluster
- Run `	minikube addons enable metrics-server` to set up the metrics

3. Apply all kubectl configs
- Run `kubectl apply -f kube/env-vals/configmap.yaml` to include the configs for the containers
- Then run `kubectl apply -f` for all of the files within the `kube/services` directory except for the `server.yaml` config

4. Get keycloak ip
- Run `minicube service keycloak --url` to get the url of the keycloak service
- Replace the `<keycloak-ip>` values in the `configmap.yaml` to that url
- Rerun `kubectl apply -f kube/env-vals/configmap.yaml`
- Then run `kubectl apply -f kube/services/server.yaml` to start the server
5. View minikube dashboard
- Run `minikube dashboard` to view the dashboard and ensure all of the deployments were successful
6. Port-forward server to access locally
- Run `kubectl port-forward service/web-server-service 8090:8090` to access the server locally

## Using GPU inference
1. Install Nvidia-docker
- Follow this [guide](https://minikube.sigs.k8s.io/docs/tutorials/nvidia/) to set up `nvidia-docker` on your system 
- Run `minikube delete` if you had started it prior
- Run `minikube start --driver docker --container-runtime docker --gpus all --memory 14000 --cpus 4`
- Then run your gpu config file