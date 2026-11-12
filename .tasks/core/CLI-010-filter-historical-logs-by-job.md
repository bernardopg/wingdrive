---
id: CLI-010
title: Filter historical logs by job ID
status: Done
assignee: bernardopg
parent: CLI-000
priority: Medium
milestone: M3
tags: [cli, logs]
last_updated: 2026-11-11
---

## Description

`wing logs show --job-id <id>` and the historical portion of `wing logs follow --job-id <id>` include lines from other jobs. The historical formatter ignores the selected job ID.

## Reproduction

Format two daemon log lines containing `job_id=job-1` and `job_id=job-2` with the filter set to `job-1`. The second line is included.

## Implementation Steps

- [x] Filter historical lines by their `job_id` field.
- [x] Cover matching, other and unstructured lines with a regression test.

## Acceptance Criteria

- [x] Historical output includes only lines with the selected job ID.
- [x] Without a job filter, existing output remains unchanged.

## Validation

`domains::logs::tests::historical_logs_filter_by_job_id` covers matching, quoted,
different and prefix-matching IDs, plus an unstructured line. The existing
`malformed_log_lines_remain_readable` test covers output without a filter.
`cargo test -p wing-cli` passed on 2026-11-12: 556 unit tests and 6 process tests.
