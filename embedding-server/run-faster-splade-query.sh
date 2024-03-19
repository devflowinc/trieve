#!/bin/sh

model=naver/efficient-splade-VI-BT-large-query
revision=main
volume=$PWD/data # share a volume with the Docker container to avoid downloading weights every run

docker run -it --gpus all -p 7071:80 -v $volume:/data --pull always arguflow/text-embeddings --model-id $model --revision $revision --pooling splade
