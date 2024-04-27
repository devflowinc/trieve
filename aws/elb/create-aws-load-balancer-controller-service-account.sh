#!/bin/bash
source ./functions.sh
oidc_id=$(get_oidc)
region=us-west-1
account_id=$(aws sts get-caller-identity --query "Account" --output text)
cat >aws-load-balancer-controller-service-account.yaml <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  labels:
    app.kubernetes.io/component: controller
    app.kubernetes.io/name: aws-load-balancer-controller
  name: aws-load-balancer-controller
  namespace: kube-system
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::$account_id:role/AmazonEKSLoadBalancerControllerRole
EOF
kubectl apply -f aws-load-balancer-controller-service-account.yaml
