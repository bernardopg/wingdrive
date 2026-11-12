---
id: DESK-091
title: Select by Pattern turns `[!]` into a class that matches any character
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S05
tags: [desktop, explorer, selection, bug]
last_updated: 2026-11-11
---

## Description

`globMatcher` (DESK-019) closes a bracket class at the first `]` after the `[`. For `[!]` that gives
the body `!`, which becomes the JavaScript class `[^]`, and `[^]` matches any character. So
`a[!]` selects `ab`, while the shell treats `[!]` as an unterminated class and matches the literal
name `a[!]`. The same rule breaks the POSIX form with `]` as the first class member: `[]]x` should
match `]x` and `[!]]x` should match `ax`, and both matched nothing. Found while adding edge cases to
`selectionPattern.test.ts` on 2026-11-11.

## Steps to reproduce

1. In a folder with `ab` and `a[!]`, press Ctrl+S.
2. Type `a[!]` and confirm.
3. `ab` is selected instead of `a[!]`.

## Acceptance Criteria

- [x] A `]` right after `[`, `[!` or `[^` is a class member, as in the shell
- [x] An unterminated class matches its characters literally
- [x] `selectionPattern.test.ts` covers these cases

## Fix

`globMatcher` now reads a class the way the shell does: `[!` or `[^` negates, a `]` right after `[`,
`[!` or `[^` is a member, and the members are escaped (`\`, `]`, `^`) before they go into the
regular expression. A class with no closing `]` stays literal, so `a[!]` matches only `a[!]`.
"reads bracket classes the way the shell does" in `selectionPattern.test.ts` covers `a[!]`, `[]]x`,
`[!]]x`, `[^a]`, `[a^]` and a backslash member; `bun test
packages/interface/src/routes/explorer/hooks/selectionPattern.test.ts` passes 6 tests.
