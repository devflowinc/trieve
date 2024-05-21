from transformers import AutoModel
from fastapi import FastAPI, responses
from typing import Union, Annotated, Literal, List
from pydantic import BaseModel
from numpy.linalg import norm
from dotenv import load_dotenv
from os import environ

load_dotenv()

model_name = environ["MODEL"]
model = AutoModel.from_pretrained(model_name, trust_remote_code=True, cache_dir="./models", device_map = 'cuda')

app = FastAPI()

class OpenAIEmbeddingInput(BaseModel):
    input: Union[
        List[str],
        str
    ]

class _EmbeddingObject(BaseModel):
    object: Literal["embedding"] = "embedding"
    embedding: list[float]
    index: int

class OpenAIEmbeddingResult(BaseModel):
    data: List[_EmbeddingObject]
    object: Literal["embedding"] = "embedding"


@app.post(
    "/embeddings",
    response_class=responses.ORJSONResponse,
)
def embed(data: OpenAIEmbeddingInput):
    print(data.input)
    embeddings = model.encode(
        data.input
    )

    print(embeddings)
    if type(data.input) == str:
        return {
            "data": [{
                "embedding": embeddings.tolist(),
                "object": "embedding",
                "index": 0
            }],
            "object": "embedding",
            "model": model_name,
        }
    else:
        return {
            "data": [{
                "embedding": embedding,
                "object": "embedding",
                "index": i,
                } for (i, embedding) in enumerate(embeddings.tolist())
            ],
            "object": "embedding",
            "model": model_name,
        }
