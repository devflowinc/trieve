from typing import Annotated, List
from fastapi import FastAPI, Header, Response
import time
import requests
from pydantic import BaseModel

class TrackingRequest(BaseModel):
    ids: List[str]


app = FastAPI()

@app.post("/monitor_tracking")
def monitor_tracking(tracking_req: TrackingRequest, response: Response, authorization: Annotated[str | None, Header()] = None, tr_dataset: Annotated[str | None, Header()] = None):

    if authorization and tr_dataset:
        for _ in range(300):
            try:
                response = requests.post("https://api.trieve.ai/api/chunks/tracking", headers={"Content-Type": "application/json", "Authorization": authorization, "TR-Dataset": tr_dataset}, json={"tracking_ids": tracking_req.ids})
                if response.status_code == 200:
                    response.status_code = 200
                    return {"status": "Found"}
            except:
                pass

            time.sleep(0.01)

    response.status_code = 404
    return {"status": "not found"}

@app.post("/monitor")
def monitoring(tracking_req: TrackingRequest, response: Response, authorization: Annotated[str | None, Header()] = None, tr_dataset: Annotated[str | None, Header()] = None):

    if authorization and tr_dataset:
        for _ in range(300):
            try:
                response = requests.post("https://api.trieve.ai/api/chunks", headers={"Content-Type": "application/json", "Authorization": authorization, "TR-Dataset": tr_dataset}, json={"ids": tracking_req.ids})
                if response.status_code == 200:
                    response.status_code = 200
                    return {"status": "Found"}
            except:
                pass

            time.sleep(0.01)

    response.status_code = 404
    return {"status": "not found"}
