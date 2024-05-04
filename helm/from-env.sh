#!/bin/bash
startpath=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cat $startpath/values.yaml.tpl | envsubst | tee $startpath/values.yaml

