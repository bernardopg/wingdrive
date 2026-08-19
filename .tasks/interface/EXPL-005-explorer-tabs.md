---
id: EXPL-005
title: Explorer Tabs
status: In Progress
assignee: unassigned
parent: EXPL-000
priority: High
tags: [explorer, tabs, navigation, ui]
last_updated: 2026-08-19
related_tasks: []
---

## Description

Add browser-like tabs to Spacedrive Explorer, enabling users to browse multiple locations simultaneously. This requires careful integration with the keybind system, proper UI rendering, and preservation of context (selection, view mode, scroll position, navigation history) across tabs.

Phases 1-3 and tab context menu + reopen closed tab shipped (commits 0276b0742, 99b8cfd1c). Remaining: perf (phase 4), persistence (phase 5), drag-reorder and close animations (phase 6).

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

**Challenge:** React Router v6 only supports one browser router per app

**Solution:** Dynamic router type switching
- **Active tab:** `createBrowserRouter` (syncs to URL bar)
- **Inactive tabs:** `createMemoryRouter` (in-memory only)
- **On tab switch:** Swap router types

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
- [x] `TabView.tsx` rendering logic implemented
- [x] `useTabManager.ts` hook created
- [x] `Explorer.tsx` wrapped in TabManagerProvider
- [x] `context.tsx` updated with `isActiveTab` prop
- [x] `SelectionContext.tsx` updated with `isActiveTab` prop
- [x] App launches with single tab, no regressions

### Phase 2: Multi-Tab State
- [x] Create/close/switch tabs functional
- [x] Independent navigation per tab
- [x] Router type swapping works correctly
- [ ] 5+ tabs with isolated state, <50ms tab switching

### Phase 3: Keybinds
- [x] `explorer.openInNewTab` keybind removed (conflicts with global)
- [x] `tabs.newTab` (Cmd+T) - creates new tab
- [x] `tabs.closeTab` (Cmd+W) - closes active tab
- [x] `tabs.nextTab` (Cmd+Shift+]) - switches to next tab
- [x] `tabs.previousTab` (Cmd+Shift+[) - switches to previous tab
- [x] `tabs.selectTab1-9` (Cmd+1-9) - jumps to specific tab

### Phase 4: Performance
- [ ] Lazy mounting (active + 2 recent tabs only)
- [ ] Query GC for inactive tabs
- [ ] Scroll position preservation per tab
- [ ] 15 tabs <500MB memory
- [ ] No memory leaks over 100 tab cycles

### Phase 5: Persistence
- [ ] Tabs serialized on app quit
- [ ] Tabs restored on launch
- [ ] Stale tabs handled gracefully (deleted locations)
- [ ] `tabPreferences.ts` store created

### Phase 6: Polish (Post-MVP)
- [x] Tab context menu
- [ ] Drag-to-reorder tabs
- [ ] Cross-tab file drag-drop
- [ ] Tab close animations
- [x] "Reopen Closed Tab" (Cmd+Shift+T)

## Implementation Files

To be created:
- `packages/interface/src/components/TabManager/TabManagerContext.tsx`
- `packages/interface/src/components/TabManager/TabBar.tsx`
- `packages/interface/src/components/TabManager/TabView.tsx`
- `packages/interface/src/components/TabManager/useTabManager.ts`
- `packages/interface/src/components/TabManager/TabContextMenu.tsx` (Phase 6)
- `packages/interface/src/components/TabManager/index.ts`
- `packages/ts-client/src/stores/tabPreferences.ts`

To be modified:
- `packages/interface/src/Explorer.tsx`
- `packages/interface/src/components/Explorer/context.tsx`
- `packages/interface/src/components/Explorer/SelectionContext.tsx`
- `packages/interface/src/components/Explorer/views/GridView/GridView.tsx`
- `packages/interface/src/components/Explorer/views/ListView/ListView.tsx`
- `packages/interface/src/util/keybinds/registry.ts`

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

