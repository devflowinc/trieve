#!/bin/bash
account_id=$(aws sts get-caller-identity --query "Account" --output text)
echo $account_id
eksctl create addon --name aws-ebs-csi-driver --cluster trieve-02 --service-account-role-arn arn:aws:iam::$account_id:role/AmazonEKS_EBS_CSI_DriverRole --force
exit $?
