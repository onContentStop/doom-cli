from platformdirs import PlatformDirs
import tomllib


DIRS = PlatformDirs("doom-cli")


EXAMPLE_CONFIG = """
# Example configuration file
# Lines starting with '#' are comments
[engines.example]
# Name(s) to identify the engine, required
names = ["example", "ex"]
# Path to the engine executable, required
run = "/path/to/engine.exe"
# Valid kinds:
# - Vanilla: Vanilla Doom (e.g. Chocolate Doom, Crispy Doom)
# - Boom: Boom-compatible ports (e.g. ReBoom)
# - MBF: MBF-compatible ports (e.g. PrBoom+, dsda-doom)
# - Eternity: Eternity Engine
# - ZDoom: ZDoom-based ports (e.g. GZDoom, Zandronum)
kind = "Vanilla"
# Any additional arguments to pass to the engine
extra_args = []
# When adding files, use -merge instead of -file
use_merge_arg = false
""".lstrip()


def init():
    DIRS.user_config_path.mkdir(parents=True, exist_ok=True)
    with open(DIRS.user_config_path / "config.toml", "w") as f:
        f.write(EXAMPLE_CONFIG)


def load():
    with open(DIRS.user_config_path / "config.toml", "rb") as f:
        return tomllib.load(f)
