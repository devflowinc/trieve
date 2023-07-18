import json
import requests


URL = "http://localhost:8091/get_url_content"

with open('sources.json') as f:
    sources = json.load(f)
    for i, source in enumerate(sources['cards']):
        data = { "url": source['link'] }
        response = requests.post(URL, data=json.dumps(data))
        if response.status_code != 200:
            print()
            print(f"{i + 1}: {source['link']} - {response.status_code} {response.text}")
        elif response.text == "":
            print()
            print(f"{i + 1}: {source['link']} - Returned empty response")
        else:
            print(".", end="", flush=True)
