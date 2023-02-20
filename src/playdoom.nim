import doomCli/args
import doomCli/config as configMod

when isMainModule:
  let config = readConfig()
  echo parseArgs(config)
