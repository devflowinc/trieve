#!/bin/bash
cluster=trieve

function start_path() {
  echo $( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
}
startpath=$(start_path)
source $startpath/functions.sh
cluster=$(get_cluster)
cd $startpath/../../k8s/helm
#helm repo add eks https://aws.github.io/eks-charts || true
helm upgrade aws-load-balancer-controller eks/aws-load-balancer-controller -n kube-system --set clusterName=$cluster --set serviceAccount.create=false --set serviceAccount.name=aws-load-balancer-controller
exit $?
