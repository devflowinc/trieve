#!/bin/bash
function trieve_podname() {
  local pod=$1
  local cmd=${@:3}
  local podname=$(kubectl get pod -l "app.kubernetes.io/name==$pod" --field-selector=status.phase==Running -o jsonpath="{.items[0].metadata.name}")
  echo $podname
}
