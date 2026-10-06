# WingDrive CLI Redesign Plan

## Overview

Redesign the CLI structure to be more intuitive, consistent, and user-friendly while maintaining power-user capabilities. All top-level commands (except start/stop) will support interactive wizards when called without arguments, making the CLI approachable for new users while keeping direct command paths for scripting.

## Core Design Principles

1. **Interactive by Default**: Commands without args enter interactive mode
2. **Hybrid WingPath Support**: Accept both traditional paths and WingPath URIs (`local://`, `s3://`, `content://`)
3. **Consistent Patterns**: Every resource type has predictable subcommands (list, create, remove, etc.)
4. **Smart Context Awareness**: Commands adapt based on context (e.g., browse uses index when available)
5. **Scriptable**: All interactive flows have direct command equivalents with `--format json` support

## Command Structure

### Daemon Lifecycle (No Wizards)

```bash
wing start [--foreground]           # Start daemon
wing stop [--reset]                 # Stop daemon (optional data reset)
```

### Configuration

```bash
wing config                         # Interactive: show current config, prompt to edit
wing config get <key>              # Get specific config value
wing config set <key> <value>      # Set config value
```

### Library Management

```bash
wing library                        # Show current library status (name, locations, stats, devices)
wing library create                 # Interactive: name, path, settings
wing library switch                 # Interactive: select from list
wing library list                   # List all libraries
wing library delete                 # Interactive: select + confirm
```

### Location Management (Managed Directories)

```bash
wing location                       # Interactive: list → add/remove/rescan
wing location add                   # Interactive wizard (already implemented)
wing location remove                # Interactive: select from list
wing location rescan [id]          # Interactive: select location if no ID
wing location list                  # List all locations
```

### Universal Browsing (Location-Aware)

```bash
wing browse [path|uri]              # Smart browsing with interactive TUI
                                  # - Uses location index if path is managed
                                  # - Falls back to ephemeral index if outside locations
                                  # - No path = interactive root picker
```

**Behavior:**

- Inside managed location: instant (uses existing index)
- Outside locations: ephemeral index (temporary, not persisted)
- Supports WingPath URIs for remote browsing

### File Operations (Hybrid WingPath)

```bash
wing ls [path|uri]                  # List files (simple output)
wing cp <src> <dst>                 # Copy (supports URIs + --device/--cloud flags)
wing mv <src> <dst>                 # Move
wing rm <path|uri>                  # Delete (with confirmation)
wing info <path|uri>                # Show file metadata
```

**WingPath Examples:**

```bash
# Traditional paths
wing cp /Users/me/file.txt /backup/

# WingPath URIs
wing cp local://macbook/Users/me/file.txt s3://my-bucket/backup/
wing info content://550e8400-e29b-41d4-a716-446655440000
```

### Global Search

```bash
wing search                         # Interactive: query builder with filters
wing search <query>                 # Direct search
wing search --tag <tag>            # Filter by tag
wing search --type <type>          # Filter by file type
wing search --content <text>       # Content search
wing search --size <range>         # Size filter
wing search --date <range>         # Date filter
```

### Organization - Tags

```bash
wing tag                            # Interactive: select file → add tags
wing tag create                     # Interactive: name, color, namespace
wing tag apply <target> <tags>     # Direct apply tags
wing tag remove <target> <tags>    # Remove tags
wing tag list                       # List all tags
wing tag search <query>            # Search tag names (different from wing search)
```

### Organization - Collections

```bash
wing collection                     # Interactive: list → create/add/remove
wing collection create              # Interactive: name, description
wing collection add <id>           # Interactive: select files to add
wing collection remove <id>        # Interactive: select files to remove
wing collection list                # List all collections
```

### Network - Pairing

```bash
wing pair                           # Interactive: initiate or join
wing pair initiate                  # Generate pairing code
wing pair join [code]              # Interactive: enter code if not provided
```

### Network - Devices

Note: These are paired devices, not devices registered in a library, for clarity we should show which libraries these devices are participating in by quering the devices table for all libraries!

```bash
wing devices                        # Interactive: list → revoke/manage
wing devices list                   # List paired devices
wing devices remove <id>           # Remove/revoke device
```

### Network - File Sharing

This doesn't exist yet so we can implement as a stub

```bash
wing share                          # Interactive: select device → select file
wing share <device> <file>         # Direct share via Wingdrop
```

### Cloud Storage

```bash
wing cloud                          # Interactive wizard (already implemented)
wing cloud add                      # Interactive: service type → credentials
wing cloud remove                   # Interactive: select volume
wing cloud list                     # List cloud volumes
```

### Volumes

```bash
wing volume                         # Interactive: list → manage
wing volume list                    # List all volumes (local + cloud)
```

### Sync Conduits (WIP Feature)

```bash
wing sync                           # Interactive: conduit management
wing sync status                    # Show sync state
wing sync create                    # Interactive: create sync conduit
```

### Jobs & Monitoring

```bash
wing job                            # Interactive: list → monitor/pause/cancel
wing job list                       # List all jobs
wing job monitor [id]              # Monitor jobs with TUI (all or specific)
wing job pause <id>                # Pause job
wing job resume <id>               # Resume job
wing job cancel <id>               # Cancel job
```

### Logs

```bash
wing logs                           # Interactive: show or follow
wing logs show [--tail N]          # Show recent logs
wing logs follow                    # Follow logs in real-time
```

## Removed/Merged Commands

### Removed

