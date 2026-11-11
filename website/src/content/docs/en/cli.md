---
title: The wing CLI
description: Drive the WingDrive daemon from your terminal and scripts.
order: 9
group: Advanced
---

`wing` is the command-line client for the WingDrive daemon. It talks to the same daemon as the desktop app, so a script and a window always see the same library.

## Daemon

```bash
wing start               # start the daemon in the background
wing start --foreground  # run in the foreground with logs
wing status              # daemon, library and network status
wing stop
wing restart
wing logs                # view and follow logs
```

## Libraries and locations

```bash
wing library create "Personal"
wing library list
wing library switch <library>

wing location add ~/Pictures --name "Pictures"
wing location list
wing location rescan <location>
wing location remove <location>
```

## Indexing and search

```bash
wing index start ~/Projects
wing index verify <location>     # compare the index with the disk
wing search files "invoice 2026"
```

## Files

```bash
wing file list ~/Downloads
wing file info ./report.pdf
wing file copy ./a.txt ~/Backup/
wing file rename ./old.txt new.txt
wing file mkdir ~/Projects/new
wing file delete ./tmp.log
wing file undo                   # reverse the last copy, move or trash
wing file history                # what can still be undone
```

## Jobs

```bash
wing job list
wing job monitor                 # live progress
wing job pause <job>
wing job resume <job>
wing job cancel <job>
```

## Network, devices and volumes

```bash
wing network status
wing network pair generate
wing network pair join "<12 words>"
wing network devices
wing network wingdrop send <device-id> ./file
wing volume list
wing cloud                       # interactive cloud setup
```

## Multiple instances

Run isolated daemons side by side, for testing or to keep libraries apart:

```bash
wing --instance test start
wing --instance test library create "Test"
wing --instance test stop
```

## Updates

```bash
wing update    # check for a newer release and print how to install it
```

Every command has `--help`.
