# Trieve Glasskube Kubernetes installation


## 1. Start a new minikube cluster

[Minikube](https://minikube.sigs.k8s.io/docs/start/) can be installed via Homebrew or binary downloads.
Also make sure to enable the ingress and metrics plugin

```shell
minikube start -p trieve
minikube addons enable metrics-server -p trieve
minikube addons enable ingress -p trieve
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

## Optional commands & debugging info

#### Working on low CPU machines

You can scale down all embedding servers to reduce the memory and cpu usage of trieve.

```shell
kubectl scale deployment trieve-embedding-bgem3 trieve-embedding-jina trieve-embedding-reranker trieve-embedding-splade-doc trieve-embedding-splade-query --replicas 0
```

If you still want to do end-to-end test you can only start the `trieve-embedding-bgem3`
and configure the dataset in a specific way:

1. Create a new dataset
2. Disable full text search
3. Import test data

After that the ingestion worker will orchestrate the ingestion.

You can see the ingestion progress by connecting to the redis pod and execute the `redis-cli` and
monitor progress by looking at the queues `llen ingestion` and `llen processing`.

After ingestion is done you can search the dataset by only setting the search type to semantic so
only bgem3 is used.

#### Qdrant

Qdrant throws some weird exceptions during the initial database migrations, but keeps working normally after.

You can monitor the status of the collections by looking at the Qdrant ui:

```shell
glasskube open qdrant
```

You can fetch the API key with:

```
kubectl get secret trieve-qdrant-qdrant-apikey -o jsonpath='{.data.api-key}' | base64 -d
```

#### ClickHouse

In order to test if ClickHouse works you can connect to clickhouse via:

```shell
kubectl exec -it chi-trieve-clickhouse-cluster1-0-0-0 -- clickhouse-client
```

The password is currently `password` but will be changed in the future.

In order to see if the custom embedding function works you can execute:

```
select embed_p('test');
```


#### Deleting the cluster

```shell
minikube delete -p trieve
```
