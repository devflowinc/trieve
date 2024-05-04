#!/bin/bash

startpath=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source $startpath/parse_yaml.sh

function get_value() {
  local key=$1
  parse_yaml $startpath/../helm/values.yaml | awk -F = "\$1 ~ /$key/ { print substr(\$2, 2, length(\$2) - 2); }"
}

function assemble_ecr_prefix() {
  printf '%s.dkr.ecr.%s.amazonaws.com/' $(get_value accountId) $(get_value region)
}

export PREFIX=$(assemble_ecr_prefix)
bash $startpath/docker-build.sh
