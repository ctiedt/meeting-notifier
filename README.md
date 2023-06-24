Basically just a super simple proof of concept to run a command if the system detects a process whose name matches a pattern. Windows-specific.

## How to use it

```
meeting-notifier <pattern> <command>
```

You have to put the command in quotes if it has an argument.

Optionally, you can add a second command that runs when the targeted process exits.