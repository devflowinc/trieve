import configparser
import requests
from trieve.config import CONFIG_FILE_PATH


def get_login_url() -> str:
    """Get the login URL for the Trieve application."""
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    return f"{config.get('DEFAULT', 'server_url')}/api/auth?redirect_uri={config.get('DEFAULT', 'client_url')}"


def get_user() -> dict:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    user = requests.get(
        f"{config.get('DEFAULT', 'server_url')}/api/auth/me",
        headers={"Authorization": api_key},
    ).json()
    return user


def create_dataset(name: str, org_id: str) -> str:
    """Create a dataset."""
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    dataset = requests.post(
        f"{config.get('DEFAULT', 'server_url')}/api/dataset",
        headers={
            "Authorization": api_key,
            "Content-Type": "application/json",
            "TR-Organization": org_id,
        },
        json={
            "dataset_name": name,
            "organization_id": org_id,
            "server_configuration": {},
            "client_configuration": {},
        },
    ).json()

    dataset_id = dataset["id"]
    return dataset_id


def create_organization(name: str) -> str:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    organization = requests.post(
        f"{config.get('DEFAULT', 'server_url')}/api/organization",
        headers={
            "Authorization": api_key,
            "Content-Type": "application/json",
        },
        json={"name": name, "configuration": {}},
    ).json()

    organization_id = organization["id"]
    return organization_id
