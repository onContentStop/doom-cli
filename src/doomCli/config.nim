import std/options
import std/os
import std/parsecfg
import std/strformat

import appdirs

import ./version

type Engine* = object
  name*: string
  path*: string
  args*: string

type Config* = object
  defaultEngine*: string
  dir*: string
  engines*: seq[Engine]

const exampleCfg = staticRead("exampleConfig.ini")

proc readConfig*: Config =
  let app = application("playdoom", author = some(PKG_AUTHOR), roaming = true)
  let configDir = app.userConfig
  createDir configDir
  let configPath = configDir / "config.ini"
  let examplePath = configDir / "example.ini"
  try:
    let config = loadConfig(configPath)
    let defaultEngine = config.getSectionValue("doom", "default-engine")
    if defaultEngine == "":
      writeFile(examplePath, exampleCfg)
      stderr.writeLine(fmt"Please specify a default engine in {configPath}. See {examplePath} for an example.")
      quit 1
  except IOError:
    writeFile(configPath, "[doom]\n")
    writeFile(examplePath, exampleCfg)
    stderr.writeLine(fmt"Please write your config in {configPath}. An example is provided in {examplePath}.")
    quit 1
