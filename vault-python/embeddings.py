# Load model directly
from transformers import AutoTokenizer, AutoModel
import torch
import numpy as np
from flask import Flask, request, jsonify

# Create a Flask app
app = Flask(__name__)


if torch.cuda.is_available():
    # Initialize CUDA device
    device = torch.device("cuda")
else:
    device = torch.device("cpu")

tokenizer = AutoTokenizer.from_pretrained("BAAI/bge-large-en")
model = AutoModel.from_pretrained("BAAI/bge-large-en")
model.to(device)
# Tokenize sentences

@app.route("/")
def health():
    return "hello", 200

@app.route("/encode", methods=["POST"])
def encode():
    data = request.json
    encoded_input = tokenizer(
        data["input"],
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
    return jsonify({"embeddings": np.array(sentence_embeddings.cpu())[0].tolist()})


if __name__ == "__main__":
    # Run the app on localhost (127.0.0.1) and port 5000
    app.run(host="127.0.0.1", port=5000)
