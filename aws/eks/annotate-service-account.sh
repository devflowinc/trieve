#!/bin/bash
namespace=kube-system
service_account=ebs-csi-controller-sa
account_id=$(aws sts get-caller-identity --query "Account" --output text)
kubectl annotate serviceaccount -n $namespace $service_account eks.amazonaws.com/role-arn=arn:aws:iam::$account_id:role/AmazonEKS_EBS_CSI_DriverRole22 --overwrite
exit $?
