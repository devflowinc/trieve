# ensure that the ADMIN_API_KEY is set in the environment to whatever valeu you want to use

import os
import requests

ADMIN_API_KEY = "admin"
API_HOST = "http://localhost:8090/api"

user = requests.get(
    f"{API_HOST}/auth/me", headers={"Authorization": ADMIN_API_KEY}
).json()
org_id = user["user_orgs"][0]["organization_id"]

dataset = requests.post(
    f"{API_HOST}/dataset",
    headers={
        "Authorization": ADMIN_API_KEY,
        "Content-Type": "application/json",
        "AF-Organization": org_id,
    },
    json={
        "dataset_name": "default",
        "organization_id": org_id,
        "server_configuration": {},
        "client_configuration": {},
    },
).json()
