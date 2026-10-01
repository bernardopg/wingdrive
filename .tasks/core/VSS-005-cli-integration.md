---
id: VSS-005
title: "CLI Sidecar Commands"
status: To Do
assignee: jamiepine
parent: CORE-008
priority: Medium
tags: [vss, cli, tooling]
whitepaper: "Section 4.1.5"
last_updated: 2025-11-01
related_tasks: [CORE-008, CLI-000]
dependencies: [VSS-001, VSS-002]
---

## Description

Add comprehensive CLI support for sidecar operations, making derivative data management accessible from the command line.

See `workbench/core/storage/VIRTUAL_SIDECAR_SYSTEM_V2.md` Section "CLI Integration" for complete specification.

## Implementation Files

- `apps/cli/src/domains/sidecar/mod.rs` - New sidecar command domain
- `apps/cli/src/domains/sidecar/args.rs` - Command arguments
- `apps/cli/src/domains/sidecar/list.rs` - List sidecars
- `apps/cli/src/domains/sidecar/usage.rs` - Storage usage reporting
- `apps/cli/src/domains/sidecar/cleanup.rs` - Cleanup operations

## Tasks

### Command Structure
- [ ] Create `wing sidecars` subcommand family
- [ ] Add `wing sidecars list <content_uuid>` - list all sidecars for content
- [ ] Add `wing sidecars usage` - show storage usage by kind
- [ ] Add `wing sidecars pending` - show generation queue
- [ ] Add `wing sidecars cleanup` - clean up old/orphaned sidecars
- [ ] Add `wing sidecars regenerate <content_uuid>` - force regeneration

### Glob Pattern Support
- [ ] Support wildcard content UUIDs in copy/list operations
- [ ] Implement efficient pagination for large result sets
- [ ] Add `--limit` and `--page` flags
- [ ] Smart defaults (warn before processing millions of files)

### Standard File Operations
- [ ] Ensure `wing cp sidecar://...` works
- [ ] Ensure `wing ls sidecar://...` works
- [ ] Ensure `wing rm sidecar://...` works with confirmation
- [ ] Ensure `wing cat sidecar://.../ocr.json` works
- [ ] Add `wing info sidecar://...` for detailed status

## Commands Specification

### `wing sidecars list`

```bash
# List all sidecars for a content item
wing sidecars list 550e8400-e29b-41d4-a716-446655440000

# Output:
# Sidecars for content 550e8400-e29b-41d4-a716-446655440000
# ┌──────────────┬────────────┬────────┬──────────┬──────────┐
# │ Kind         │ Variant    │ Format │ Size     │ Status   │
# ├──────────────┼────────────┼────────┼──────────┼──────────┤
# │ Thumbnail    │ grid@2x    │ webp   │ 45.2 KB  │ Ready    │
# │ Thumbnail    │ detail@1x  │ webp   │ 128 KB   │ Ready    │
# │ OCR          │ default    │ json   │ 2.3 KB   │ Ready    │
# │ Embeddings   │ all-MiniLM │ json   │ 1.2 KB   │ Ready    │
# └──────────────┴────────────┴────────┴──────────┴──────────┘
# Total: 176.7 KB across 4 sidecars

# List specific kind
wing sidecars list 550e8400... --kind thumb

# List with paths
wing sidecars list 550e8400... --show-paths
```

### `wing sidecars usage`

```bash
# Show overall sidecar storage usage
wing sidecars usage

# Output:
# Sidecar Storage Usage
# ┌──────────────┬───────┬──────────┬──────────┐
# │ Kind         │ Count │ Size     │ Avg/File │
# ├──────────────┼───────┼──────────┼──────────┤
# │ Thumbnails   │ 45234 │ 2.3 GB   │ 52 KB    │
# │ Proxies      │ 1245  │ 18.7 GB  │ 15.4 MB  │
# │ OCR          │ 8934  │ 234 MB   │ 27 KB    │
# │ Transcripts  │ 3456  │ 1.1 GB   │ 334 KB   │
# │ Embeddings   │ 12456 │ 89 MB    │ 7 KB     │
# ├──────────────┼───────┼──────────┼──────────┤
# │ Total        │ 71325 │ 22.4 GB  │ 330 KB   │
# └──────────────┴───────┴──────────┴──────────┘

# Show usage for specific kind
wing sidecars usage --kind proxy

# Show usage by variant
wing sidecars usage --by-variant
```

