import click
from . import config

@click.command()
def main():
    config.init()

main()
