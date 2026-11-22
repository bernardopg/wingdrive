import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "coverage-ratchet.py"


def load_module():
	spec = importlib.util.spec_from_file_location("coverage_ratchet", SCRIPT)
	module = importlib.util.module_from_spec(spec)
	assert spec.loader is not None
	sys.modules[spec.name] = module
	spec.loader.exec_module(module)
	return module


ratchet = load_module()


def write_json(path: Path, data) -> Path:
	path.write_text(json.dumps(data))
	return path


def summary_with(components, **overrides):
	base = {
		"commit": "abc123",
		"generated": "2026-11-22T00:00Z",
		"total": {"files": 1, "lines": 100, "covered": 80, "percent": 80.0},
		"rust": {"files": 1, "lines": 100, "covered": 80, "percent": 80.0},
		"frontend": {"files": 0, "lines": 0, "covered": 0, "percent": None},
		"components": components,
	}
	base.update(overrides)
	return base


def floors_with(components, changed=80.0):
	return {
		"version": 1,
		"components": components,
		"changed_production_lines": {"min_percent": changed},
	}


class FloorsTest(unittest.TestCase):
	def test_load_floors_rejects_missing_file(self):
		with self.assertRaises(FileNotFoundError):
			ratchet.load_floors("/nope/floors.json")

	def test_load_floors_rejects_invalid_version(self):
		with tempfile.TemporaryDirectory() as td:
			p = write_json(Path(td) / "f.json", {"version": 2, "components": {"core": {"min_percent": 1}}})
			with self.assertRaises(ValueError):
				ratchet.load_floors(p)

	def test_load_floors_rejects_missing_components(self):
		with tempfile.TemporaryDirectory() as td:
			p = write_json(Path(td) / "f.json", {"version": 1})
			with self.assertRaises(ValueError):
				ratchet.load_floors(p)


class ComponentFloorTest(unittest.TestCase):
	def test_passes_at_boundary(self):
		summary = summary_with([
			{"id": "core", "title": "Core", "files": 1, "lines": 1000, "covered": 650, "percent": 65.0},
		])
		floors = floors_with({"core": {"min_percent": 65.0, "min_lines": 1000}})
		self.assertEqual(ratchet.check_component_floors(summary, floors), [])

	def test_fails_below_floor(self):
		summary = summary_with([
			{"id": "core", "title": "Core", "files": 1, "lines": 1000, "covered": 649, "percent": 64.9},
		])
		floors = floors_with({"core": {"min_percent": 65.0, "min_lines": 1000}})
		errors = ratchet.check_component_floors(summary, floors)
		self.assertTrue(any("regressed below floor" in e for e in errors))

	def test_fails_missing_component(self):
		summary = summary_with([])
		floors = floors_with({"core": {"min_percent": 65.0, "min_lines": 0}})
		errors = ratchet.check_component_floors(summary, floors)
		self.assertTrue(any("Missing component" in e for e in errors))

	def test_fails_incomplete_counts(self):
		summary = summary_with([
			{"id": "core", "title": "Core", "files": 1, "lines": None, "covered": 1, "percent": 1.0},
		])
		floors = floors_with({"core": {"min_percent": 1.0, "min_lines": 0}})
		errors = ratchet.check_component_floors(summary, floors)
		self.assertTrue(any("incomplete line counts" in e for e in errors))

	def test_fails_smaller_denominator(self):
		summary = summary_with([
			{"id": "core", "title": "Core", "files": 1, "lines": 900, "covered": 900, "percent": 100.0},
		])
		floors = floors_with({"core": {"min_percent": 65.0, "min_lines": 1000}})
		errors = ratchet.check_component_floors(summary, floors)
		self.assertTrue(any("denominator unexpectedly smaller" in e for e in errors))

	def test_fails_no_instrumented_lines(self):
		summary = summary_with([
			{"id": "core", "title": "Core", "files": 0, "lines": 0, "covered": 0, "percent": None},
		])
		floors = floors_with({"core": {"min_percent": 1.0, "min_lines": 0}})
		errors = ratchet.check_component_floors(summary, floors)
		self.assertTrue(any("no instrumented lines" in e for e in errors))


