#!/bin/bash
EKS_CLUSTER_NAME=trieve
# get managed node group name (assuming there's only one node group)
GROUP_NAME=$(aws eks list-nodegroups --cluster-name $EKS_CLUSTER_NAME \
  --query nodegroups --out text)
# fetch role arn given node group name
ROLE_ARN=$(aws eks describe-nodegroup --cluster-name $EKS_CLUSTER_NAME \
  --nodegroup-name $GROUP_NAME --query nodegroup.nodeRole --out text)
# extract just the name part of role arn
ROLE_NAME=${ROLE_ARN##*/}
POLICY_ARN=$(aws iam list-policies --query 'Policies[?PolicyName==`AllowExternalDNSUpdates`].Arn' --output text)
aws iam attach-role-policy --role-name $ROLE_NAME --policy-arn $POLICY_ARN
exit $?
