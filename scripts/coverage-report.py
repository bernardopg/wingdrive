#!/usr/bin/env python3
"""Merge lcov reports into per-component coverage, badges and a summary.

Usage: coverage-report.py <out-dir> <lcov.info>...

Writes to <out-dir>:
  badges/<id>.json   shields.io endpoint for the whole project, each language
                     (rust, frontend) and each component in COMPONENTS
  summary.json       the same numbers, with the commit and the time
  summary.md         a Markdown table for the workflow summary
  merged.lcov        every report merged, paths relative to the repository

A line counts as covered when any report hit it. Paths outside the repository
(dependencies), tests, benchmarks and generated code are left out. COVERAGE.md
reads the badges, so its numbers follow every run without edits.
"""
import datetime
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# (id, title, path prefixes). A file belongs to the first component that matches.
COMPONENTS = [
	("core", "Core: daemon, operations, sync", ["core/"]),
	("desktop", "Desktop: Tauri shell and its crates", ["apps/tauri/"]),
	("cli", "CLI (`wing`)", ["apps/cli/"]),
	("server", "Server (`wing-server`)", ["apps/server/"]),
	("mobile", "Mobile core", ["apps/mobile/"]),
	("crates", "Shared crates", ["crates/"]),
	("interface", "Interface (React)", ["packages/interface/"]),
	("ts-client", "TypeScript client", ["packages/ts-client/"]),
	("packages", "Other packages", ["packages/"]),
]
RUST = {".rs"}
SKIP = ("/node_modules/", "/target/", "/tests/", "/benches/", "/generated/", "core/benchmarks/", "xtask/")


def relative(path):
	p = Path(path)
	if not p.is_absolute():
		p = ROOT / p
	try:
		rel = p.resolve().relative_to(ROOT).as_posix()
	except ValueError:
		return None
	if any(part in f"/{rel}" for part in SKIP) or ".test." in rel or "_test.rs" in rel:
		return None
	return rel


def merge(paths):
	lines: dict[str, dict[int, int]] = {}
	for path in paths:
		current = None
		for raw in Path(path).read_text(errors="replace").splitlines():
			if raw.startswith("SF:"):
				current = relative(raw[3:])
				if current:
					lines.setdefault(current, {})
			elif raw.startswith("DA:") and current:
				number, hits = raw[3:].split(",")[:2]
				counts = lines[current]
				n = int(number)
				counts[n] = max(counts.get(n, 0), int(float(hits)))
			elif raw == "end_of_record":
				current = None
	return lines


def component_of(rel):
	for cid, _, prefixes in COMPONENTS:
		if any(rel.startswith(p) for p in prefixes):
			return cid
	return None


def stats(files):
	found = sum(len(c) for c in files.values())
	hit = sum(1 for c in files.values() for h in c.values() if h > 0)
	return {"files": len(files), "lines": found, "covered": hit, "percent": round(100 * hit / found, 1) if found else None}


def color(percent):
	if percent is None:
		return "lightgrey"
	for limit, name in ((80, "brightgreen"), (60, "green"), (40, "yellowgreen"), (25, "yellow"), (10, "orange")):
		if percent >= limit:
			return name
	return "red"


def badge(label, s):
	message = f"{s['percent']:.0f}%" if s["percent"] is not None else "no tests"
	return {"schemaVersion": 1, "label": label, "message": message, "color": color(s["percent"])}


