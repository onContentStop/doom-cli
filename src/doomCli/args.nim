import std/options

import argparse


type Args* = object
  compatibility*: Option[string]
  engine*: string
  iwad*: string
  confirm*: bool
  pwads*: seq[string]
  record*: Option[string]
  render*: Option[string]
  warp*: Option[string]

proc parseArgs*: Args =
  var p = newParser:
    help("Run Doom via command-line arguments")
    option("-c", "--compatibility", help = "Set the compatibility level")
    option("-e", "--engine", help = "Select the engine (default: first configured)")
    option("-i", "--iwad", help = "Select the IWAD (default: doom2.wad)", default = some("doom2.wad"))
    flag("-n", "--no-confirm", help = "Skip the confirmation prompt")
    option("-p", "--pwads", help = "Select any number of PWADS", multiple = true)
    option("-r", "--record", help = "Record a demo to $PLAYDOOM_DIR/demo/{name}.lmp")
    option("-R", "--render", help = "Render a demo to $PLAYDOOM_DIR/recordings/{name}.mp4")
    option("-w", "--warp", help = "Warp to a level (supply 1-1 for E1M1 if using doom 1)")

  try:
    discard p.parse()
  except ShortCircuit as e:
    if e.flag == "argparse_help":
      echo p.help
      quit 0
  except UsageError:
    stderr.writeLine p.help
    stderr.writeLine getCurrentExceptionMsg()
    quit 1