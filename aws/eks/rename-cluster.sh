#!/bin/bash
declare -a files=("add-driver.sh" "create-cluster.sh" "cluster.yaml" "create-identity-provider.sh" "make-trust-policy-file.sh")
for file in "${files[@]}"
do
  sed -i "s/$1/$2/g" $file
done
 
