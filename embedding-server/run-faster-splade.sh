#!/bin/sh

model=naver/efficient-splade-VI-BT-large-doc
revision=main
volume=$PWD/data # share a volume with the Docker container to avoid downloading weights every run

docker run -it --gpus all -p 7070:80 -v $volume:/data --pull always ghcr.io/huggingface/text-embeddings-inference:86-1.1 --model-id $model --revision $revision --pooling splade
