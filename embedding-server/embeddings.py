from transformers import AutoTokenizer, AutoModel
import uvicorn
import torch
import numpy as np
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from pydantic import BaseModel

# Create a Flask app
app = FastAPI()

if torch.cuda.is_available():
    # Initialize CUDA device
    device = torch.device("cuda")
else:
    device = torch.device("cpu")

tokenizer = AutoTokenizer.from_pretrained("BAAI/bge-large-en")
model = AutoModel.from_pretrained("BAAI/bge-large-en")
model.to(device)
# Tokenize sentences

def inference(words: str):
    encoded_input = tokenizer(
        words,
        padding=True,
        truncation=True,
        max_length=512,
        add_special_tokens=True,
        return_tensors="pt",
    ).to(device)
    # for s2p(short query to long passage) retrieval task, add an instruction to query (not add instruction for passages)
    # encoded_input = tokenizer([instruction + q for q in queries], padding=True, truncation=True, return_tensors='pt')

    # Compute token embeddings
    with torch.no_grad():
        model_output = model(**encoded_input)
        # Perform pooling. In this case, cls pooling.
        sentence_embeddings = model_output[0][:, 0]
    # normalize embeddings
    sentence_embeddings = torch.nn.functional.normalize(sentence_embeddings, p=2, dim=1)
    return {"embeddings": np.array(sentence_embeddings.cpu())[0].tolist()}

@app.get("/")
async def health():
    return {"message": "hello embeddings" }


class EncodeRequest(BaseModel):
    input: str

@app.post("/encode")
async def encode(encodingRequest: EncodeRequest):
    encoded_input = tokenizer(
        encodingRequest.input,
        padding=True,
        truncation=True,
        max_length=512,
        add_special_tokens=True,
        return_tensors="pt",
    ).to(device)
    # for s2p(short query to long passage) retrieval task, add an instruction to query (not add instruction for passages)
    # encoded_input = tokenizer([instruction + q for q in queries], padding=True, truncation=True, return_tensors='pt')

    # Compute token embeddings
    with torch.no_grad():
        model_output = model(**encoded_input)
        # Perform pooling. In this case, cls pooling.
        sentence_embeddings = model_output[0][:, 0]
    # normalize embeddings
    sentence_embeddings = torch.nn.functional.normalize(sentence_embeddings, p=2, dim=1)
    return JSONResponse(content={"embeddings": np.array(sentence_embeddings.cpu())[0].tolist(), "status": 200})


if __name__ == "__main__":
   uvicorn.run("embeddings:app", host="0.0.0.0", port=7070, reload=True)

