#!/bin/bash
aws iam create-role --role-name AmazonEKS_EBS_CSI_DriverRole --assume-role-policy-document file://aws-ebs-csi-driver-trust-policy.json --description "EBS CSI DriverRole"
exit $?
