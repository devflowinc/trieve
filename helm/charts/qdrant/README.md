# Qdrant helm chart

[Qdrant documentation](https://qdrant.tech/documentation/)

## TLDR

```bash
helm repo add qdrant https://qdrant.github.io/qdrant-helm
helm repo update
helm upgrade -i your-qdrant-installation-name qdrant/qdrant
```

## Description

This chart installs and bootstraps a Qdrant instance.

## Prerequisites

- Kubernetes v1.24+ (as you need grpc probe)
- Helm
- PV provisioner (by the infrastructure)

## Installation & Setup

You can install the chart from source via:

```bash
helm upgrade -i your-qdrant-installation-name charts/qdrant
```

Uninstall via:

```bash
helm uninstall your-qdrant-installation-name
```

Delete the volume with

```bash
kubectl delete pvc -l app.kubernetes.io/instance=your-qdrant-installation-name
```

## Configuration

For documentation of the settings please refer to [Qdrant Configuration File](https://github.com/qdrant/qdrant/blob/master/config/config.yaml)
All of these configuration options could be overwritten under config in `values.yaml`.
A modification example is provided there.

### Distributed setup

Running a distributed cluster just needs a few changes in your `values.yaml` file.
Increase the number of replicas to the desired number of nodes and set `config.cluster.enabled` to true.

Depending on your environment or cloud provider you might need to change the service in the `values.yaml` as well.
For example on AWS EKS you would need to change the `cluster.type` to `NodePort`.

### Snapshot Restoration

Disclaimer: Snapshot restoration is only supported for single qdrant node setups

To restore a snapshot create a Persistent Volume and a Persistent Volume Claim using a storage class according to your setup, copy the snapshots to the PV, enable snapshot restoration along with the snapshot file names and pvc name in values.yaml file and run the helm install command.

Example EBS pv, pvc and volume creation command is added in examples directory
Note: Make sure volume is on the same region and availability zone as where qdrant is going to be installed.

### Metrics endpoints

Metrics are available through rest api (default port set to 6333) at `/metrics`

Refer to [qdrant metrics configuration](https://qdrant.tech/documentation/telemetry/#metrics) for more information.
