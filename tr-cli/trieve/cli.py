"""This module provides the Trieve CLI."""
# rptodo/cli.py

from distutils.command import upload
from http import client, server
from re import sub
import subprocess
from typing import Optional

from numpy import delete

import typer

from trieve import __app_name__, __version__
from trieve.config import _init_config_file, get_value, store_value
from trieve.api import (
    create_dataset,
    create_organization,
    delete_dataset_api,
    download_sample_data,
    get_datasets_for_org,
    get_login_url,
    get_user,
    upload_sample_data,
)
from trieve.db import terminate_connections
from rich import print
from rich.table import Table
from rich.console import Console
from rich.prompt import Prompt
import subprocess

app = typer.Typer(no_args_is_help=True, subcommand_metavar="COMMAND")
add_app = typer.Typer()
reset_app = typer.Typer()
delete_app = typer.Typer()
app.add_typer(add_app, name="add", help="Add a dataset or organization.")
app.add_typer(reset_app, name="reset", help="Reset a service.")
app.add_typer(delete_app, name="delete", help="Delete a dataset or organization.")
console = Console()


@app.command()
def init() -> None:
    """Initialize the Trieve CLI with your account information. (Run this first!)"""
    _init_config_file()
    print("[bold]Hi! Welcome to the Trieve CLI!\n")
    startup = typer.confirm("Would you like to start up the local docker containers?")
    if startup:
        print("Starting local services...")
        subprocess.run(["docker", "compose", "up", "-d", "db"])
        subprocess.run(["docker", "compose", "up", "-d", "redis"])
        subprocess.run(["docker", "compose", "up", "-d", "qdrant-database"])
        subprocess.run(["docker", "compose", "up", "-d", "s3"])
        subprocess.run(["docker", "compose", "up", "-d", "s3-client"])
        subprocess.run(["docker", "compose", "up", "-d", "keycloak"])
        subprocess.run(["docker", "compose", "up", "-d", "keycloak-db"])
    print(
        "Ensure that you have the API server and the search client running on your machine.\n"
    )
    print("Let's get started!\n")

    print("\nGreat! Now, let's set up your account.\n")

    which_key = Prompt.ask(
        "Would you like to set up your own account, or use the default admin account? (setup/admin)",
        choices=["setup", "admin"],
    )
    if which_key == "setup":
        server_url = Prompt.ask(
            "What is the URL of the server you are running?",
            default="http://localhost:8090",
        )
        client_url = Prompt.ask(
            "What is the URL of the client you are running?",
            default="http://localhost:5174",
        )
        store_value("server_url", server_url)
        store_value("client_url", client_url)

        print("\nPlease follow this link to set up your account:")
        print(f"[bold][link={get_login_url()}]Login url[/link]\n")
        print(
            "Once you have successfully authenticated, click on the profile icon in the top right corner of the page."
        )
        print("Then, click on [italics]'Settings'[/italics].")
        print("Finally, click on [italics]'Generate'[/italics] to create your api key.")
        print("Copy the key and paste it here:\n")
        api_key = Prompt.ask("API Key")
        store_value("api_key", api_key)
    else:
        print(
            "\n[bold][red][alert]Any datasets/chunks that are created using the default account can only be accessed via the api and not through the client"
        )
        print(
            "Please ensure that you have set the ADMIN_API_KEY environment variable.\n"
        )
        api_key = Prompt.ask(
            "What is the API Key you have set?",
            default="admin",
        )
        store_value("api_key", api_key)

    typer.confirm("Would you like to create a dataset?", abort=True)

    print("\nGreat! Now, let's set up your dataset.\n")

    user = get_user()
    use_org = 1
    if len(user.orgs) > 1:
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user.orgs):
            org_table.add_row(str(i + 1), org.name, org.id)

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to create your dataset in? (1 - {len(user.orgs)})",
            choices=[str(x + 1) for x in range(len(user.orgs))],
            show_choices=False,
        )

    dataset_name = Prompt.ask(
        "What would you like to name your dataset?", default="default"
    )
    dataset_id = create_dataset(dataset_name, user.orgs[int(use_org) - 1].id)

    print("\nGreat! Now, here is the information you need to connect to your dataset:")
    print(f"Dataset ID: {dataset_id}")
    print(f"API Key: {api_key}\n")

    # TODO: add docs on how to upload data to the dataset


@add_app.command(name="dataset")
def add_dataset() -> None:
    """Add a dataset to your account."""
    if not get_value("api_key"):
        print("API key not set! Please run `trieve init` to set up.")
        typer.Abort()
    user = get_user()
    use_org = 1
    if len(user.orgs) > 1:
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user.orgs):
            org_table.add_row(str(i + 1), org.name, org.id)

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to create your dataset in? (1 - {len(user.orgs)})",
            choices=[str(x + 1) for x in range(len(user.orgs))],
            show_choices=False,
        )

    dataset_name = Prompt.ask(
        "What would you like to name your dataset?", default="default"
    )
    dataset = create_dataset(dataset_name, user.orgs[int(use_org) - 1].id)

    print("\nGreat! Now, here is the information you need to connect to your dataset:")
    print(f"Dataset ID: {dataset.id}")
    print(f"Dataset Name: {dataset.name}")
    print(f"API Key: {get_value('api_key')}\n")


