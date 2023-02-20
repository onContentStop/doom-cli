import std/options
import std/os
import std/sequtils
import std/strformat
import std/tables

import appdirs
import kdl

import ./optionsExt
import ./version

{.experimental: "caseStmtMacros".}

type Engine* = object
  name*: string
  path*: string
  args*: seq[string]

type Config* = object
  defaultEngine*: string
  dir*: string
  engines*: seq[Engine]

const exampleCfg = staticRead("exampleConfig.kdl")

proc getKey(node: KdlNode, key: string): Option[KdlNode] =
  for val in node.children.filterIt(it.name == key):
    return some(val)

  return none[KdlNode]()

proc getStringKey(node: KdlNode): Option[string] =
  if node.args.len == 1 and node.args[0].isString:
    node.args[0].getString.some
  else:
    none[string]()

proc showErr(key: string, node: string) =
  stderr.writeLine(fmt"Error parsing config: Expected a node named ""{key}"" in ""{node}"".")
  quit 1

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
    result.defaultEngine =
      doom.getKey("default-engine").map(getStringKey).flatten.getOrElse:
        showErr("default-engine", "doom")
        quit 1
    result.dir =
      doom.getKey("dir")
        .map(getStringKey)
        # dir key missing
        .getOrElse(app.userData.some)
        # dir key present but non-string
        .getOrElse:
          stderr.writeLine("Error parsing config: Expected a plain string for \"doom.dir\"")
          quit 1
    let enginesNode = doom.getKey("engines").getOrElse:
      showErr("engines", "doom")
      quit 1
    for engineNode in enginesNode.children:
      var engine = Engine(name: engineNode.name)
      engine.path =
        try: engineNode.props["path"].getString
        except KeyError:
          stderr.writeLine(
            "Error parsing config: Expected a \"path\" property on the \"" &
            engineNode.name & "\" engine."
          )
          quit 1
      case engineNode.children.len
      of 0:
        discard
      of 1:
        let engineArgs = engineNode.children[0].getKey("args").getOrElse:
          showErr("args", fmt"doom.engines.{engineNode.name}")
          quit 1
        for arg in engineArgs.args:
          engine.args.add(arg.getString)
      else:
        stderr.writeLine(
          fmt"Error parsing config: doom.engines.{engineNode.name} has too many children"
        )
        quit 1
      result.engines.add(engine)
  except IOError:
    writeFile(configPath, "doom {\n}\n")
    writeFile(examplePath, exampleCfg)
    stderr.writeLine(fmt"Please write your config in ""{configPath}"". An example is provided in ""{examplePath}"".")
    quit 1
  except KdlParserError:
    stderr.writeLine(fmt"Error parsing ""{configPath}"": {getCurrentExceptionMsg()}.")
    quit 1
