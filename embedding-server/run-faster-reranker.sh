#!/bin/sh

model=BAAI/bge-reranker-large
revision=refs/pr/4
volume=$PWD/data # share a volume with the Docker container to avoid downloading weights every run

docker run --gpus all -p 7777:80 -v $volume:/data --pull always ghcr.io/huggingface/text-embeddings-inference:86-1.1 --model-id $model --revision $revision

