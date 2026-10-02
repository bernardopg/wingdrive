---
id: EXPL-005
title: Explorer Tabs
status: In Progress
assignee: unassigned
parent: EXPL-000
priority: High
sprint: S01
milestone: M1
tags: [explorer, tabs, navigation, ui]
last_updated: 2026-10-02
related_tasks: []
---

## Description

Add browser-like tabs to WingDrive Explorer, enabling users to browse multiple locations simultaneously. This requires careful integration with the keybind system, proper UI rendering, and preservation of context (selection, view mode, scroll position, navigation history) across tabs.

Shipped: tab bar, create/close/switch, batch close (others / to the right), reopen closed tab with its explorer state, drag reorder, keybinds, and localStorage persistence scoped per window.

Each tab now owns an independent memory router. Only the active view is mounted; the active and two recent routers remain cached. The browser URL mirrors the active router. Selection, sorting and scroll persist per tab, and history survives switching and reloads.

## Dependencies

None - this is a standalone architectural feature.

## Implementation Notes

### Current Architecture

**Explorer Context** (`/packages/interface/src/components/Explorer/context.tsx`):
- Dual-reducer pattern: `navigationReducer` (history, index) + `uiReducer` (view mode, sorting, UI state)
- URL-based navigation as single source of truth
- Navigation synced via React Router (`useNavigate`, `useLocation`)
- View preferences persisted per space item

**Keybind System** (`/packages/interface/src/util/keybinds/`):
- Type-safe, scope-based keybind registry
- Platform-aware (Cmd/Ctrl auto-conversion)
- Has unused tab keybinds defined that need cleanup

**Existing Tab Pattern** (`/packages/interface/src/components/Inspector/Tabs.tsx`):
- Icon-based tabs with framer-motion animations
- Active indicator using `layoutId` for smooth transitions

### Architectural Decision: Tab Isolation via React Keys

Each tab will have **isolated contexts** created through React's `key` prop mechanism. This approach:
- Forces separate context instances per tab via React's `key` prop
- Requires no changes to ExplorerProvider/SelectionProvider internals
- Provides true state isolation (navigation, selection, view mode)
- Maintains "URL as single source of truth" principle for active tab

### Router Strategy

Use a memory router for every tab and mirror only the active route to the browser URL. Dispose routers outside the three-entry cache. Inactive views unmount, releasing their query subscriptions; query data follows the existing TanStack cache lifetime.

### Per-Tab State (Isolated)

Each tab maintains:
- Navigation history with back/forward stack
- Current target (path or view)
- Selected files (independent selection)
- View mode & sort settings
- Scroll position
- UI state (Quick Preview, tag mode)

### Shared Global State

Synchronized across all tabs:
- Sidebar/Inspector visibility
- Current library ID
- Theme preferences

## Acceptance Criteria

### Phase 1: Core Infrastructure (MVP)
- [x] `TabManagerContext.tsx` created with core state management
- [x] `TabBar.tsx` UI component implemented
- [x] `TabView.tsx` renders the active isolated router
- [x] `useTabManager.ts` hook created
- [x] `Explorer.tsx` wrapped in TabManagerProvider
- [x] Only the mounted active Explorer receives keyboard and platform handlers
- [x] Selection restores from per-tab state when the active view mounts
- [x] App launches with single tab, no regressions

### Phase 2: Multi-Tab State
- [x] Create/close/switch tabs functional
- [x] Saved path per tab restored on switch
- [x] Independent navigation history per tab
- [x] Active memory router mirrors its route to the browser URL
- [ ] 5+ tabs with isolated state, <50ms tab switching

### Phase 3: Keybinds
- [x] `explorer.openInNewTab` keybind removed (conflicts with global)
- [x] `tabs.newTab` (Cmd+T) - creates new tab
- [x] `tabs.closeTab` (Cmd+W) - closes active tab
- [x] `tabs.nextTab` (Cmd+Shift+]) - switches to next tab
- [x] `tabs.previousTab` (Cmd+Shift+[) - switches to previous tab
- [x] `tabs.selectTab1-9` (Cmd+1-9) - jumps to specific tab

### Phase 4: Performance
- [x] Only active view mounts; active plus two recent routers are cached
- [x] Inactive views release query subscriptions and use existing query GC
- [x] Scroll position preservation per tab
- [ ] 15 tabs <500MB memory
- [x] No memory leaks over 100 tab cycles

### Phase 5: Persistence
- [x] Tabs serialized on change (localStorage, key scoped per window label)
- [x] Tabs restored on launch
- [x] Stale tabs handled gracefully (deleted locations)
- [x] Reuse inline persistence with validation in TabManagerContext

### Phase 6: Polish (Post-MVP)
- [x] Tab context menu
- [x] Drag-to-reorder tabs
- [x] Cross-tab file drag-drop
- [x] Tab close animations
- [x] "Reopen Closed Tab" (Cmd+Shift+T)

## Implementation Files

- `packages/interface/src/components/TabManager/{TabManagerContext,TabBar,TabView,tabRouter}.tsx` (router helper uses `.ts`)
- `packages/interface/src/routes/explorer/hooks/useTabScroll.ts`
- Existing Explorer contexts and view components persist state through TabManagerContext.

## User Experience

**Before:**
- Single Explorer view only
- Navigation history lost when switching locations via sidebar
- No way to compare two folders side-by-side workflow

**After:**
- Multiple tabs like browser (Cmd+T to create)
- Each tab maintains independent history (back/forward)
- URL always reflects active tab
- Keyboard shortcuts match browser conventions
- Scroll position preserved per tab
- Tabs restored on app restart

## Testing

### Unit Tests
- Tab lifecycle (create/close/switch)
- State serialization/deserialization
- Router type swapping
- Scroll state save/restore

### Integration Tests
- Multi-tab state isolation
- Navigation history per tab
- Selection independence
- Platform API sync (active tab only)

### Performance Tests
- Memory usage with 15 tabs
- Tab switch latency (<50ms)
- Memory leak detection (100 cycles)

### Manual Testing
- All keybinds functional
- Edge cases (deleted locations, same path in multiple tabs)
- Drag-drop between tabs
- Session restore
- Last tab cannot be closed

## Edge Cases

- **Same path in multiple tabs:** Independent state maintained
- **Backend location deleted:** Show error state with "Close Tab" or "Go to Overview" options
- **Drag-drop between tabs:** Hovering tab for 1s switches to it
- **Closing last tab:** Prevented, show tooltip "Cannot close last tab"

## Performance Targets

- Active tab render: <100ms
- Tab switch: <50ms
- 15 tabs total memory: <500MB
- No memory leaks over 100 tab cycles


## Verification (2026-10-02)

Production Playwright passes independent history, selection and scroll, a real move between tabs, close/reopen, last-tab protection and deleted-location recovery. After 100 switches with 15 tabs, renderer PSS was 236.7 MiB and JS heap was 20.1 MiB. Whole Chromium PSS was 517.4 MiB including browser/GPU services. Native memory and uncontended latency checks remain pending.
