from angle_emb import AnglE, Prompts
import uvicorn
import torch
import numpy as np
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from transformers import AutoModelForMaskedLM, AutoTokenizer

# Create a Flask app
app = FastAPI()

doc_model_id = "naver/efficient-splade-VI-BT-large-doc"
doc_tokenizer = AutoTokenizer.from_pretrained(doc_model_id)
doc_model = AutoModelForMaskedLM.from_pretrained(doc_model_id)

query_model_id = "naver/efficient-splade-VI-BT-large-query"
query_tokenizer = AutoTokenizer.from_pretrained(query_model_id)
query_model = AutoModelForMaskedLM.from_pretrained(query_model_id)

angle = AnglE.from_pretrained("WhereIsAI/UAE-Large-V1", pooling_strategy="cls")
if torch.cuda.is_available():
    # Initialize CUDA device
    device = torch.device("cuda")
    angle = angle.cuda()
else:
    device = torch.device("cpu")
angle.set_prompt(Prompts.C)
# Tokenize sentences
query_model.to(device)
doc_model.to(device)


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
    tokens = tokenizer(text, return_tensors="pt")
    output = model(**tokens)
    logits, attention_mask = output.logits, tokens.attention_mask
    relu_log = torch.log(1 + torch.relu(logits))
    weighted_log = relu_log * attention_mask.unsqueeze(-1)
    max_val, _ = torch.max(weighted_log, dim=1)
    vec = max_val.squeeze()

    return vec


class EncodeRequest(BaseModel):
    input: str


@app.post("/encode")
async def encode(encodingRequest: EncodeRequest):
    # normalize embeddings
    sentence_embeddings = angle.encode(
        {"text": encodingRequest.input}, device=device, to_numpy=True
    )
    return JSONResponse(
        content={
            "embeddings": sentence_embeddings.tolist(),
            "status": 200,
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
            }
        )
    indices = vec.nonzero().squeeze().cpu().tolist()
    values = vec[indices].cpu().tolist()
    return JSONResponse(
        content={
            "embeddings": list(zip(indices, values)),
            "status": 200,
        }
    )


if __name__ == "__main__":
    uvicorn.run("embeddings:app", host="0.0.0.0", port=7070, reload=True)
