#!/usr/bin/python3
import sys
import requests
import os

request_timeout = 3


def completion_with_backoff(model_input):
    url = os.getenv("EMBEDDING_SERVER_URL")
    parameters = {"model": "dense-embeddings", "input": f"Search for {model_input}"}
    headers = {
        "Content-Type": "application/json",
    }
    try:
        response = requests.post(
            f"{url}/embeddings?api-version=2023-05-15",
            headers=headers,
            json=parameters,
        )
        response.raise_for_status()
        return [embedding["embedding"] for embedding in response.json()["data"]][0]
    except requests.exceptions.RequestException as e:
        raise Exception("Failed to send message to embedding server") from e
    except Exception as e:
        raise Exception("Failed to get text from embeddings") from e


def embed(text):
    if text == "":
        return "NULL"
    try:
        response = completion_with_backoff(text)
        return response
    except:
        return "ERROR"


for size in sys.stdin:
    try:
        # collect a batch for performance
        for row in range(0, int(size)):
            print(embed(sys.stdin.readline().strip()))
        sys.stdout.flush()
    except Exception as e:
        print(f"ERROR: {e}")
        sys.stdout.flush()
        continue
