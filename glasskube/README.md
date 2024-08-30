# Trieve Glasskube Kubernetes installation


## 1. Start a new minikube cluster

[Minikube](https://minikube.sigs.k8s.io/docs/start/) can be installed via Homebrew or binary downloads.

```shell
minikube start -p trieve
```

## 2. Bootstrap glasskube

[Glasskube](https://glasskube.dev/docs/getting-started/install/) can also be installed via Homebrew or binary downloads.

```shell
glasskube bootstrap
```

## 3. Install trieve dependencies

Make sure you cloned the repository and navigated into the `glasskube/` folder.

```shell
kubectl apply -k dependencies 
```

Depending on your API Server, you might need to execute the command multiple times to install all dependencies.

## 4. Install trieve

Feel free to modify the kustomize based configurations as desired.

```shell
kubectl apply -k trieve 
```


## 5. Configure local DNS


Receive the IP address of your minikube cluster

```shell
minikube ip -p trieve
```

and make sure to add following entries into your `/etc/hosts` file (make sure you edit this file as root).

```txt
192.168.xxx.x   analytics.localtrieve.com
192.168.xxx.x   chat.localtrieve.com
192.168.xxx.x   api.localtrieve.com
192.168.xxx.x   dashboard.localtrieve.com
192.168.xxx.x   search.localtrieve.com
127.0.0.1       trieve-keycloak-service
```


## 6. Port-forward Keycloak

```shell
kubectl port-forward svc/trieve-keycloak-service 8080:8080
```


### 7. Open trieve

Open [dashboard.localtrieve.com](http://dashboard.localtrieve.com) in your browser.

## Optional commands

#### Scale down all embedding servers:

```shell
kubectl scale deployment trieve-embedding-bgem3  trieve-embedding-jina trieve-embedding-reranker trieve-embedding-splade-doc trieve-embedding-splade-query --replicas 0

```

#### Deleting the cluster

```shell
minikube delete -p trieve
```