class LcovParseTest(unittest.TestCase):
	def test_relative_repo_path_skips_tests_and_generated(self):
		self.assertIsNone(ratchet.relative_repo_path(str(ROOT / "core/tests/foo.rs")))
		self.assertIsNone(ratchet.relative_repo_path(str(ROOT / "packages/x/src/a.test.ts")))
		self.assertIsNone(ratchet.relative_repo_path(str(ROOT / "target/generated/out.rs")))
		self.assertEqual(ratchet.relative_repo_path(str(ROOT / "core/src/lib.rs")), "core/src/lib.rs")

	def test_parse_lcov_normalizes_paths_and_skips_tests(self):
		with tempfile.TemporaryDirectory() as td:
			lcov = Path(td) / "a.lcov"
			lcov.write_text(
				f"SF:{ROOT / 'core/src/lib.rs'}\nDA:1,1\nDA:2,0\nend_of_record\n"
				f"SF:{ROOT / 'core/tests/foo_test.rs'}\nDA:1,1\nend_of_record\n"
			)
			parsed = ratchet.parse_lcov([lcov])
			self.assertIn("core/src/lib.rs", parsed)
			self.assertNotIn("core/tests/foo_test.rs", parsed)
			self.assertEqual(parsed["core/src/lib.rs"], {1: 1, 2: 0})

	def test_parse_lcov_merges_hits(self):
		with tempfile.TemporaryDirectory() as td:
			a = Path(td) / "a.lcov"
			b = Path(td) / "b.lcov"
			a.write_text("SF:core/src/lib.rs\nDA:1,0\nDA:2,1\nend_of_record\n")
			b.write_text("SF:core/src/lib.rs\nDA:1,3\nDA:2,0\nend_of_record\n")
			parsed = ratchet.parse_lcov([a, b])
			self.assertEqual(parsed["core/src/lib.rs"], {1: 3, 2: 1})


