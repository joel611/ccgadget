(work in progress)

# Command line

The cli to setup and gather information sending to paired bluetooth device.

# Installation

```bash
brew install ccgadget
```

# Available Command

`ccgadget pair` - pair with bluetooth device
`ccgadget start` - start a deamon to run in background to fetch data from `ccusage` and send data to paired bluetooth device
`ccgadget trigger` - claude code hook command to trigger ccgadget with events

## install claude code hook

`ccgadget install-hook` - (default: project level)
`ccgadget install-hook -u` - user level

### Manually install hook

```json
{
  "hooks": {
    "EventName": [
      {
        "matcher": "ToolPattern",
        "hooks": [
          {
            "type": "command",
            "command": "ccgadget trigger"
          }
        ]
      }
    ]
  }
}
```

## Reference

- [claude-code-hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [ccusage](https://github.com/ryoppippi/ccusage)
