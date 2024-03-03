#!/bin/sh

model=jinaai/jina-embeddings-v2-small-en # nomic-ai/nomic-embed-text-v1
revision=main
volume=$PWD/data # share a volume with the Docker container to avoid downloading weights every run

docker run -it --gpus all -p 9999:80 -v $volume:/data --pull always ghcr.io/huggingface/text-embeddings-inference:86-1.1 --model-id $model --revision $revision
