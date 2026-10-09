#!/usr/bin/env python3
"""Replay AM source and mapping mutations against the live correspondence rules."""
from __future__ import annotations

import argparse
import copy
import importlib.util
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER_PATH = ROOT / "check_correspondence.py"
MANIFEST_PATH = ROOT / "fixtures/checker-controls.json"
DEFAULT_RECEIPT = ROOT / "generated/checker-controls-receipt.json"


def load_module(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


CHECKER = load_module("am_correspondence_controls_target", CHECKER_PATH)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one mutation anchor, got {count}: {old!r}")
    return text.replace(old, new, 1)


def set_path(data: dict[str, Any], path: str, value: Any) -> None:
    keys = path.split(".")
    parent: Any = data
    for key in keys[:-1]:
        parent = parent[int(key)] if isinstance(parent, list) else parent[key]
    last = keys[-1]
    if isinstance(parent, list):
        parent[int(last)] = value
    else:
        parent[last] = value


def mutate_text(text: str, control: dict[str, Any]) -> str:
    kind = control["kind"]
    label = control["id"]
    if kind == "replace_once":
        return replace_once(text, control["old"], control["new"], label)
    if kind == "insert_before":
        needle = control["old"]
        return replace_once(text, needle, control["text"] + needle, label)
    if kind == "append":
        return text + control["text"]
    if kind == "move_before":
        needle = control["needle"]
        before = control["before"]
        text = replace_once(text, needle, "", label + " remove")
        return replace_once(text, before, needle + before, label + " insert")
    if kind == "move_after":
        needle = control["needle"]
        after = control["after"]
        text = replace_once(text, needle, "", label + " remove")
        return replace_once(text, after, after + needle, label + " insert")
    if kind == "swap_terminal_calls":
        child = "    bytes_child_terminal_drop(child,scope.borrow_mut(),child_receipt.borrow_mut());\n"
        root = "    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());\n"
        text = replace_once(text, child, "@@AM_CHILD@@\n", label + " child")
        text = replace_once(text, root, child, label + " root-to-child")
        return replace_once(text, "@@AM_CHILD@@\n", root, label + " child-to-root")
    raise RuntimeError(f"{label}: unsupported text mutation kind {kind!r}")


def load_inputs(positive_dir: pathlib.Path | None = None) -> dict[str, Any]:
    selected = positive_dir.resolve() if positive_dir else ROOT / "generated"
    base = (ROOT / "src/promotion.rs").read_text()
    helper = (selected / "terminal-helper.rs").read_text()
    client = (selected / "elaborated-client.rs").read_text()
    active = (selected / "active.rs").read_text()
    lib = (ROOT / "src/lib.rs").read_text()
    return {
        "base": base,
        "helper": helper,
        "client": client,
        "active": active,
        "lib": lib,
        "lifecycle": (ROOT / "src/lifecycle.rs").read_text(),
        "cargo": (ROOT / "Cargo.toml").read_text(),
        "production_cargo": (CHECKER.CRATE_ROOT / "Cargo.toml").read_text(),
        "build": (ROOT / "build.rs").read_text(),
        "extractor": (ROOT / "extract_public.py").read_text(),
        "mapping": json.loads((selected / "mapping.json").read_text()),
        "baseline_active_path": str(selected / "active.rs"),
    }


def validate(control: dict[str, Any], value: Any, sources: dict[str, Any],
             native_checker: Any, native_bundle: dict[str, Any]) -> None:
    target = control["target"]
    if target == "helper":
        helper = value
        CHECKER.assert_active_composition(sources["base"], helper, sources["client"],
                                          sources["base"] + "\n" + helper + sources["client"])
    elif target == "client":
        client = value
        CHECKER.assert_active_composition(sources["base"], sources["helper"], client,
                                          sources["base"] + "\n" + sources["helper"] + client)
    elif target == "active":
        CHECKER.assert_active_composition(sources["base"], sources["helper"], sources["client"], value)
        CHECKER.assert_promotion_module_route(sources["lib"], value)
    elif target == "lib":
        CHECKER.assert_promotion_module_route(value, sources["active"])
    elif target == "lifecycle":
        CHECKER.assert_frozen_al_sources({"lifecycle.rs": value.encode()})
    elif target == "mapping":
        CHECKER.assert_mapping_data(value, sources["base"], sources["helper"], sources["client"],
                                    sources["active"], native_bundle["capture"],
                                    native_bundle["mir_sources"]["client"], native_checker)
    elif target == "native_summary":
        CHECKER.assert_reviewed_native_production(value)
    elif target == "cargo":
        CHECKER.assert_cargo_routes(value, sources["production_cargo"])
    elif target == "build":
        CHECKER.AL.assert_build_script_surface(value.encode())
    elif target == "extractor":
        CHECKER.AL.assert_extractor_source_surface(value.encode())
    else:
        raise RuntimeError(f"{control['id']}: unsupported target {target!r}")


def run_controls(positive_dir: pathlib.Path | None = None) -> dict[str, Any]:
    manifest = json.loads(MANIFEST_PATH.read_text())
    if manifest.get("version") != 1 or not isinstance(manifest.get("controls"), list):
        raise RuntimeError("checker controls manifest shape/version is invalid")
    controls = manifest["controls"]
    ids = [control.get("id") for control in controls]
    if not ids or any(not isinstance(item, str) for item in ids) or len(ids) != len(set(ids)):
        raise RuntimeError("checker control IDs must be unique strings")

    sources = load_inputs(positive_dir)
    CHECKER.assert_frozen_al_sources()
    CHECKER.assert_promotion_module_route(sources["lib"], sources["active"])
    CHECKER.assert_active_composition(sources["base"], sources["helper"], sources["client"], sources["active"])
    native_checker = CHECKER.load_module("am_controls_native_checker", CHECKER.NATIVE_CHECKER_PATH)
    native_bundle = native_checker.load_bundle()
    native_baseline = native_checker.audit_bundle(native_bundle)
    if native_baseline.get("status") != "pass":
        raise RuntimeError("native source/MIR baseline fails before structural controls")
    CHECKER.assert_reviewed_native_production(native_baseline)
    sources["native_summary"] = native_baseline
    mapping = sources["mapping"]
    CHECKER.assert_mapping_data(mapping, sources["base"], sources["helper"], sources["client"],
                                sources["active"], native_bundle["capture"],
                                native_bundle["mir_sources"]["client"], native_checker)

    results = []
    for control in controls:
        label, target, kind = control["id"], control["target"], control["kind"]
        if target not in sources:
            raise RuntimeError(f"{label}: unknown mutation target {target!r}")
        mutated_sources = dict(sources)
        original = sources[target]
        if target in ("mapping", "native_summary"):
            mutated = copy.deepcopy(original)
            if kind == "mapping_set":
                set_path(mutated, control["path"], control["value"])
            elif kind == "mapping_pop":
                rows = mutated[control["path"]]
                if not isinstance(rows, list) or not rows:
                    raise RuntimeError(f"{label}: mapping path is not a nonempty list")
                rows.pop()
            elif kind == "mapping_duplicate":
                rows = mutated[control["path"]]
                if not isinstance(rows, list) or not rows:
                    raise RuntimeError(f"{label}: mapping path is not a nonempty list")
                rows.append(copy.deepcopy(rows[0]))
            else:
                raise RuntimeError(f"{label}: unsupported mapping mutation {kind!r}")
        else:
            mutated = mutate_text(original, control)
        try:
            validate(control, mutated, mutated_sources, native_checker, native_bundle)
        except (CHECKER.CheckError, CHECKER.AL.CheckError,
                CHECKER.AI.CorrespondenceError, native_checker.AuditError) as exc:
            results.append({"id": label, "status": "rejected_as_expected", "reason": str(exc)})
        else:
            results.append({"id": label, "status": "UNEXPECTEDLY_ACCEPTED"})
    rejected = sum(row["status"] == "rejected_as_expected" for row in results)
    return {
        "status": "pass" if rejected == len(results) else "fail",
        "manifest": str(MANIFEST_PATH),
        "baseline_active_source": sources["baseline_active_path"],
        "baseline_correspondence_status": "pass",
        "control_count": len(results),
        "rejected_as_expected": rejected,
        "controls": results,
        "source_mutated_on_disk": False,
        "cargo_or_rust_build_invoked": False,
        "solver_invoked": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_RECEIPT)
    parser.add_argument("--positive-dir", type=pathlib.Path,
                        help="use an archived positive generated source set while generator output is on a negative feature")
    args = parser.parse_args()
    result = run_controls(args.positive_dir)
    rendered = json.dumps(result, indent=2) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