@add_app.command("organization")
def add_organization() -> None:
    """Add an organization to your account."""
    if not get_value("api_key"):
        print("API key not set! Please run `trieve init` to set up.")
        typer.Abort()
    organization_name = Prompt.ask(
        "What would you like to name your organization?", default="default"
    )
    organization = create_organization(organization_name)

    print(
        "\nGreat! Now, here is the information about your newly created organization:"
    )
    print(f"Organization ID: {organization.id} \n")
    print(f"Organization Name: {organization.name} \n")

    typer.confirm("Would you like to create a dataset?", abort=True)

    add_dataset()


@add_app.command("sample")
def add_sample_data() -> None:
    """Add a sample dataset to your account."""
    if not get_value("api_key"):
        print("API key not set! Please run `trieve init` to set up.")
        typer.Abort()
    typer.confirm(
        "Are you sure you would like to create a sample dataset? Will require download of sample data (around 1.1 gb)",
        abort=True,
    )
    file_path = Prompt.ask(
        "Where would you like to download the sample data to?",
        default="./sample-data/enron.csv",
    )
    download_sample_data(file_path)
    print("Successfully downloaded sample dataset!")
    user = get_user()
    use_org = 1
    if len(user.orgs) > 1:
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user.orgs):
            org_table.add_row(str(i + 1), org.name, org.id)

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to create your dataset in? (1 - {len(user.orgs)})",
            choices=[str(x + 1) for x in range(len(user.orgs))],
            show_choices=False,
        )

    dataset_name = Prompt.ask(
        "What would you like to name your dataset?", default="enron"
    )
    dataset = create_dataset(dataset_name, user.orgs[int(use_org) - 1].id)
    upload_sample_data(dataset.id, file_path)
    print("Successfully uploaded sample dataset!")
    print("\nGreat! Now, here is the information you need to connect to your dataset:")
    print(f"Dataset ID: {dataset.id}")
    print(f"Dataset Name: {dataset.name}")
    print(f"API Key: {get_value('api_key')}\n")


@reset_app.command("db")
def reset_db() -> None:
    """Reset your Qdrant and Postgres databases."""
    typer.confirm(
        "Are you sure you want to reset your Qdrant and Postgres databases?", abort=True
    )
    if not get_value("db_url"):
        db_url = Prompt.ask(
            "\nWhat is the url of your Postgres db?",
            default="postgres://postgres:password@localhost:5432/vault",
            show_default=False,
        )

        store_value("db_url", db_url)

    subprocess.run(["docker", "compose", "stop", "qdrant-database"])
    subprocess.run(["docker", "compose", "rm", "-f", "qdrant-database"])
    subprocess.run(["docker", "volume", "rm", "arguflow_qdrant_data"])
    subprocess.run(["docker", "compose", "up", "-d", "qdrant-database"])
    terminate_connections(get_value("db_url"))  # type: ignore
    subprocess.run(
        [
            "diesel",
            "db",
            "reset",
            "--migration-dir",
            "../server/migrations",
            "--config-file",
            "../server/diesel.toml",
            "--database-url",
            get_value("db_url"),  # type: ignore
        ]
    )

    print("\nSuccessfully reset your databases!")


@reset_app.command("s3")
def reset_s3() -> None:
    """Reset your S3 instance."""
    typer.confirm("Are you sure you want to reset your S3 instance?", abort=True)
    subprocess.run(["docker", "compose", "stop", "s3"])
    subprocess.run(["docker", "compose", "rm", "-f", "s3"])
    subprocess.run(["docker", "volume", "rm", "vault_s3-data"])
    subprocess.run(["docker", "compose", "up", "-d", "s3"])


@reset_app.command("redis")
def reset_redis() -> None:
    """Reset your Redis instance."""
    typer.confirm("Are you sure you want to reset your Redis db?", abort=True)
    subprocess.run(["docker", "compose", "stop", "script-redis"])
    subprocess.run(["docker", "compose", "rm", "-f", "script-redis"])
    subprocess.run(["docker", "volume", "rm", "vault_script-redis-data"])
    subprocess.run(["docker", "compose", "up", "-d", "script-redis"])


@delete_app.command("dataset")
def delete_dataset() -> None:
    """Delete a dataset from your account."""
    if not get_value("api_key"):
        print("API key not set! Please run `trieve init` to set up.")
        typer.Abort()
    user = get_user()
    use_org = 1
    if len(user.orgs) > 1:
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user.orgs):
            org_table.add_row(str(i + 1), org.name, org.id)

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to delete your dataset in? (1 - {len(user.orgs)})",
            choices=[str(x + 1) for x in range(len(user.orgs))],
            show_choices=False,
        )

    datasets = get_datasets_for_org(user.orgs[int(use_org) - 1].id)
    dataset_table = Table("#", "Name", "ID")
    for i, dataset in enumerate(datasets):
        dataset_table.add_row(str(i + 1), dataset.dataset.name, dataset.dataset.id)
    console.print(dataset_table)

    use_dataset = Prompt.ask(
        f"Which dataset would you like to delete? (1 - {len(user.orgs)})",
        choices=[str(x + 1) for x in range(len(user.orgs))],
        show_choices=False,
    )
    delete_dataset_api(datasets[int(use_dataset) - 1].dataset.id)

    print("\nSuccessfully deleted dataset!")


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(f"{__app_name__} v{__version__}")
        raise typer.Exit()


@app.callback()
def main(
    version: Optional[bool] = typer.Option(
        None,
        "--version",
        "-v",
        help="Show the application's version and exit.",
        callback=_version_callback,
        is_eager=True,
    )
) -> None:
    return
