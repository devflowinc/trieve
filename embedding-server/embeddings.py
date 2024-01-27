from math import trunc
from typing import Optional
import uvicorn
import torch
import numpy as np
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from transformers import AutoModelForMaskedLM, AutoTokenizer, AutoModel
from sentence_transformers.cross_encoder import CrossEncoder
import huggingface_hub
import os
from dotenv import load_dotenv

load_dotenv()
hf_token = os.environ.get('HF_TOKEN')
huggingface_hub.login(token=hf_token)
embedding_model = AutoModel.from_pretrained('jinaai/jina-embeddings-v2-base-en', token=hf_token, trust_remote_code=True) # trust_remote_code is needed to use the encode method

# Create a Flask app
app = FastAPI()

doc_model_id = "naver/efficient-splade-VI-BT-large-doc"
doc_tokenizer = AutoTokenizer.from_pretrained(
    doc_model_id, use_fast=True, truncation=True, max_length=512
)
doc_model = AutoModelForMaskedLM.from_pretrained(doc_model_id)

query_model_id = "naver/efficient-splade-VI-BT-large-query"
query_tokenizer = AutoTokenizer.from_pretrained(
    query_model_id, use_fast=True, truncation=True, max_length=512
)
query_model = AutoModelForMaskedLM.from_pretrained(query_model_id)

cross_encoder_model_id = "BAAI/bge-reranker-large"
cross_encoder_model = CrossEncoder(cross_encoder_model_id)

if torch.cuda.is_available():
    device = torch.device("cuda")
else:
    device = torch.device("cpu")

# Tokenize sentences
query_model.to(device)
doc_model.to(device)
embedding_model.to(device)

# Create a Flask app
app = FastAPI()

@app.get("/")
async def health():
    return {"message": "hello embeddings"}


def compute_vector(text, tokenizer, model):
    """
    Computes a vector from logits and attention mask using ReLU, log, and max operations.

    Args:
    logits (torch.Tensor): The logits output from a model.
    attention_mask (torch.Tensor): The attention mask corresponding to the input tokens.

    Returns:
    torch.Tensor: Computed vector.
    """
    tokens = tokenizer(text[:512], return_tensors="pt")
    tokens = tokens.to(device)
    output = model(**tokens)
    logits, attention_mask = output.logits, tokens.attention_mask
    relu_log = torch.log(1 + torch.relu(logits))
    weighted_log = relu_log * attention_mask.unsqueeze(-1)
    max_val, _ = torch.max(weighted_log, dim=1)
    vec = max_val.squeeze()

    return vec


class EncodeRequest(BaseModel):
    input: str
    model: str
    encoding_format: Optional[str] = None
    user: Optional[str] = None


@app.post("/embeddings")
async def encode(encodingRequest: EncodeRequest):
    sentence_embeddings = embedding_model.encode([encodingRequest.input])

    return JSONResponse(
        content={
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": list(sentence_embeddings[0].astype(float)),
                    "index": 0,
                }
            ],
            "model": encodingRequest.model,
            "usage": {
                "prompt_tokens": 0,
                "total_tokens": 0,
            },
        }
    )


class SparseEncodeRequest(BaseModel):
    input: str
    encode_type: str


@app.post("/sparse_encode")
async def sparse_encode(encodingRequest: SparseEncodeRequest):
    vec = []
    if encodingRequest.encode_type == "doc":
        vec = compute_vector(
            encodingRequest.input, model=doc_model, tokenizer=doc_tokenizer
        )
    elif encodingRequest.encode_type == "query":
        vec = compute_vector(
            encodingRequest.input, model=query_model, tokenizer=query_tokenizer
        )
    else:
        return JSONResponse(
            content={
                "embeddings": [],
                "status": 400,
            },
            status_code=400
        )
    indices = vec.nonzero().squeeze().cpu().tolist()
    values = vec[indices].cpu().tolist()

    if type(indices) != list:
        indices = [indices]
    if type(values) != list:
        values = [values]

    return JSONResponse(
        content={
            "embeddings": list(zip(indices, values)),
            "status": 200,
        },
        status_code=200
    )


class ReRankRequest(BaseModel):
    query: str
    docs: list[str]


@app.post("/rerank")
async def rerank(rerankRequest: ReRankRequest):
    combined_docs = [[rerankRequest.query, doc] for doc in rerankRequest.docs]
    doc_scores = cross_encoder_model.predict(combined_docs)
    sim_scores_argsort = reversed(np.argsort(doc_scores))
    reranked_docs = [rerankRequest.docs[i] for i in sim_scores_argsort]
    return JSONResponse(
        content={
            "docs": reranked_docs,
            "status": 200,
        },
        status_code=200
    )


if __name__ == "__main__":
    uvicorn.run("embeddings:app", host="0.0.0.0", port=7070, reload=True)