### `wing sidecars pending`

```bash
# Show pending sidecar generation jobs
wing sidecars pending

# Output:
# Pending Sidecar Generation
# ┌──────────────┬─────────┬─────────┬──────────┐
# │ Kind         │ Queued  │ Running │ Failed   │
# ├──────────────┼─────────┼─────────┼──────────┤
# │ Thumbnails   │ 456     │ 12      │ 3        │
# │ OCR          │ 89      │ 5       │ 0        │
# │ Transcripts  │ 23      │ 2       │ 1        │
# └──────────────┴─────────┴─────────┴──────────┘

# Show details for failed jobs
wing sidecars pending --failed
```

### `wing sidecars cleanup`

```bash
# Clean up orphaned sidecars
wing sidecars cleanup

# Output:
# Scanning for orphaned sidecars...
# Found 45 sidecars for deleted content
# Total: 234 MB
# Clean up? [y/N] y
# Deleted 45 sidecars, freed 234 MB

# Dry run mode
wing sidecars cleanup --dry-run

# Clean specific kind
wing sidecars cleanup --kind proxy --older-than 180d
```

### `wing sidecars regenerate`

```bash
# Regenerate all sidecars for a content item
wing sidecars regenerate 550e8400-e29b-41d4-a716-446655440000

# Output:
# Regenerating sidecars for 550e8400...
# ✓ Thumbnails: 3 variants queued
# ✓ OCR: queued
# ✓ Embeddings: queued
# Jobs queued: 5

# Regenerate specific kind
wing sidecars regenerate 550e8400... --kind thumb --variant grid@2x
```

### Standard Operations with Sidecars

```bash
# Copy thumbnail to local file
wing cp sidecar://550e8400.../thumbs/grid@2x.webp ~/Desktop/thumb.webp

# List all thumbnails
wing ls "sidecar://*/thumbs/*" --limit 100

# Export all OCR text
wing cp "sidecar://*/ocr/ocr.json" ~/ocr-exports/

# Delete large proxies
wing rm "sidecar://*/proxies/2160p"
# Output:
# ️  This will delete 1,247 files totaling 45.2GB
# Continue? [y/N]

# View OCR text directly
wing cat sidecar://550e8400.../ocr/ocr.json | jq .text

# Check sidecar info
wing info sidecar://550e8400.../thumbs/grid@2x.webp
# Output:
# Path: sidecar://550e8400-e29b-41d4-a716-446655440000/thumbs/grid@2x.webp
# Status: Ready
# Size: 45.2 KB
# Format: WebP
# Created: 2025-10-15 14:32:11
# Local: Yes
# Available on: MacBook Pro, Home Server
# Checksum: abc123...
```

## Acceptance Criteria

### Commands Implemented
- [ ] `wing sidecars list` shows all sidecars for content
- [ ] `wing sidecars usage` shows storage breakdown
- [ ] `wing sidecars pending` shows generation queue
- [ ] `wing sidecars cleanup` removes orphaned sidecars
- [ ] `wing sidecars regenerate` triggers regeneration

### Standard Operations
- [ ] `wing cp sidecar://...` copies sidecars
- [ ] `wing ls sidecar://...` lists sidecar directories
- [ ] `wing rm sidecar://...` deletes with confirmation
- [ ] `wing cat sidecar://...` displays content
- [ ] `wing info sidecar://...` shows detailed status

### User Experience
- [ ] Clear, formatted output with tables
- [ ] Progress indicators for long operations
- [ ] Helpful error messages
- [ ] Confirmation prompts for destructive operations
- [ ] JSON output mode for scripting (`--json` flag)

### Integration
- [ ] Glob patterns work correctly
- [ ] Pagination prevents expensive operations
- [ ] Works with piped operations
- [ ] Respects user preferences and config

## Timeline

Estimated: 4-5 days focused work

- Day 1: Command structure and args parsing
- Day 2: List, usage, pending commands
- Day 3: Cleanup and regenerate commands
- Day 4: Integration with standard operations
- Day 5: Testing, polish, documentation