class ChangedLinesTest(unittest.TestCase):
	def test_changed_lines_fail_when_no_instrumented_lines(self):
		with tempfile.TemporaryDirectory() as td:
			repo = Path(td)
			subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.email", "t@t"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.name", "t"], cwd=repo, check=True)
			(repo / "src").mkdir()
			f = repo / "src/lib.rs"
			f.write_text("fn a() {}\n")
			subprocess.run(["git", "add", "."], cwd=repo, check=True)
			subprocess.run(["git", "commit", "-qm", "init"], cwd=repo, check=True)
			subprocess.run(["git", "checkout", "-qb", "feat"], cwd=repo, check=True)
			f.write_text("fn a() {}\nfn b() {}\n")
			subprocess.run(["git", "commit", "-am", "change"], cwd=repo, check=True)
			lcov = repo / "cov.lcov"
			lcov.write_text("SF:src/other.rs\nDA:1,1\nend_of_record\n")
			# relative_repo_path will treat src/lib.rs as outside ROOT? No, it's in temp repo.
			# We pass repo_root explicitly; but relative_repo_path uses global ROOT.
			# So this test exercises behavior through changed_lines_from_diff with repo_root.
			# For parse_lcov we need paths relative to ROOT or absolute under ROOT; easier to test via module ROOT patch.
			old_root = ratchet.ROOT
			ratchet.ROOT = repo
			try:
				errors, details = ratchet.check_changed_production_lines([lcov], "HEAD~1", 80.0, repo_root=repo)
			finally:
				ratchet.ROOT = old_root
			self.assertTrue(errors)
			self.assertEqual(details["considered_lines"], 0)

	def test_changed_lines_at_boundary_and_path_normalization(self):
		with tempfile.TemporaryDirectory() as td:
			repo = Path(td)
			subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.email", "t@t"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.name", "t"], cwd=repo, check=True)
			(repo / "core/src").mkdir(parents=True)
			f = repo / "core/src/lib.rs"
			f.write_text("line1\nline2\nline3\nline4\n")
			subprocess.run(["git", "add", "."], cwd=repo, check=True)
			subprocess.run(["git", "commit", "-qm", "init"], cwd=repo, check=True)
			subprocess.run(["git", "checkout", "-qb", "feat"], cwd=repo, check=True)
			f.write_text("line1\nchanged2\nchanged3\nline4\nchanged5\n")
			subprocess.run(["git", "commit", "-am", "change"], cwd=repo, check=True)
			# LCOV uses absolute path; changed lines are 2,3,5 - cover 4 of 5? let's set 4/5 = 80%.
			lcov = repo / "cov.lcov"
			lcov.write_text(
				f"SF:{repo / 'core/src/lib.rs'}\nDA:2,1\nDA:3,1\nDA:5,1\nDA:4,0\nend_of_record\n"
			)
			old_root = ratchet.ROOT
			ratchet.ROOT = repo
			try:
				errors, details = ratchet.check_changed_production_lines([lcov], "HEAD~1", 80.0, repo_root=repo)
			finally:
				ratchet.ROOT = old_root
			self.assertEqual(errors, [])
			self.assertEqual(details["covered_lines"], 3)
			self.assertEqual(details["considered_lines"], 3)
			self.assertEqual(details["percent"], 100.0)

	def test_changed_lines_reports_uncovered(self):
		with tempfile.TemporaryDirectory() as td:
			repo = Path(td)
			subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.email", "t@t"], cwd=repo, check=True)
			subprocess.run(["git", "config", "user.name", "t"], cwd=repo, check=True)
			(repo / "core/src").mkdir(parents=True)
			f = repo / "core/src/lib.rs"
			f.write_text("a\n")
			subprocess.run(["git", "add", "."], cwd=repo, check=True)
			subprocess.run(["git", "commit", "-qm", "init"], cwd=repo, check=True)
			subprocess.run(["git", "checkout", "-qb", "feat"], cwd=repo, check=True)
			f.write_text("a\nb\nc\n")
			subprocess.run(["git", "commit", "-am", "change"], cwd=repo, check=True)
			lcov = repo / "cov.lcov"
			lcov.write_text(f"SF:{repo / 'core/src/lib.rs'}\nDA:2,0\nDA:3,1\nend_of_record\n")
			old_root = ratchet.ROOT
			ratchet.ROOT = repo
			try:
				errors, details = ratchet.check_changed_production_lines([lcov], "HEAD~1", 80.0, repo_root=repo)
			finally:
				ratchet.ROOT = old_root
			self.assertTrue(any("below floor" in e for e in errors))
			self.assertEqual(details["percent"], 50.0)
			self.assertTrue(any("core/src/lib.rs:2" in e for e in errors))


class CliTest(unittest.TestCase):
	def test_check_command_writes_json_out(self):
		with tempfile.TemporaryDirectory() as td:
			td = Path(td)
			floors = write_json(td / "floors.json", floors_with({"core": {"min_percent": 50.0, "min_lines": 10}}))
			summary = write_json(td / "summary.json", summary_with([
				{"id": "core", "title": "Core", "files": 1, "lines": 100, "covered": 60, "percent": 60.0},
			]))
			out = td / "out.json"
			code = ratchet.main(["check", "--floors", str(floors), "--summary", str(summary), "--json-out", str(out)])
			self.assertEqual(code, 0)
			report = json.loads(out.read_text())
			self.assertTrue(report["ok"])

	def test_check_command_fails_missing_summary(self):
		with tempfile.TemporaryDirectory() as td:
			td = Path(td)
			floors = write_json(td / "floors.json", floors_with({"core": {"min_percent": 50.0, "min_lines": 0}}))
			code = ratchet.main(["check", "--floors", str(floors), "--summary", str(td / "missing.json")])
			self.assertEqual(code, 1)


if __name__ == "__main__":
	unittest.main()
