import doomCli/[
  args,
  config,
]

when isMainModule:
  discard readConfig()
  discard parseArgs()
