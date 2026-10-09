#!/usr/bin/env python3
"""Replay source-level adversarial controls for check_correspondence.py.

All mutations are in-memory. This checks that the live validator functions
reject representative contract, trust, effect, route, type, and callback
changes; it does not build Rust code or invoke Creusot/Why3.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_correspondence.py"
MANIFEST_PATH = ROOT / "generated" / "checker-controls.json"
spec = importlib.util.spec_from_file_location("al_correspondence_controls_target", CHECKER_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError(f"cannot load checker: {CHECKER_PATH}")
CHECKER = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = CHECKER
spec.loader.exec_module(CHECKER)


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: mutation anchor count is {count}, expected one: {old!r}")
    return source.replace(old, new, 1)


def apply_mutation(source: str, mutation: dict[str, Any], label: str) -> str:
    kind = mutation.get("kind")
    if kind == "insert_before":
        needle = mutation["needle"]
        return replace_once(source, needle, mutation["text"] + needle, label)
    if kind == "insert_after":
        needle = mutation["needle"]
        return replace_once(source, needle, needle + mutation["text"], label)
    if kind == "replace_once":
        return replace_once(source, mutation["old"], mutation["new"], label)
    if kind == "insert_function_start":
        name = mutation["function"]
        matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", source))
        if len(matches) != 1:
            raise RuntimeError(f"{label}: expected one function `{name}`, found {len(matches)}")
        opening = source.find("{", matches[0].end())
        if opening < 0:
            raise RuntimeError(f"{label}: no body opening brace for `{name}`")
        return source[:opening + 1] + "\n    " + mutation["text"] + "\n" + source[opening + 1:]
    raise RuntimeError(f"{label}: unsupported mutation kind {kind!r}")


def load_sources() -> dict[str, str]:
    names = (
        "promotion", "lifecycle", "owned_pointer", "free_effect", "physical_projection",
        "field_event", "event", "provenance_specs", "lib",
    )
    sources = {name: (ROOT / "src" / ("lib.rs" if name == "lib" else f"{name}.rs")).read_text()
               for name in names}
    sources.update({
        "build": (ROOT / "build.rs").read_text(),
        "extractor": (ROOT / "extract_public.py").read_text(),
        "probe_cargo": (ROOT / "Cargo.toml").read_text(),
        "production_cargo": (CHECKER.CRATE_ROOT / "Cargo.toml").read_text(),
        "public_shared": (ROOT / "src/public_shared.rs").read_text(),
    })
    return sources


def validate(control: dict[str, Any], source: str, sources: dict[str, str]) -> None:
    validator = control["validator"]
    kind = validator["kind"]
    target = control["target"]
    if kind == "module_tokens":
        CHECKER.assert_module_token_surface(target, source)
    elif kind == "effect_body":
        fn = validator["function"]
        expected = CHECKER.EXPECTED_EFFECT_BODY_HASHES[("promotion", fn)]
        CHECKER.assert_body_token_hash(source, fn, expected, f"control::{fn}")
    elif kind == "no_trust":
        CHECKER.assert_no_unreviewed_proof_attributes(source, f"control::{target}", set(validator["allowed"]))
    elif kind == "proof_state":
        CHECKER.assert_proof_state_types(source)
    elif kind == "function_surface":
        CHECKER.assert_closed_function_surface(source)
    elif kind == "trust_boundary":
        CHECKER.assert_trust_boundary(source)
    elif kind == "routes":
        CHECKER.assert_routes(source, sources["promotion"])
    elif kind == "include_routes":
        CHECKER.assert_selected_include_routes(source, ROOT / "src/promotion.rs", "promotion.rs")
    elif kind == "build_source":
        CHECKER.assert_build_script_surface(source.encode())
    elif kind == "extractor_source":
        CHECKER.assert_extractor_source_surface(source.encode())
    elif kind == "cargo_routes":
        probe = source if target == "probe_cargo" else sources["probe_cargo"]
        production = source if target == "production_cargo" else sources["production_cargo"]
        CHECKER.assert_cargo_manifest_routes(probe, production)
    else:
        raise RuntimeError(f"{control['id']}: unsupported validator kind {kind!r}")


def run_controls() -> dict[str, Any]:
    manifest = json.loads(MANIFEST_PATH.read_text())
    if manifest.get("version") != 1 or not isinstance(manifest.get("controls"), list):
        raise RuntimeError("checker control manifest has an unsupported shape")
    controls = manifest["controls"]
    ids = [c.get("id") for c in controls]
    if not ids or len(ids) != len(set(ids)) or any(not isinstance(x, str) for x in ids):
        raise RuntimeError("checker control IDs must be nonempty and unique")
    sources = load_sources()
    results = []
    for control in controls:
        label = control["id"]
        target = control["target"]
        if target not in sources:
            raise RuntimeError(f"{label}: unknown mutation target {target!r}")
        mutated = apply_mutation(sources[target], control["mutation"], label)
        try:
            validate(control, mutated, sources)
        except (CHECKER.CheckError, CHECKER.AI.CorrespondenceError) as exc:
            results.append({"id": label, "status": "rejected_as_expected", "reason": str(exc)})
        else:
            results.append({"id": label, "status": "UNEXPECTEDLY_ACCEPTED"})
    passed = sum(result["status"] == "rejected_as_expected" for result in results)
    return {
        "status": "pass" if passed == len(results) else "fail",
        "manifest": str(MANIFEST_PATH),
        "control_count": len(results),
        "rejected_as_expected": passed,
        "controls": results,
        "solver_invoked": False,
        "source_mutated_on_disk": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    result = run_controls()
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
