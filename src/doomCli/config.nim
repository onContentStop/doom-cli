import std/options
import std/os
import std/sequtils
import std/strformat

import appdirs
import kdl

import ./version

{.experimental: "caseStmtMacros".}

type Engine* = object
  name*: string
  path*: string
  args*: string

type Config* = object
  defaultEngine*: string
  dir*: string
  engines*: seq[Engine]

const exampleCfg = staticRead("exampleConfig.kdl")

proc getKey(node: KdlNode, key: string): Option[KdlNode] =
  for val in node.children.filterIt(it.name == key):
    return some(val)

  return none[KdlNode]()

proc showErr(key: string, node: string) =
  stderr.writeLine(fmt"Error parsing config: Expected a node named ""{key}"" in ""{node}"".")
  quit 1

template getOrElse[T](opt: Option[T], otherwise: untyped): untyped =
  if opt.isSome:
    opt.get
  else:
    otherwise

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
    let doom = doc[0]
    let defaultEngine = doom.getKey("default-engine").getOrElse:
      showErr("default-engine", "doom")
      quit 1
  except IOError:
    writeFile(configPath, "doom {\n}\n")
    writeFile(examplePath, exampleCfg)
    stderr.writeLine(fmt"Please write your config in ""{configPath}"". An example is provided in ""{examplePath}"".")
    quit 1
  except KdlParserError:
    stderr.writeLine(fmt"Error parsing ""{configPath}"": {getCurrentExceptionMsg()}.")
    quit 1
