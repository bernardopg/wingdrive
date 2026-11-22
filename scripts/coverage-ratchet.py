#!/usr/bin/env python3
"""Coverage ratchet gate for WingDrive.

Checks component floors against a coverage summary, and optionally checks
coverage of changed production lines against an LCOV report.

Fails for missing/incomplete reports, unexpectedly smaller denominators,
regressions below floors, or insufficient changed-line coverage.
Never updates the floors file automatically.

Usage:
  coverage-ratchet.py check \
    --floors scripts/coverage-floors.json \
    --summary coverage/site/summary.json \
    [--lcov coverage/site/merged.lcov ...] \
    [--diff-base origin/main] \
    [--changed-lines-min-percent 80]

  coverage-ratchet.py changed-lines \
    --lcov coverage/site/merged.lcov \
    --diff-base origin/main \
    --min-percent 80
"""
from __future__ import annotations

import argparse
import datetime
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Keep in sync with scripts/coverage-report.py.
SKIP_PARTS = ("/node_modules/", "/target/", "/tests/", "/benches/", "/generated/", "core/benchmarks/", "xtask/")


def relative_repo_path(path: str | Path) -> str | None:
	p = Path(path)
	if not p.is_absolute():
		p = (ROOT / p).resolve()
	else:
		try:
			p = p.resolve()
		except OSError:
			return None
	try:
		rel = p.relative_to(ROOT).as_posix()
	except ValueError:
		return None
	if any(part in f"/{rel}" for part in SKIP_PARTS) or ".test." in rel or "_test.rs" in rel:
		return None
	return rel


def is_production_path(rel: str) -> bool:
	return relative_repo_path(rel) is not None


def load_json(path: str | Path, label: str) -> dict:
	p = Path(path)
	if not p.exists():
		raise FileNotFoundError(f"{label} not found: {p}")
	try:
		return json.loads(p.read_text())
	except json.JSONDecodeError as exc:
		raise ValueError(f"{label} is not valid JSON: {p}: {exc}") from exc


def load_floors(path: str | Path) -> dict:
	floors = load_json(path, "Floors file")
	if floors.get("version") != 1:
		raise ValueError(f"Unsupported floors version: {floors.get('version')!r}")
	components = floors.get("components")
	if not isinstance(components, dict) or not components:
		raise ValueError("Floors file missing non-empty 'components' map")
	for cid, cfg in components.items():
		if not isinstance(cfg, dict):
			raise ValueError(f"Floor for component {cid!r} must be an object")
		if "min_percent" not in cfg:
			raise ValueError(f"Floor for component {cid!r} missing min_percent")
		min_percent = cfg["min_percent"]
		if not isinstance(min_percent, (int, float)) or not (0 <= float(min_percent) <= 100):
			raise ValueError(f"Floor for component {cid!r} has invalid min_percent: {min_percent!r}")
		min_lines = cfg.get("min_lines", 0)
		if not isinstance(min_lines, int) or min_lines < 0:
			raise ValueError(f"Floor for component {cid!r} has invalid min_lines: {min_lines!r}")
	changed = floors.get("changed_production_lines", {})
	if changed:
		m = changed.get("min_percent", 80.0)
		if not isinstance(m, (int, float)) or not (0 <= float(m) <= 100):
			raise ValueError(f"changed_production_lines.min_percent is invalid: {m!r}")
	return floors


def load_summary(path: str | Path) -> dict:
	summary = load_json(path, "Summary")
	for key in ("total", "rust", "frontend", "components"):
		if key not in summary:
			raise ValueError(f"Summary missing required key: {key}")
	if not isinstance(summary["components"], list):
		raise ValueError("Summary 'components' must be a list")
	return summary


def component_map(summary: dict) -> dict[str, dict]:
	out = {}
	for comp in summary.get("components", []):
		cid = comp.get("id")
		if not cid:
			raise ValueError("Summary component missing id")
		out[cid] = comp
	return out