PAGE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>WingDrive coverage</title>
<style>
:root{{--bg:#14151c;--panel:#1c1d26;--line:#2a2c38;--ink:#e6e7ee;--dim:#9a9cab;--accent:#7c3aed}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font:14px/1.5 system-ui,sans-serif}}
main{{max-width:1100px;margin:0 auto;padding:32px 16px}}h1{{margin:0 0 4px;font-size:24px}}
.meta{{color:var(--dim);margin-bottom:24px}}a{{color:#a78bfa}}
.cards{{display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px;margin-bottom:28px}}
.card{{background:var(--panel);border:1px solid var(--line);border-radius:10px;padding:14px}}
.card b{{display:block;font-size:22px}}.card span{{color:var(--dim);font-size:12px}}
.bar{{height:6px;background:var(--line);border-radius:3px;margin-top:8px;overflow:hidden}}
.bar i{{display:block;height:100%;background:var(--accent)}}
input{{width:100%;padding:8px 10px;margin-bottom:12px;background:var(--panel);border:1px solid var(--line);border-radius:8px;color:var(--ink)}}
table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:6px 8px;border-bottom:1px solid var(--line)}}
th{{color:var(--dim);font-weight:500;cursor:pointer}}td.n{{text-align:right;font-variant-numeric:tabular-nums}}
td.p{{width:160px}}code{{font-size:12px}}
</style></head><body><main>
<h1>WingDrive coverage</h1>
<div class="meta">Commit <code>{commit}</code> · {generated} · line coverage of the tests CI runs ·
<a href="https://github.com/bernardopg/wingdrive/blob/main/COVERAGE.md">how it is measured</a></div>
<div class="cards">{cards}</div>
<input id="q" placeholder="Filter files">
<table id="t"><thead><tr><th>File</th><th>Component</th><th class="n">Covered</th><th class="n">Lines</th><th>Coverage</th></tr></thead>
<tbody>{rows}</tbody></table>
</main><script>
const q=document.getElementById('q'),rows=[...document.querySelectorAll('#t tbody tr')];
q.oninput=()=>{{const v=q.value.toLowerCase();rows.forEach(r=>r.hidden=!r.textContent.toLowerCase().includes(v))}};
document.querySelectorAll('#t th').forEach((th,i)=>th.onclick=()=>{{const b=th.parentNode.parentNode.nextElementSibling;
const s=[...b.rows].sort((a,c)=>{{const x=a.cells[i].dataset.v??a.cells[i].textContent,y=c.cells[i].dataset.v??c.cells[i].textContent;
return isNaN(x)?x.localeCompare(y):x-y}});if(th.dataset.d)s.reverse();th.dataset.d=th.dataset.d?'':'1';s.forEach(r=>b.append(r))}});
</script></body></html>
"""


def html_report(summary, per_component, files):
	from html import escape

	def pct(s):
		return f"{s['percent']:.1f}%" if s["percent"] is not None else "no tests"

	def card(title, s):
		width = s["percent"] or 0
		return (f'<div class="card"><span>{escape(title)}</span><b>{pct(s)}</b>'
			f'<span>{s["covered"]:,} / {s["lines"]:,} lines</span><div class="bar"><i style="width:{width}%"></i></div></div>')

	cards = [card("Total", summary["total"]), card("Rust", summary["rust"]), card("Frontend", summary["frontend"])]
	cards += [card(c["title"].replace("`", ""), c) for c in summary["components"] if c["lines"]]
	owner = {rel: cid for cid, members in per_component.items() for rel in members}
	rows = []
	for rel, counts in sorted(files.items(), key=lambda kv: kv[0]):
		found = len(counts)
		hit = sum(1 for h in counts.values() if h > 0)
		p = 100 * hit / found if found else 0
		rows.append(f'<tr><td><a href="https://github.com/bernardopg/wingdrive/blob/{summary["commit"] or "main"}/{escape(rel)}">'
			f'<code>{escape(rel)}</code></a></td><td>{owner.get(rel, "")}</td><td class="n">{hit}</td><td class="n">{found}</td>'
			f'<td class="p" data-v="{p:.1f}">{p:.1f}%<div class="bar"><i style="width:{p:.1f}%"></i></div></td></tr>')
	return PAGE.format(commit=escape(summary["commit"][:9] or "local"), generated=summary["generated"],
		cards="".join(cards), rows="\n".join(rows))


def main():
	out, *reports = sys.argv[1:]
	out = Path(out)
	(out / "badges").mkdir(parents=True, exist_ok=True)
	lines = merge(reports)

	groups = {"total": lines, "rust": {}, "frontend": {}}
	per_component = {cid: {} for cid, _, _ in COMPONENTS}
	for rel, counts in lines.items():
		groups["rust" if Path(rel).suffix in RUST else "frontend"][rel] = counts
		cid = component_of(rel)
		if cid:
			per_component[cid][rel] = counts

	summary = {
		"commit": os.environ.get("GITHUB_SHA", ""),
		"generated": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%MZ"),
		"total": stats(groups["total"]),
		"rust": stats(groups["rust"]),
		"frontend": stats(groups["frontend"]),
		"components": [{"id": cid, "title": title, **stats(per_component[cid])} for cid, title, _ in COMPONENTS],
	}

	writes = {"coverage": ("coverage", summary["total"]), "rust": ("rust coverage", summary["rust"]),
		"frontend-coverage": ("frontend coverage", summary["frontend"])}
	for cid, title, _ in COMPONENTS:
		writes[f"component-{cid}"] = (cid, stats(per_component[cid]))
	for name, (label, s) in writes.items():
		(out / "badges" / f"{name}.json").write_text(json.dumps(badge(label, s)) + "\n")
	(out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")

	def row(title, s):
		pct = f"{s['percent']:.1f}%" if s["percent"] is not None else "no tests"
		return f"| {title} | {pct} | {s['covered']:,} / {s['lines']:,} | {s['files']} |"

	md = ["| Area | Lines covered | Covered / instrumented | Files |", "|---|---|---|---|",
		row("**Total**", summary["total"]), row("Rust", summary["rust"]), row("Frontend", summary["frontend"])]
	md += [row(c["title"], c) for c in summary["components"]]
	(out / "summary.md").write_text("\n".join(md) + "\n")

	(out / "index.html").write_text(html_report(summary, per_component, groups["total"]))

	with open(out / "merged.lcov", "w") as f:
		for rel, counts in sorted(lines.items()):
			f.write(f"SF:{rel}\n")
			for n in sorted(counts):
				f.write(f"DA:{n},{counts[n]}\n")
			f.write(f"LF:{len(counts)}\nLH:{sum(1 for h in counts.values() if h > 0)}\nend_of_record\n")
	print("\n".join(md))


if __name__ == "__main__":
	main()
