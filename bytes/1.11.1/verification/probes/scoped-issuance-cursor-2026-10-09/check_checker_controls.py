#!/usr/bin/env python3
"""Run source mutations against the closed-client checker without a prover."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_closed_scope.py"
FIXTURE_PATH = ROOT / "fixtures" / "checker-controls.json"
RESULT_PATH = ROOT / "fixtures" / "checker-control-results.json"
SOURCE_PATH = ROOT / "src" / "driver.rs"
LIB_PATH = ROOT / "src" / "lib.rs"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


spec = importlib.util.spec_from_file_location("closed_scope_checker", CHECKER_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError("could not load check_closed_scope.py")
sys.dont_write_bytecode = True
checker = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = checker
spec.loader.exec_module(checker)


def once(source: str, old: str, new: str) -> str:
    if source.count(old) != 1:
        raise AssertionError(f"mutation anchor must occur exactly once: {old!r}")
    return source.replace(old, new, 1)


def mutate(case_id: str, source: str) -> str:
    line_one = "    let (_, second) = registry.register(first.borrow(), cursor.borrow_mut());\n"
    line_two = "    let (_, third) = registry.register(first.borrow(), cursor.borrow_mut());\n"
    if case_id == "escaped_clone":
        return once(source, line_one, line_one + "    let escaped = second.clone();\n")
    if case_id == "unfinished_callback":
        callback = "    let unfinished = || { registry.register(first.borrow(), cursor.borrow_mut()); };\n"
        return once(source, line_one, line_one + callback)
    if case_id == "forgotten_live":
        return once(source, line_one, line_one + "    core::mem::forget(second);\n")
    if case_id == "thread_escape":
        return once(source, line_one,
                    line_one + "    std::thread::spawn(move || { let _ = second; });\n")
    if case_id == "raw_alias":
        anchor = "    let (registry, first, mut cursor) = Registry::new(\n"
        return once(source, anchor,
                    anchor + "    let raw = &mut cursor as *mut _;\n")
    if case_id == "untracked_event":
        return once(source, line_two,
                    "    let (_, third) = registry.register(first.borrow());\n")
    if case_id == "fakecomment":
        return once(source, line_two,
                    "    // registry.register(first.borrow(), cursor.borrow_mut());\n")
    if case_id == "fakestring":
        return once(source, line_two,
                    '    let marker = "registry.register(first.borrow(), cursor.borrow_mut());";\n')
    if case_id == "wrong_target":
        return once(source, line_two,
                    "    let (_, third) = registry.register(second.borrow(), cursor.borrow_mut());\n")
    if case_id == "wrong_order":
        third = (
            "    let (third_last, third_recovery) = registry.retire(third, cursor.borrow_mut());\n"
            "    proof_assert!(!third_last && third_recovery.inner_logic() == None);\n"
        )
        first = (
            "    let (first_last, first_recovery) = registry.retire(first, cursor.borrow_mut());\n"
            "    proof_assert!(!first_last && first_recovery.inner_logic() == None);\n"
        )
        return once(source, third + first, first + third)
    if case_id == "duplicate":
        return once(source, line_one,
                    line_one + "    let (_, duplicate) = registry.register(first.borrow(), cursor.borrow_mut());\n")
    if case_id == "omitted":
        return once(source, line_two, "")
    if case_id == "import_override":
        return once(source, "use super::*;\n",
                    "use super::*;\nuse crate::shadow::Registry;\n")
    if case_id == "fake_fn":
        return source + "\npub fn closed_sparse_lifecycle() -> bool { true }\n"
    if case_id == "module_alias":
        return source
    raise AssertionError(f"unknown checker-control mutation: {case_id}")


def mutate_lib(case_id: str, source: str) -> str:
    if case_id == "module_alias":
        return once(source, "pub use scoped::Registry;\n",
                    "use scoped as registry_impl;\npub use registry_impl::Registry;\n")
    return source


def main() -> int:
    fixture = json.loads(FIXTURE_PATH.read_text())
    source_bytes = SOURCE_PATH.read_bytes()
    source = source_bytes.decode()
    lib_bytes = LIB_PATH.read_bytes()
    lib_source = lib_bytes.decode()
    baseline = checker.audit_source(source, source_name="src/driver.rs",
                                    lib_source=lib_source)
    cases = []
    for row in fixture["controls"]:
        mutated = mutate(row["id"], source)
        mutated_lib = mutate_lib(row["id"], lib_source)
        try:
            checker.audit_source(mutated, source_name=f"<mutation:{row['id']}>", lib_source=mutated_lib)
            actual = "correspondence_pass"
            error = None
        except checker.CoverageError as exc:
            actual = "coverage_failure"
            error = str(exc)
        cases.append({
            "id": row["id"],
            "expected": row["expected"],
            "actual": actual,
            "pass": actual == row["expected"],
            "error": error,
            "prover_invoked": False,
        })
    report = {
        "schema_version": 1,
        "status": "controls_pass" if all(x["pass"] for x in cases) else "controls_failed",
        "checker_sha256": sha(CHECKER_PATH.read_bytes()),
        "fixture_sha256": sha(FIXTURE_PATH.read_bytes()),
        "source_sha256": sha(source_bytes),
        "lib_source_sha256": sha(lib_bytes),
        "baseline": {
            "status": baseline["status"],
            "source_sha256": baseline["source_sha256"],
            "support_identity": baseline["support_identity"],
            "events": baseline["events"],
            "live_tickets_at_normal_return": baseline["live_tickets_at_normal_return"],
            "ids_assumed": baseline["ids_assumed"],
        },
        "controls": cases,
        "prover_invoked": False,
        "semantic_boundary": fixture["semantic_boundary"],
    }
    RESULT_PATH.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({
        "status": report["status"],
        "controls": len(cases),
        "failures": [x for x in cases if not x["pass"]],
        "report": RESULT_PATH.as_posix(),
    }, indent=2))
    return 0 if report["status"] == "controls_pass" else 2


if __name__ == "__main__":
    raise SystemExit(main())