def check_component_floors(summary: dict, floors: dict) -> list[str]:
	errors: list[str] = []
	components = component_map(summary)
	for cid, cfg in floors["components"].items():
		comp = components.get(cid)
		if comp is None:
			errors.append(f"Missing component in summary: {cid}")
			continue
		lines = comp.get("lines")
		covered = comp.get("covered")
		percent = comp.get("percent")
		if not isinstance(lines, int) or not isinstance(covered, int):
			errors.append(f"Component {cid} has incomplete line counts in summary")
			continue
		if lines <= 0 or percent is None:
			errors.append(f"Component {cid} has no instrumented lines in summary")
			continue
		if not isinstance(percent, (int, float)):
			errors.append(f"Component {cid} has invalid percent in summary: {percent!r}")
			continue
		min_lines = int(cfg.get("min_lines", 0))
		if lines < min_lines:
			errors.append(
				f"Component {cid} denominator unexpectedly smaller: {lines} < min_lines {min_lines}"
			)
		min_percent = float(cfg["min_percent"])
		if float(percent) + 1e-9 < min_percent:
			errors.append(
				f"Component {cid} regressed below floor: {percent:.1f}% < {min_percent:.1f}%"
			)
	return errors


def parse_lcov(paths: list[str | Path]) -> dict[str, dict[int, int]]:
	"""Return {relative_path: {line_no: max_hits}} for production files only."""
	merged: dict[str, dict[int, int]] = {}
	for path in paths:
		p = Path(path)
		if not p.exists():
			raise FileNotFoundError(f"LCOV report not found: {p}")
		current: str | None = None
		for raw in p.read_text(errors="replace").splitlines():
			if raw.startswith("SF:"):
				current = relative_repo_path(raw[3:])
				if current:
					merged.setdefault(current, {})
			elif raw.startswith("DA:") and current:
				parts = raw[3:].split(",", 2)
				if len(parts) < 2:
					continue
				try:
					n = int(parts[0])
					hits = int(float(parts[1]))
				except ValueError:
					continue
				counts = merged[current]
				counts[n] = max(counts.get(n, 0), hits)
			elif raw == "end_of_record":
				current = None
	return merged


def changed_lines_from_diff(diff_base: str, repo_root: Path = ROOT) -> dict[str, set[int]]:
	"""Return {relative_path: {new_line_numbers}} for added/modified lines."""
	cmd = ["git", "diff", "--unified=0", "--no-color", f"{diff_base}...HEAD"]
	result = subprocess.run(cmd, cwd=repo_root, capture_output=True, text=True, check=False)
	if result.returncode != 0:
		raise RuntimeError(f"git diff failed: {result.stderr.strip()}")
	changed: dict[str, set[int]] = {}
	current: str | None = None
	for raw in result.stdout.splitlines():
		if raw.startswith("+++ b/"):
			path = raw[6:]
			current = relative_repo_path(path)
			if current:
				changed.setdefault(current, set())
			continue
		if raw.startswith("@@") and current:
			# @@ -old,count +new,count @@
			plus = raw.split("+", 1)[1].split(" ", 1)[0]
			if "," in plus:
				start_s, count_s = plus.split(",", 1)
				start = int(start_s)
				count = int(count_s)
			else:
				start = int(plus)
				count = 1
			if count <= 0:
				continue
			changed[current].update(range(start, start + count))
	return {k: v for k, v in changed.items() if v}


def check_changed_production_lines(
	lcov_paths: list[str | Path],
	diff_base: str,
	min_percent: float,
	repo_root: Path = ROOT,
) -> tuple[list[str], dict]:
	errors: list[str] = []
	if not lcov_paths:
		return ["Changed-line check requires at least one LCOV report"], {}
	coverage = parse_lcov(lcov_paths)
	changed = changed_lines_from_diff(diff_base, repo_root=repo_root)
	considered = 0
	covered = 0
	uncovered: list[str] = []
	for rel, line_numbers in sorted(changed.items()):
		file_cov = coverage.get(rel)
		if not file_cov:
			# Changed production file not present in LCOV: cannot prove coverage.
			# Count its changed instrumentable? We only know LCOV lines; if none, treat as incomplete.
			continue
		for n in sorted(line_numbers):
			if n not in file_cov:
				continue
			considered += 1
			if file_cov[n] > 0:
				covered += 1
			else:
				if len(uncovered) < 20:
					uncovered.append(f"{rel}:{n}")
	percent = (100.0 * covered / considered) if considered else None
	details = {
		"considered_lines": considered,
		"covered_lines": covered,
		"percent": round(percent, 1) if percent is not None else None,
		"uncovered_examples": uncovered,
	}
	if considered == 0:
		errors.append(
			"Changed-line check found no instrumented production lines in the provided LCOV reports"
		)
		return errors, details
	if percent + 1e-9 < min_percent:
		errors.append(
			f"Changed production line coverage below floor: {percent:.1f}% < {min_percent:.1f}%"
		)
		if uncovered:
			errors.append("Uncovered changed lines (first 20): " + ", ".join(uncovered))
	return errors, details


