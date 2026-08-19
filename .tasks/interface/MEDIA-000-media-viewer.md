---
id: MEDIA-000
title: Media Viewer
status: In Progress
assignee: jamiepine
parent: UI-000
priority: Medium
tags: [media, viewer, photos, videos]
whitepaper: N/A
last_updated: 2026-08-19
---

## Description

Build a full-screen media viewer for photos and videos with support for navigation, zoom, and metadata display.

## Features

- Photo viewer with zoom/pan
- Video player with controls
- Swipe/arrow navigation between items
- Metadata panel (EXIF, location, tags)
- Quick actions (tag, favorite, share, delete)
- Slideshow mode
- Live Photo support (via reference sidecars)

## Implementation Notes

- Uses platform-appropriate media APIs
- Hardware-accelerated rendering
- Lazy loading for performance
- Keyboard shortcuts for all actions
- Touch gestures on mobile

## Acceptance Criteria

- [x] Open photo in full-screen viewer
- [x] Zoom and pan with mouse/touch
- [x] Navigate with arrow keys or swipe
- [x] Video playback with controls
- [x] Display metadata panel
- [ ] Quick tag/favorite actions
- [ ] Slideshow mode
- [ ] Live Photo playback
- [x] Smooth transitions between items
