"""Fixture adapter for the contract harness: one note per .txt file, deletes for removed files."""
import json
import pathlib
import sys

data = json.load(sys.stdin)
folder = pathlib.Path(data["config"]["folder"])
if not folder.is_dir():
    print(f"{folder} is not a folder", file=sys.stderr)
    sys.exit(2)
seen = json.loads(data["cursor"]) if data["cursor"] else []
names = sorted(path.name for path in folder.glob("*.txt"))
for name in names:
    body = (folder / name).read_text(encoding="utf-8")
    fields = {"title": name[:-4], "body": body, "words": len(body.split())}
    print(json.dumps({"upsert": "note", "external_id": name, "fields": fields}))
for name in sorted(set(seen) - set(names)):
    print(json.dumps({"delete": "note", "external_id": name}))
print(json.dumps({"cursor": json.dumps(names)}))
