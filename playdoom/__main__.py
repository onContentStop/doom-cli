import click
from . import config

@click.command()
def main():
    try:
        print(config.load())
    except IOError:
        config.init()

main()
