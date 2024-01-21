"""This module provides the Trieve CLI."""
# rptodo/cli.py

from http import client, server
from typing import Optional

import typer

from trieve import __app_name__, __version__
from trieve.config import _init_config_file, get_api_key, store_value
from trieve.api import create_dataset, create_organization, get_login_url, get_user
from rich import print
from rich.table import Table
from rich.console import Console
from rich.prompt import Prompt

app = typer.Typer()
add_app = typer.Typer()
app.add_typer(add_app, name="add")
console = Console()


@app.command()
def init() -> None:
    """Initialize the application."""
    _init_config_file()
    print("[bold]Hi! Welcome to the Trieve CLI!\n")
    print(
        "Before you can use this CLI, ensure that all containers are running using [code]./convenience.sh -l"
    )
    print(
        "Also, ensure that you have the API server and the search client running on your machine.\n"
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

    print("\nGreat! Now, let's set up your dataset.\n")

    user = get_user()
    use_org = 0
    if len(user["orgs"] > 1):
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user["orgs"]):
            org_table.add_row(str(i + 1), org["name"], org["id"])

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to create your dataset in (1 - {len(user['orgs'])})",
            choices=[str(x + 1) for x in range(len(user["orgs"]))],
            show_choices=False,
        )

    dataset_name = Prompt.ask(
        "What would you like to name your dataset?", default="default"
    )
    dataset_id = create_dataset(dataset_name, user["orgs"][use_org]["id"])

    print("\nGreat! Now, here is the information you need to connect to your dataset:")
    print(f"Dataset ID: {dataset_id}")
    print(f"API Key: {api_key}\n")

    # TODO: add docs on how to upload data to the dataset


@add_app.command(name="dataset")
def add_dataset() -> None:
    user = get_user()
    use_org = 0
    if len(user["orgs"] > 1):
        org_table = Table("#", "Name", "ID")

        for i, org in enumerate(user["orgs"]):
            org_table.add_row(str(i + 1), org["name"], org["id"])

        console.print(org_table)

        use_org = Prompt.ask(
            f"Which organization would you like to create your dataset in (1 - {len(user['orgs'])})",
            choices=[str(x + 1) for x in range(len(user["orgs"]))],
            show_choices=False,
        )

    dataset_name = Prompt.ask(
        "What would you like to name your dataset?", default="default"
    )
    dataset_id = create_dataset(dataset_name, user["orgs"][use_org]["id"])

    print("\nGreat! Now, here is the information you need to connect to your dataset:")
    print(f"Dataset ID: {dataset_id}")
    print(f"API Key: {get_api_key()}\n")


@add_app.command("organization")
def add_organization() -> None:
    organization_name = Prompt.ask(
        "What would you like to name your organization?", default="default"
    )
    organization_id = create_organization(organization_name)

    print("\nGreat! Now, here is the information about your newly created dataset:")
    print(f"Organization ID: {organization_id} \n")

    typer.confirm("Would you like to create a dataset?", abort=True)

    add_dataset()


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
