"""This module provides the RP To-Do config functionality."""
# rptodo/config.py

from calendar import c
import configparser
from pathlib import Path

import typer

from trieve import DIR_ERROR, FILE_ERROR, SUCCESS, __app_name__

CONFIG_DIR_PATH = Path(typer.get_app_dir(__app_name__))
CONFIG_FILE_PATH = CONFIG_DIR_PATH / "config.ini"


def init_app(db_path: str) -> int:
    """Initialize the application."""
    config_code = _init_config_file()
    if config_code != SUCCESS:
        return config_code
    return SUCCESS


def _init_config_file() -> int:
    try:
        CONFIG_DIR_PATH.mkdir(exist_ok=True)
    except OSError:
        return DIR_ERROR
    try:
        CONFIG_FILE_PATH.touch(exist_ok=True)
    except OSError:
        return FILE_ERROR

    config_parser = configparser.ConfigParser()
    config_parser["DEFAULT"] = {
        "client_url": "http://localhost:5174",
        "server_url": "http://localhost:8090",
        "api_key": "admin",
        "db_url": "postgres://postgres:password@localhost:5432/vault",
    }
    return SUCCESS


def store_value(key: str, value: str) -> None:
    """Store a value in the config file."""
    config_parser = configparser.ConfigParser()
    config_parser.read(CONFIG_FILE_PATH)
    config_parser["DEFAULT"][key] = value
    with open(CONFIG_FILE_PATH, "w") as config_file:
        config_parser.write(config_file)


def get_value(key: str) -> str | None:
    config_parser = configparser.ConfigParser()
    config_parser.read(CONFIG_FILE_PATH)
    return config_parser.get("DEFAULT", key, fallback=None)

