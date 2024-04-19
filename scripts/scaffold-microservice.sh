#!/bin/bash

startpath=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
export NAME=$1

cd $startpath/template
for i in $(ls *.yaml); do
  cat $i | envsubst | tee $startpath/../helm/templates/${NAME}-${i}
  echo "$i > $startpath/../helm/templates/${NAME}-${i}"
done
