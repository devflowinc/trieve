#!/bin/bash
account_id=$(aws sts get-caller-identity --query "Account" --output text)
namespace=kube-system
function start_path() {
  echo $( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
}
startpath=$(start_path)
source $startpath/functions.sh
cluster=$(get_cluster)
eksctl create iamserviceaccount \
  --cluster=$cluster \
  --namespace=$namespace \
  --name=aws-load-balancer-controller \
  --attach-policy-arn=arn:aws:iam::${account_id}:policy/AWSLoadBalancerControllerIAMPolicy \
  --override-existing-serviceaccounts \
  --approve

exit $?
