#!/bin/bash

prefix=${PREFIX:-localhost:5001/}
startpath=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source $startpath/parse_yaml.sh

function get_value() {
  local key=$1
  parse_yaml $startpath/../helm/values.yaml | awk -F = "\$1 ~ /$key/ { print substr(\$2, 2, length(\$2) - 2); }"
}

function tag_and_push() {
  local tag=$(get_value containers_${1}_tag)
  local fulltag=${prefix}${1}:${tag}
  echo $fulltag
  docker tag trieve/$1 $fulltag
  docker push $fulltag
}

function docker_build() {
  docker build --progress=plain $*
}

function build_images() {

  cd $startpath/../docker/keycloak
  docker_build -t trieve/keycloak .
  cd $startpath/../docker/minio
  docker_build -t trieve/minio .
  cd $startpath/../docker/postgres
  docker_build -t trieve/postgres .
  cd $startpath/../docker/tika
  docker_build -t trieve/tika .
  cd $startpath/../docker/mc
  docker_build -t trieve/mc .
  cd $startpath/../chat
  docker_build -t trieve/chat .
  cd $startpath/../search
  docker_build -t trieve/search .
  cd $startpath/../dashboard
  docker_build -t trieve/dashboard .
  cd $startpath/../server
  docker_build -t trieve/server -f Dockerfile.server . 
  # docker_build -t trieve/ingest -f Dockerfile.microservice .
}

function tag_images() {
  tag_and_push keycloak
  tag_and_push minio
  tag_and_push postgres
  tag_and_push tika
  tag_and_push mc
  tag_and_push server
  tag_and_push ingest
  tag_and_push chat
  tag_and_push search
  tag_and_push dashboard
}

build_images
tag_images
