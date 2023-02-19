# Package

import std/strformat

version       = "0.1.0"
author        = "Kyle Coffey"
description   = "Doom launcher for the command line, supporting most engines"
license       = "MIT"
srcDir        = "src"
bin           = @["playdoom"]
binDir        = "bin"


before build:
  writeFile(
    "src/doomCli/version.nim",
    """
      const PKG_VERSION* = "{version}"
      const PKG_AUTHOR* = "{author}"
    """.fmt.dedent
  )

# Dependencies

requires "nim >= 1.9.1"
requires "argparse >= 4.0.1"
requires "https://github.com/Phytolizer/nim-appdirs#4e22fcf13a2eaa2b7e5bd91cf537f5c4a5951378"
