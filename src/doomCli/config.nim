import std/options
import std/os
import std/sequtils
import std/strformat
import std/sugar

import appdirs
import kdl

import ./version

type Engine* = object
  name*: string
  path*: string
  args*: string

type Config* = object
  defaultEngine*: string
  dir*: string
  engines*: seq[Engine]

const exampleCfg = staticRead("exampleConfig.kdl")

proc readConfig*: Config =
  let app = application("playdoom", author = some(PKG_AUTHOR), roaming = true)
  let configDir = app.userConfig
  createDir configDir
  let configPath = configDir / "config.kdl"
  let examplePath = configDir / "example.kdl"
  try:
    let doc = parseKdlFile(configPath)
    if doc[0].name != "doom":
      stderr.writeLine(fmt"Error parsing ""{configPath}"": Expected a top-level node named ""doom"".")
      stderr.writeLine(fmt"See {examplePath} for an example.")
      quit 1
    if not doc[0].children.any(node => node.name == "default-engine"):
      writeFile(examplePath, exampleCfg)
      stderr.writeLine(fmt"Please specify a default engine in {configPath}. See {examplePath} for an example.")
      quit 1
  except IOError:
    writeFile(configPath, "doom {\n}\n")
    writeFile(examplePath, exampleCfg)
    stderr.writeLine(fmt"Please write your config in ""{configPath}"". An example is provided in ""{examplePath}"".")
    quit 1
  except KdlParserError:
    stderr.writeLine(fmt"Error parsing ""{configPath}"": {getCurrentExceptionMsg()}.")
    quit 1
