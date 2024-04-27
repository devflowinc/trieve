#!/bin/bash

cluster=trieve-02
function get_cluster() {
  echo $cluster
}
function get_oidc() {
  aws eks describe-cluster --name $(get_cluster) --query "cluster.identity.oidc.issuer" --output text | cut -d '/' -f 5
}

