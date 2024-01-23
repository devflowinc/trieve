import configparser
from email import header
import os
import pandas as pd
import requests
from trieve.config import CONFIG_FILE_PATH
from rich.progress import track
import concurrent.futures
import trieve_python_client
from trieve_python_client.models.slim_user import SlimUser
from trieve_python_client.models.dataset import Dataset
from trieve_python_client.models.organization import Organization
from trieve_python_client.api.chunk_api import ChunkApi
from trieve_python_client.models import *
from trieve_python_client.configuration import Configuration
from trieve_python_client.api_client import ApiClient
from trieve_python_client.api.auth_api import AuthApi
from trieve_python_client.api.dataset_api import DatasetApi
from trieve_python_client.api.organization_api import OrganizationApi
from trieve_python_client.api.chunk_api import ChunkApi
from typing import Dict, List


def get_login_url() -> str:
    """Get the login URL for the Trieve application."""
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    return f"{config.get('DEFAULT', 'server_url')}/api/auth?redirect_uri={config.get('DEFAULT', 'client_url')}"


def get_user() -> SlimUser:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    client = Configuration(host=config.get("DEFAULT", "server_url"))
    default_headers = {
        "Authorization": api_key,
    }
    user = AuthApi(ApiClient(client)).get_me(_headers=default_headers)
    return user


def create_dataset(name: str, org_id: str) -> Dataset:
    """Create a dataset."""
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    client = Configuration(host=config.get("DEFAULT", "server_url"))
    headers = {
        "Authorization": api_key,
        "TR-Organization": org_id,
    }
    dataset_api = DatasetApi(ApiClient(client)).create_dataset(
        create_dataset_request=CreateDatasetRequest(
            dataset_name=name,
            organization_id=org_id,
            client_configuration={},
            server_configuration={},
        ),
        _headers=headers,
    )

    # TODO: account for if user is exceeding plan limits
    return dataset_api


def create_organization(name: str) -> Organization:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    client = Configuration(host=config.get("DEFAULT", "server_url"))
    headers = {
        "Authorization": api_key,
    }
    organization_api = OrganizationApi(ApiClient(client)).create_organization(
        create_organization_data=CreateOrganizationData(name=name),
        _headers=headers,
    )

    return organization_api


def download_sample_data(file_path: str) -> None:
    if os.path.exists(file_path):
        return
    response = requests.get(
        "https://tr-enron-dataset.s3.us-east-2.amazonaws.com/enron_05_17_2015_filtered.csv",
        stream=True,
    )
    # Sizes in bytes.
    total_size = int(response.headers.get("content-length", 0))
    block_size = 1024
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with open(file_path, "wb") as file:
        for bytes in track(
            sequence=response.iter_content(block_size),
            total=total_size / block_size,
            description="Downloading sample data...",
        ):
            file.write(bytes)


def upload_sample_data(dataset_id: str, file_path: str):
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    data = pd.read_csv(file_path, delimiter="#")
    data.fillna("", inplace=True)
    config = Configuration(host=config.get("DEFAULT", "server_url"))
    client = ChunkApi(ApiClient(config))
    headers = {
        "Authorization": api_key,
        "TR-Dataset": dataset_id,
    }
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = []
        for i in range(len(data)):
            futures.append(
                executor.submit(
                    upload_sample_data_parallel_helper,
                    i,
                    data,
                    dataset_id,
                    client,
                    headers,
                )
            )
        for future in track(
            concurrent.futures.as_completed(futures),
            total=len(futures),
            description="Uploading data...",
        ):
            future.result()


def upload_sample_data_parallel_helper(
    i: int,
    data: pd.DataFrame,
    dataset_id: str,
    client: ChunkApi,
    headers: Dict[str, str],
):
    req = client.create_chunk(
        create_chunk_data=CreateChunkData(
            chunk_html=data.iloc[i]["content"],
            link=data.iloc[i]["file"],
            time_stamp=data.iloc[i]["Date"],
            metadata={
                "Message-ID": data.iloc[i]["Message-ID"],
                "From": data.iloc[i]["From"][12:-2],
                "To": data.iloc[i]["To"][12:-2],
                "Subject": data.iloc[i]["Subject"],
                "X-From": data.iloc[i]["X-From"],
                "X-To": data.iloc[i]["X-To"],
                "X-CC": data.iloc[i]["X-cc"],
                "X-BCC": data.iloc[i]["X-bcc"],
                "X-Folder": data.iloc[i]["X-Folder"],
                "X-Origin": data.iloc[i]["X-Origin"],
                "X-FileName": data.iloc[i]["X-FileName"],
                "User": data.iloc[i]["user"],
            },
        ),
        _headers=headers,
    )


def get_datasets_for_org(org_id: str) -> List[DatasetAndUsage]:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    client = Configuration(
        host=config.get("DEFAULT", "server_url"), api_key={"api_key": api_key}
    )
    headers = {
        "Authorization": api_key,
        "TR-Organization": org_id,
    }
    datasets = DatasetApi(ApiClient(client)).get_datasets_from_organization(
        organization_id=org_id, _headers=headers
    )
    return datasets


def delete_dataset_api(dataset_id: str) -> None:
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE_PATH)
    api_key = config.get("DEFAULT", "api_key")
    client = Configuration(
        host=config.get("DEFAULT", "server_url"), api_key={"api_key": api_key}
    )
    headers = {
        "Authorization": api_key,
        "TR-Dataset": dataset_id,
    }
    DatasetApi(ApiClient(client)).delete_dataset(
        delete_dataset_request=DeleteDatasetRequest(
            dataset_id=dataset_id,
        ),
        _headers=headers,
    )
