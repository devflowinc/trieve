#!/bin/bash
cat >service-account.yaml <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: ebs-csi-controller-sa
  namespace: kube-system
EOF
kubectl apply -f service-account.yaml
exit $?

