"""Trieve CLI entry point script."""
# trieve/__main__.py

from trieve import cli, __app_name__


def main():
    cli.app(prog_name=__app_name__)


if __name__ == "__main__":
    main()
