#!/usr/bin/env python3
"""Collect hashes and VC counts for isolated component proof runs.

This records component evidence only. It deliberately does not assign runtime
integration status or calculate a completion percentage.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PROBES = ROOT / "verification" / "probes"
ARTIFACTS = ROOT / "verification" / "artifacts"
LOGS = ARTIFACTS / "logs"
SNAPSHOTS = ARTIFACTS / "component-evidence"
SUMMARY = ARTIFACTS / "component-evidence.json"

TARGETS = (
    "helpers",
    "storage",
    "deallocation",
    "bounded-ops",
    "slice-ops",
    "cursor-ops",
    "byte-codecs",
    "comparison-ops",
    "chain-ops",
    "capacity-ops",
    "slice-read-ops",
    "slice-wide-read-ops",
    "variable-read-ops",
    "initialized-storage",
    "uninit-ops",
    "wide-codecs",
    "endian-ops",
    "signed-wide-ops",
    "region-permissions",
    "provenance-ops",
    "ownership-frontier",
)

FOUNDATIONS = {"helpers", "storage", "deallocation", "region-permissions", "provenance-ops", "ownership-frontier"}
DIRECT_PATH_RE = re.compile(r"#\s*\[\s*path\s*=\s*(\"(?:\\.|[^\"])*\")\s*\]", re.DOTALL)
PROVED_RE = re.compile(r"^Proved \((?:(\d+) files?|((?:verif/)[^ )]+\.coma))\) ✔$", re.MULTILINE)
RESULT_RE = re.compile(
    r"^component-result: (PASS|FAIL) target=([^\s]+) exit_code=(\d+) proved_files=(\d+)(?:\s+.*)?$",
    re.MULTILINE,
)


class EvidenceError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def repo_relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError as error:
        raise EvidenceError(f"evidence path is outside repository: {path}") from error


def write_json_atomic(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def component_scope(target: str) -> str:
    return "foundation" if target in FOUNDATIONS else "connected_helpers"


def current_inputs(target: str) -> dict[str, dict[str, str]]:
    probe = PROBES / target
    required = (
        probe / "Cargo.toml",
        probe / "Cargo.lock",
        probe / "why3find.json",
        probe / "src" / "lib.rs",
    )
    missing = [path for path in required if not path.is_file()]
    if missing:
        raise EvidenceError(
            f"{target}: required probe files missing: "
            + ", ".join(repo_relative(path) for path in missing)
        )

    probe_sources = sorted(path for path in (probe / "src").rglob("*.rs") if path.is_file())
    if not probe_sources:
        raise EvidenceError(f"{target}: probe has no Rust source files")

    source_hashes: dict[str, str] = {}
    direct_path_hashes: dict[str, str] = {}
    for source in probe_sources:
        source_hashes[repo_relative(source)] = sha256(source)
        source_text = source.read_text(encoding="utf-8")
        for match in DIRECT_PATH_RE.finditer(source_text):
            try:
                relative = ast.literal_eval(match.group(1))
            except (SyntaxError, ValueError) as error:
                raise EvidenceError(f"{repo_relative(source)}: invalid #[path] string") from error
            if not isinstance(relative, str):
                raise EvidenceError(f"{repo_relative(source)}: #[path] is not a string")
            referenced = (source.parent / relative).resolve()
            if not referenced.is_file():
                raise EvidenceError(
                    f"{repo_relative(source)}: direct #[path] source is missing: {relative}"
                )
            direct_path_hashes[repo_relative(referenced)] = sha256(referenced)

    config_paths = (
        probe / "Cargo.toml",
        probe / "Cargo.lock",
        probe / "why3find.json",
        ROOT / "scripts" / "verify-bytes.sh",
        ROOT / "scripts" / "verify-components.sh",
        ROOT / "scripts" / "collect-component-evidence.py",
        ARTIFACTS / "tool-versions.txt",
        ARTIFACTS / "installation-manifest.json",
    )
    config_hashes: dict[str, str] = {}
    for path in config_paths:
        if not path.is_file():
            raise EvidenceError(f"{target}: proof configuration is missing: {repo_relative(path)}")
        config_hashes[repo_relative(path)] = sha256(path)

    return {
        "probe_sources": dict(sorted(source_hashes.items())),
        "direct_path_sources": dict(sorted(direct_path_hashes.items())),
        "probe_and_tool_config": dict(sorted(config_hashes.items())),
    }


def parse_log(target: str, log_path: Path) -> tuple[int, dict[str, Any]]:
    if not log_path.is_file():
        raise EvidenceError(f"{target}: proof log is missing: {repo_relative(log_path)}")
    text = log_path.read_text(encoding="utf-8", errors="replace")
    summaries = list(PROVED_RE.finditer(text))
    if not summaries:
        raise EvidenceError(f"{target}: proof log has no recognized Proved summary")
    summary = summaries[-1]
    if summary.group(1) is not None:
        proved_files = int(summary.group(1))
    else:
        summary_path = Path(summary.group(2))
        if summary_path.is_absolute() or ".." in summary_path.parts or summary_path.parts[0] != "verif":
            raise EvidenceError(f"{target}: Proved summary path escapes the probe output")
        coma_path = PROBES / target / summary_path
        paired_json = coma_path.parent / coma_path.stem / "proof.json"
        if not coma_path.is_file() or not paired_json.is_file():
            raise EvidenceError(f"{target}: singleton Proved path has no paired .coma/proof.json")
        proved_files = 1
    if proved_files <= 0:
        raise EvidenceError(f"{target}: proof log reports zero proved files")
    if any(marker in text for marker in ("Compilation failed", "unproved file", " ✘")):
        raise EvidenceError(f"{target}: proof log contains a failure marker")

    results = list(RESULT_RE.finditer(text))
    if results:
        result = results[-1]
        if result.group(1) != "PASS" or result.group(2) != target or int(result.group(3)) != 0:
            raise EvidenceError(f"{target}: proof log has a failing or mismatched component result")
        if int(result.group(4)) != proved_files:
            raise EvidenceError(f"{target}: result marker disagrees with the Proved summary")
    return proved_files, {"path": repo_relative(log_path), "sha256": sha256(log_path)}


def proof_files(verif_root: Path) -> tuple[list[Path], list[Path]]:
    if not verif_root.is_dir():
        raise EvidenceError(f"proof output directory is missing: {repo_relative(verif_root)}")
    coma_files = sorted(path for path in verif_root.rglob("*.coma") if path.is_file())
    json_files = sorted(path for path in verif_root.rglob("proof.json") if path.is_file())
    if not coma_files:
        raise EvidenceError(f"no .coma VCs found under {repo_relative(verif_root)}")
    if len(coma_files) != len(json_files):
        raise EvidenceError(
            f"proof artifact mismatch under {repo_relative(verif_root)}: "
            f"{len(coma_files)} .coma files, {len(json_files)} proof.json files"
        )

    coma_set = {path.resolve() for path in coma_files}
    json_set = {path.resolve() for path in json_files}
    for coma in coma_files:
        expected = (coma.parent / coma.stem / "proof.json").resolve()
        if expected not in json_set:
            raise EvidenceError(
                f"missing proof.json paired with {repo_relative(coma)}"
            )
    for proof_json in json_files:
        coma = proof_json.parent.parent / f"{proof_json.parent.name}.coma"
        if coma.resolve() not in coma_set:
            raise EvidenceError(
                f"proof.json has no paired .coma: {repo_relative(proof_json)}"
            )
    return coma_files, json_files


def read_vc_count(proof_json: Path) -> int:
    try:
        content = json.loads(proof_json.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"cannot parse {repo_relative(proof_json)}: {error}") from error
    proofs = content.get("proofs", {}).get("Coma", {})
    if not isinstance(proofs, dict):
        raise EvidenceError(f"unexpected Coma proof shape in {repo_relative(proof_json)}")
    return len(proofs)


def collect_target(target: str) -> dict[str, Any]:
    probe = PROBES / target
    log_path = LOGS / f"{target}-proof.log"
    proved_files, log_hash = parse_log(target, log_path)
    inputs = current_inputs(target)
    coma_files, json_files = proof_files(probe / "verif")
    if len(coma_files) != proved_files:
        raise EvidenceError(
            f"{target}: log reports {proved_files} proved files but current output has "
            f"{len(coma_files)} .coma files"
        )

    artifact_rows: list[dict[str, str]] = []
    vc_count = 0
    for source in sorted(coma_files + json_files):
        relative = source.relative_to(probe / "verif")
        artifact_rows.append({
            "path": relative.as_posix(),
            "kind": "coma" if source.suffix == ".coma" else "proof_json",
            "sha256": sha256(source),
        })
        if source.name == "proof.json":
            vc_count += read_vc_count(source)

    snapshot = SNAPSHOTS / target
    SNAPSHOTS.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{target}.", dir=SNAPSHOTS))
    backup: Path | None = None
    try:
        for source in sorted(coma_files + json_files):
            relative = source.relative_to(probe / "verif")
            destination = stage / "verif" / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)

        component = {
            "schema_version": 1,
            "target": target,
            "evidence_scope": component_scope(target),
            "component_status": "isolated_component",
            "integrated_runtime": False,
            "counts": {
                "proved_files": proved_files,
                "verification_conditions": vc_count,
                "coma_files": len(coma_files),
                "proof_json_files": len(json_files),
            },
            "inputs": inputs,
            "proof_log": log_hash,
            "proof_artifacts": artifact_rows,
        }
        write_json_atomic(stage / "component.json", component)

        if snapshot.exists():
            backup = SNAPSHOTS / f".{target}.previous.{os.getpid()}"
            if backup.exists():
                shutil.rmtree(backup)
            os.replace(snapshot, backup)
        os.replace(stage, snapshot)
        if backup is not None:
            shutil.rmtree(backup)
    except Exception:
        if backup is not None and backup.exists():
            if snapshot.exists():
                shutil.rmtree(snapshot)
            os.replace(backup, snapshot)
        raise
    finally:
        if stage.exists():
            shutil.rmtree(stage)

    component["snapshot"] = repo_relative(snapshot)
    write_json_atomic(snapshot / "component.json", component)
    return component


def validate_snapshot(target: str) -> dict[str, Any]:
    snapshot = SNAPSHOTS / target
    metadata_path = snapshot / "component.json"
    if not metadata_path.is_file():
        raise EvidenceError(f"{target}: evidence snapshot is missing: {repo_relative(metadata_path)}")
    try:
        component = json.loads(metadata_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceError(f"{target}: cannot read component snapshot: {error}") from error
    if component.get("component_status") != "isolated_component":
        raise EvidenceError(f"{target}: snapshot has an invalid component status")
    if component.get("integrated_runtime") is not False:
        raise EvidenceError(f"{target}: snapshot must explicitly mark runtime integration false")
    if component.get("target") != target:
        raise EvidenceError(f"{target}: snapshot target mismatch")
    if component.get("evidence_scope") != component_scope(target):
        raise EvidenceError(f"{target}: snapshot evidence scope mismatch")

    current = current_inputs(target)
    if current != component.get("inputs"):
        raise EvidenceError(f"{target}: source/configuration hashes changed after proof collection")
    proved_files, log_hash = parse_log(target, LOGS / f"{target}-proof.log")
    if log_hash != component.get("proof_log") or proved_files != component.get("counts", {}).get("proved_files"):
        raise EvidenceError(f"{target}: proof log changed after evidence collection")

    artifact_rows = component.get("proof_artifacts")
    if not isinstance(artifact_rows, list):
        raise EvidenceError(f"{target}: snapshot artifact list is missing")
    seen: set[str] = set()
    vc_count = 0
    for row in artifact_rows:
        relative = row.get("path")
        if not isinstance(relative, str) or relative in seen:
            raise EvidenceError(f"{target}: invalid or duplicate snapshot artifact path")
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise EvidenceError(f"{target}: snapshot artifact path escapes its evidence directory")
        seen.add(relative)
        artifact_path = snapshot / "verif" / relative_path
        if not artifact_path.is_file() or sha256(artifact_path) != row.get("sha256"):
            raise EvidenceError(f"{target}: missing or changed snapshot artifact {relative}")
        if row.get("kind") == "proof_json":
            vc_count += read_vc_count(artifact_path)

    counts = component.get("counts", {})
    if counts.get("verification_conditions") != vc_count:
        raise EvidenceError(f"{target}: verification-condition count changed")
    coma_count = sum(row.get("kind") == "coma" for row in artifact_rows)
    json_count = sum(row.get("kind") == "proof_json" for row in artifact_rows)
    if counts.get("coma_files") != coma_count or counts.get("proof_json_files") != json_count:
        raise EvidenceError(f"{target}: artifact counts do not match the snapshot")
    return component


def collect_all(targets: tuple[str, ...]) -> dict[str, Any]:
    components = [validate_snapshot(target) for target in targets]
    # Totals count only the isolated component outputs represented here.
    totals = {
        "components": len(components),
        "proved_files": sum(row["counts"]["proved_files"] for row in components),
        "verification_conditions": sum(row["counts"]["verification_conditions"] for row in components),
        "coma_files": sum(row["counts"]["coma_files"] for row in components),
        "proof_json_files": sum(row["counts"]["proof_json_files"] for row in components),
    }
    summary = {
        "schema_version": 1,
        "scope": "sequential default-configuration isolated component proofs for the explicitly listed set",
        "integrated_runtime": False,
        "component_order": list(targets),
        "components": components,
        "isolated_component_totals": totals,
    }
    if targets == TARGETS:
        summary_path = SUMMARY
    else:
        selection = hashlib.sha256("\n".join(targets).encode()).hexdigest()[:16]
        summary_path = ARTIFACTS / "component-evidence-subsets" / f"{selection}.json"
    write_json_atomic(summary_path, summary)
    summary["summary_path"] = repo_relative(summary_path)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=TARGETS, help="snapshot and report one successful target")
    parser.add_argument(
        "--targets",
        nargs="+",
        choices=TARGETS,
        help="aggregate exactly this selected set; without it, require the full configured set",
    )
    args = parser.parse_args()
    if args.target and args.targets:
        parser.error("use either --target or --targets, not both")
    if args.targets:
        if len(set(args.targets)) != len(args.targets):
            parser.error("--targets cannot contain duplicates")
        selected = tuple(target for target in TARGETS if target in set(args.targets))
    else:
        selected = TARGETS
    try:
        if args.target:
            component = collect_target(args.target)
            print(json.dumps({
                "target": component["target"],
                "component_status": component["component_status"],
                "integrated_runtime": component["integrated_runtime"],
                "counts": component["counts"],
                "snapshot": component["snapshot"],
            }, sort_keys=True))
        else:
            summary = collect_all(selected)
            print(json.dumps({
                "component_status": (
                    "all_configured_isolated_components_collected"
                    if selected == TARGETS
                    else "selected_isolated_components_collected"
                ),
                "integrated_runtime": summary["integrated_runtime"],
                "isolated_component_totals": summary["isolated_component_totals"],
                "summary": summary["summary_path"],
            }, sort_keys=True))
    except EvidenceError as error:
        print(f"component evidence error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
