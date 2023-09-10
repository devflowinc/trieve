from sklearn.metrics.pairwise import cosine_similarity
from sklearn.feature_extraction.text import CountVectorizer
from pydantic import BaseModel
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from transformers import pipeline, AutoTokenizer
import nltk
import uvicorn
from fastapi.responses import HTMLResponse

nltk.download("punkt")
application = FastAPI()
application.add_middleware(
    CORSMiddleware,
    allow_origins="*",
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
summarizer = pipeline(
    "summarization",
    model="NotXia/pubmedbert-bio-ext-summ",
    tokenizer=AutoTokenizer.from_pretrained("NotXia/pubmedbert-bio-ext-summ"),
    trust_remote_code=True,
    device=0,
)


class AutoCutReqeuest(BaseModel):
    input: str


@application.post("/cut", response_class=HTMLResponse)
async def auto_cut(rq: AutoCutReqeuest):
    sentences = nltk.sent_tokenize(rq.input)
    summary = summarizer(
        {"sentences": sentences},
        strategy="count",
        strategy_args=round(len(sentences) * 0.3),
    )

    # Create a list to store the closest match for each target sentence

    for target_sentence in summary[0]:
        # Tokenize and process the target sentence
        # Create a list of sentences from the larger list

        # Vectorize the sentences using CountVectorizer
        vectorizer = CountVectorizer().fit_transform([target_sentence] + sentences)

        # Calculate cosine similarity between the target sentence and all sentences
        cosine_similarities = cosine_similarity(vectorizer)

        # Find the index of the closest match (excluding the target sentence itself)
        closest_match_index = cosine_similarities[0, 1:].argmax() + 1

        # Get the closest match sentence

        sentences[closest_match_index - 1] = (
            "<b><u>" + sentences[closest_match_index - 1] + "</u></b>"
        )
    return " ".join(sentences)


if __name__ == "__main__":
    uvicorn.run("application:application", reload=True)