def cmd_check(args: argparse.Namespace) -> int:
	errors: list[str] = []
	try:
		floors = load_floors(args.floors)
		summary = load_summary(args.summary)
	except (FileNotFoundError, ValueError) as exc:
		print(f"Coverage ratchet FAILED: {exc}", file=sys.stderr)
		report = {"ok": False, "errors": [str(exc)], "checked_components": [], "changed_lines": None}
		if args.json_out:
			out = Path(args.json_out)
			out.parent.mkdir(parents=True, exist_ok=True)
			out.write_text(json.dumps(report, indent=2) + "\n")
		return 1
	errors.extend(check_component_floors(summary, floors))

	changed_cfg = floors.get("changed_production_lines", {})
	changed_min = args.changed_lines_min_percent
	if changed_min is None:
		changed_min = float(changed_cfg.get("min_percent", 80.0))
	changed_details = None
	if args.diff_base:
		if not args.lcov:
			errors.append("Changed-line check requested but no --lcov reports provided")
		else:
			changed_errors, changed_details = check_changed_production_lines(
				args.lcov, args.diff_base, float(changed_min), repo_root=Path(args.repo_root)
			)
			errors.extend(changed_errors)

	report = {
		"ok": not errors,
		"errors": errors,
		"checked_components": sorted(floors["components"].keys()),
		"changed_lines": changed_details,
		"summary_commit": summary.get("commit"),
		"generated": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%MZ"),
	}
	if args.json_out:
		out = Path(args.json_out)
		out.parent.mkdir(parents=True, exist_ok=True)
		out.write_text(json.dumps(report, indent=2) + "\n")

	if errors:
		print("Coverage ratchet FAILED:", file=sys.stderr)
		for err in errors:
			print(f"- {err}", file=sys.stderr)
		return 1

	print("Coverage ratchet OK")
	for cid in sorted(floors["components"].keys()):
		comp = component_map(summary)[cid]
		cfg = floors["components"][cid]
		print(
			f"- {cid}: {comp['percent']:.1f}% >= {float(cfg['min_percent']):.1f}%"
			f" (lines {comp['lines']}, min_lines {int(cfg.get('min_lines', 0))})"
		)
	if changed_details and changed_details.get("percent") is not None:
		print(
			f"- changed production lines: {changed_details['percent']:.1f}%"
			f" ({changed_details['covered_lines']}/{changed_details['considered_lines']})"
		)
	return 0


def cmd_changed_lines(args: argparse.Namespace) -> int:
	errors, details = check_changed_production_lines(
		args.lcov, args.diff_base, float(args.min_percent), repo_root=Path(args.repo_root)
	)
	payload = {"ok": not errors, "errors": errors, **details}
	print(json.dumps(payload, indent=2))
	return 0 if not errors else 1


def build_parser() -> argparse.ArgumentParser:
	p = argparse.ArgumentParser(description=__doc__)
	sub = p.add_subparsers(dest="cmd", required=True)

	check = sub.add_parser("check", help="Check summary against versioned floors")
	check.add_argument("--floors", required=True)
	check.add_argument("--summary", required=True)
	check.add_argument("--lcov", action="append", default=[])
	check.add_argument("--diff-base")
	check.add_argument("--changed-lines-min-percent", type=float)
	check.add_argument("--json-out")
	check.add_argument("--repo-root", default=str(ROOT))
	check.set_defaults(func=cmd_check)

	changed = sub.add_parser("changed-lines", help="Check changed production line coverage")
	changed.add_argument("--lcov", action="append", required=True)
	changed.add_argument("--diff-base", required=True)
	changed.add_argument("--min-percent", type=float, default=80.0)
	changed.add_argument("--repo-root", default=str(ROOT))
	changed.set_defaults(func=cmd_changed_lines)

	return p


def main(argv: list[str] | None = None) -> int:
	parser = build_parser()
	args = parser.parse_args(argv)
	return args.func(args)


if __name__ == "__main__":
	raise SystemExit(main())