- `wing index` → Functionality absorbed into `wing location` and `wing browse`
- `wing status` → Replaced by `wing library` (shows current state)
- `wing network` → Split into `wing pair`, `wing devices`, `wing share`
- `wing restart` → Can be achieved with `wing stop && wing start`
- `wing update` → Can be system-level or `wing daemon update` if needed

### Merged/Reorganized

- `wing location browse` → `wing browse` (root level, location-aware)
- `wing index quick-scan` → `wing browse` (ephemeral mode automatic)
- `wing index start` → `wing location add` (with mode flags)
- `wing index verify` → `wing location rescan --verify`
- `wing network pair` → `wing pair`
- `wing network devices` → `wing devices`
- `wing network spacedrop` → `wing share`

## Implementation Plan

### Phase 1: Command Restructure

**Goal**: Reorganize command structure and file layout

**Tasks:**

1. Create new domain modules:
   - `apps/cli/src/domains/browse/` (new)
   - `apps/cli/src/domains/pair/` (split from network)
   - `apps/cli/src/domains/share/` (split from network)
   - `apps/cli/src/domains/collection/` (new)

2. Remove obsolete modules:
   - `apps/cli/src/domains/index/` (merge into location + browse)

3. Update `apps/cli/src/main.rs`:
   - Restructure `Commands` enum to match new hierarchy
   - Remove merged commands
   - Update command routing

4. Update existing domain modules to match new patterns

### Phase 2: Smart Browse Implementation

**Goal**: Create location-aware browsing command

**Tasks:**

1. Implement browse command with dual-mode indexing:
   - Check if path is within managed location
   - Use location index if available (fast)
   - Fall back to ephemeral index if outside (slower)

2. Add interactive TUI for navigation:
   - Tree view or grid view
   - Keyboard navigation
   - Preview panel
   - Reference existing location wizard UX

3. Support WingPath URIs for remote browsing:
   - `wing browse local://device/path`
   - `wing browse s3://bucket/prefix`

### Phase 3: Enhanced Search

**Goal**: Restore and improve global search

**Tasks:**

1. Restore `domains/search/` with enhanced functionality
2. Implement filter flags:
   - `--tag`, `--type`, `--content`, `--size`, `--date`
3. Create interactive query builder
4. Multiple output formats: table, json, paths-only

### Phase 4: Interactive Wizards

**Goal**: Add wizards to all commands that should have them

**Commands requiring wizards:**

- `wing config` - show/edit flow
- `wing browse` - TUI navigator
- `wing search` - query builder
- `wing tag` - tagging workflow
- `wing collection` - collection management
- `wing pair` - pairing flow
- `wing devices` - device management
- `wing share` - file sharing picker
- `wing volume` - volume management
- `wing job` - job list → actions
- `wing logs` - show/follow picker

**Implementation approach:**

- Use `dialoguer` crate for prompts
- Show contextual info before prompts
- Maintain direct command paths for scripting
- Pattern: detect when called with no subcommand/args

### Phase 5: Hybrid WingPath Support

**Goal**: Support both traditional paths and WingPath URIs

**Tasks:**

1. Create URI parser that accepts both formats
2. Update file operations (ls, cp, mv, rm, info, browse):
   - Parse traditional paths: `/Users/me/file.txt`
   - Parse WingPath URIs: `local://device/path`, `s3://bucket/key`, `content://uuid`
   - Support shortcut flags: `--device`, `--cloud`

3. Add resolution logic:
   - Convert traditional paths to WingPath internally
   - Resolve URIs to actual storage locations
   - Handle cross-device operations

4. Error handling:
   - Clear messages for malformed URIs
   - Suggestions for common mistakes

### Phase 6: Documentation & Polish

**Goal**: Comprehensive documentation and UX refinement

**Tasks:**

1. Update help text for all commands
2. Add examples in `--help` output
3. Update documentation files in `docs/cli/`
4. Create migration guide from old commands to new
5. Add shell completions (bash, zsh, fish)
6. Test all interactive flows
7. Ensure `--format json` works consistently

## Success Criteria

- All commands follow consistent patterns
- Interactive mode works for all designated commands
- Direct command paths work for scripting
- WingPath URIs work across file operations
- Browse intelligently uses location index when available
- Search provides powerful filtering
- Documentation is comprehensive
- No regression in existing functionality
- Shell completions work

## Migration Notes

**For users upgrading from current CLI:**

Breaking changes:

- `wing index` removed → use `wing location add` or `wing browse`
- `wing network pair` → `wing pair`
- `wing network spacedrop` → `wing share`
- `wing status` → `wing library`

Non-breaking:

- All location commands remain the same
- Library commands remain the same
- Job commands remain the same
- Logs commands remain the same

## Design Rationale

### Why `browse` is separate from `ls`

- `browse` is an interactive navigator with TUI
- `ls` is a simple list command (like traditional Unix ls)
- Different use cases: exploration vs scripting

### Why `search` is global while `tag search` exists

- `wing search` searches file content, names, metadata across entire library
- `wing tag search` searches for tag names themselves
- Different domains: files vs tags

### Why split `network` into `pair`, `devices`, `share`

- Each has distinct mental models and workflows
- Pairing is an infrequent setup task
- Devices is for ongoing management
- Share is a frequent operation that should be quick

### Why remove `wing status`

- `wing library` provides library-level status (the most common query)
- System-level status can be `wing daemon status` if needed
- Reduces command clutter

### Why keep `wing config` separate

- Global configuration spans libraries
- Different scope than library-specific settings
- Common pattern in other CLIs (git config, npm config)
