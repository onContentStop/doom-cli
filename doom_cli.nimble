# Package

version       = "0.1.0"
author        = "Kyle Coffey"
description   = "Doom launcher for the command line, supporting any engine"
license       = "MIT"
srcDir        = "src"
bin           = @["doom_cli"]
binDir        = "bin"


# Dependencies

requires "nim >= 1.9.1"
requires "argparse >= 4.0.1"
