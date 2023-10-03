FROM python:3.10
WORKDIR /app
COPY requirements.txt .
RUN pip3 install -r requirements.txt

COPY embeddings.py /app/embeddings.py
EXPOSE 7070
CMD ["uvicorn", "embeddings:app", "--proxy-headers", "--host", "0.0.0.0", "--port", "7070"]
