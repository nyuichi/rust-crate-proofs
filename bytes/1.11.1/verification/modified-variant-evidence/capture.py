#!/usr/bin/env python3
"""Capture and audit an already-completed modified-variant verifier run.

This utility is read-only with respect to the crate, verifier output, and logs.
It never invokes Cargo, Creusot, Why3, or a prover. Each invocation writes to a
new run directory and refuses to replace an existing archive.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import re
import sys
import tarfile
from pathlib import Path
from typing import Any, NoReturn


SCRIPT = Path(__file__).resolve()
CRATE = SCRIPT.parents[2]
DEFAULT_OUTPUT_ROOT = SCRIPT.parent / "runs"
WRAPPER = Path("scripts/verify-verified-bytes.sh")
TOOL_MANIFEST = Path("verification/CLOUD_RUNTIME_2026-10-05.json")
EFFECTIVE_WHY3 = Path(".verified-proof-config/creusot/why3.conf")

# This is the selected modified production source graph: the normal library
# entry, the verified entry and its canonical physical ownership dependencies.
FIXED_SOURCE_FILES = (
    Path("Cargo.toml"),
    Path("Cargo.lock"),
    Path("src/lib.rs"),
    Path("src/verified_ownership.rs"),
    Path("src/allocation_ops.rs"),
    Path("src/provenance_specs.rs"),
    Path("src/byte_codec_ops.rs"),
    Path("src/byte_codec_wide_ops.rs"),
    Path("src/slice_ops.rs"),
    Path("src/slice_read_ops.rs"),
    Path("src/slice_wide_read_ops.rs"),
    Path("src/endian_ops.rs"),
    Path("src/signed_wide_ops.rs"),
    Path("src/variable_read_ops.rs"),
    Path("src/ownership_proof/bound_ptr.rs"),
    Path("src/ownership_proof/owned_region.rs"),
    Path("src/ownership_proof/raw_vec.rs"),
    Path("src/ownership_proof/frozen_region.rs"),
    Path("verification/MODIFIED_VARIANT_SPEC.md"),
    Path("verification/MODIFIED_VARIANT_ADMISSION.md"),
)

def feature_configuration(features: str, native_features: str | None, target: str | None) -> dict[str, Any]:
    proof_features = features.split(",")
    native_features_list = (native_features or features).split(",")
    if "verified" not in proof_features or "verified" not in native_features_list:
        fail("both configurations must select verified")
    if any(not value or any(ch.isspace() for ch in value) for value in proof_features + native_features_list):
        fail("feature lists must contain nonempty names without whitespace")
    target_args = ["--target", target] if target else []
    build_std_args = ["-Zbuild-std=core,alloc"] if target == "msp430-none-elf" else []
    cargo_args = ["--no-default-features", "--features", features, *target_args, *build_std_args]
    return {
        "cargo_target": "--lib",
        "target": target or "host",
        "no_default_features": True,
        "features": proof_features,
        "native_features": native_features_list,
        "native_proof_requested_feature_difference": proof_features != native_features_list,
        "native_proof_feature_difference": True,
        "implicit_cargo_creusot_features": ["creusot-std/creusot", "creusot-std/nightly"],
        "effective_proof_cargo_feature_arguments": [*cargo_args, "-F", "creusot-std/creusot creusot-std/nightly"],
        "cargo_feature_arguments": cargo_args,
        "native_cargo_feature_arguments": ["--no-default-features", "--features", native_features or features, *target_args],
        "creusot_translation_command": ["cargo", "creusot", "--only=coma", "--", "--locked", "--lib", *cargo_args],
        "creusot_proof_command": ["cargo", "creusot", "--only=prove", "--why3find-arg=-j", "--why3find-arg=1"],
        "correspondence_note": (
            "Requested feature equality does not mean dependency graph equality: pinned cargo-creusot automatically enables creusot-std/creusot and creusot-std/nightly. Capture the effective feature graph and separately audit source/type/configuration correspondence; this record alone proves none."
        ),
    }


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fail(message: str) -> NoReturn:
    raise ValueError(message)


def null_paths(value: Any, path: tuple[str, ...] = ()):
    """Yield paths of null leaves in a proof-result subtree."""
    if value is None:
        yield list(path)
    elif isinstance(value, dict):
        for key, item in value.items():
            yield from null_paths(item, path + (str(key),))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            yield from null_paths(item, path + (str(index),))


def json_bytes(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def read_member(relative: Path) -> bytes:
    path = CRATE / relative
    if not path.is_file():
        fail(f"required capture input is missing: {relative}")
    return path.read_bytes()


def add(members: dict[str, bytes], name: str, data: bytes) -> None:
    if name in members:
        fail(f"duplicate archive member: {name}")
    members[name] = data


def safe_component(value: str, what: str) -> str:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", value):
        fail(f"invalid {what} {value!r}; use letters, digits, dot, underscore, or hyphen")
    if value in {".", ".."}:
        fail(f"invalid {what}: {value!r}")
    return value


def parse_log(spec: str) -> tuple[str, Path]:
    if "=" not in spec:
        fail(f"--log must be NAME=PATH, got {spec!r}")
    name, raw_path = spec.split("=", 1)
    name = safe_component(name, "log name")
    path = Path(raw_path).expanduser().resolve()
    if not path.is_file():
        fail(f"log file does not exist: {path}")
    return name, path


def source_files() -> list[Path]:
    paths = set(FIXED_SOURCE_FILES)
    verified_dir = CRATE / "src/verified"
    if not verified_dir.is_dir():
        fail("src/verified is missing; no modified-variant source can be captured")
    paths.update(
        path.relative_to(CRATE)
        for path in verified_dir.rglob("*")
        if path.is_file()
    )
    return sorted(paths, key=lambda path: path.as_posix())


def source_refs(coma: bytes, crate_sources: set[str]) -> list[dict[str, Any]]:
    """Match Creusot source spans to the exact captured crate source files."""
    text = coma.decode("utf-8", errors="replace")
    refs: set[tuple[str, int | None]] = set()
    for match in re.finditer(r'"([^"\n]+\.rs)"(?:\s+(\d+))?', text):
        raw_path = Path(match.group(1))
        try:
            absolute = (raw_path if raw_path.is_absolute() else CRATE / raw_path).resolve()
            relative = absolute.relative_to(CRATE).as_posix()
        except (OSError, ValueError):
            continue
        if relative in crate_sources:
            line = int(match.group(2)) if match.group(2) else None
            refs.add((relative, line))
        elif absolute.is_relative_to(CRATE):
            fail(f"Coma references crate source absent from captured source graph: {relative}")
    return [
        {"source": path, "line": line}
        for path, line in sorted(refs, key=lambda item: (item[0], item[1] or 0))
    ]


def verify_tool_manifest(data: bytes) -> dict[str, Any]:
    manifest = json.loads(data)
    checks: dict[str, Any] = {}
    all_match = True
    for name, item in manifest.get("binaries", {}).items():
        raw_path = Path(item["path"])
        if not raw_path.is_absolute():
            raw_path = CRATE / raw_path
        if not raw_path.is_file():
            checks[name] = {"path": str(raw_path), "status": "missing"}
            all_match = False
            continue
        actual = sha256(raw_path.read_bytes())
        expected = item.get("sha256")
        matches = actual == expected
        checks[name] = {
            "path": str(raw_path),
            "expected_sha256": expected,
            "actual_sha256": actual,
            "matches_manifest": matches,
        }
        all_match = all_match and matches
    if not manifest.get("binaries"):
        fail("installation manifest does not list pinned tool binaries")
    return {"manifest_sha256": sha256(data), "binaries_match": all_match, "binaries": checks}


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Archive an existing modified-variant proof run. This is a capture/audit "
            "tool only; it does not execute a proof. Full-coverage claims are not accepted."
        )
    )
    parser.add_argument("--kind", required=True, choices=("positive", "failure", "negative"))
    parser.add_argument("--name", required=True, help="unique run label within the selected kind")
    parser.add_argument(
        "--run-dir",
        required=True,
        help="Creusot verif output directory (may be absent for a frontend failure capture)",
    )
    parser.add_argument(
        "--exit-code",
        required=True,
        type=int,
        help="observed exit code of the completed command being captured",
    )
    parser.add_argument(
        "--log",
        action="append",
        required=True,
        metavar="NAME=PATH",
        help="run log to archive; repeat for translation/proof/native/diagnostic logs",
    )
    parser.add_argument(
        "--output-root",
        default=str(DEFAULT_OUTPUT_ROOT),
        help=f"evidence root (default: {DEFAULT_OUTPUT_ROOT})",
    )
    parser.add_argument("--features", default="verified,std", help="exact proof feature list")
    parser.add_argument("--native-features", help="exact native feature list if different from proof")
    parser.add_argument("--target", help="explicit Cargo target, otherwise host")
    parser.add_argument("--std-source-root", help="actual resolved creusot-std package, required for a local path override")
    args = parser.parse_args()
    selected_configuration = feature_configuration(args.features, args.native_features, args.target)

    run_name = safe_component(args.name, "run name")
    logs = [parse_log(spec) for spec in args.log]
    if len({name for name, _ in logs}) != len(logs):
        fail("log names must be unique within a run")

    run_dir = Path(args.run_dir).expanduser().resolve()
    if not run_dir.is_dir() and args.kind != "failure":
        fail(f"run directory does not exist: {run_dir}")
    coma_paths = sorted(run_dir.rglob("*.coma")) if run_dir.is_dir() else []
    proof_paths = sorted(path for path in run_dir.rglob("proof.json")) if run_dir.is_dir() else []
    if args.kind == "positive" and (not coma_paths or not proof_paths):
        fail("positive capture requires at least one exact .coma and proof.json leaf")
    if args.kind == "negative" and (not coma_paths or not proof_paths):
        fail("negative capture requires the translated .coma and proof.json leaves")

    members: dict[str, bytes] = {}
    source_paths = source_files()
    source_member_names: dict[str, str] = {}
    for relative in source_paths:
        member_name = "production-source/" + relative.as_posix()
        add(members, member_name, read_member(relative))
        source_member_names[relative.as_posix()] = member_name

    wrapper_data = read_member(WRAPPER)
    tool_manifest_data = read_member(TOOL_MANIFEST)
    why3_config_data = read_member(EFFECTIVE_WHY3)
    add(members, "configuration/verify-verified-bytes.sh", wrapper_data)
    add(members, "configuration/CLOUD_RUNTIME_2026-10-05.json", tool_manifest_data)
    add(members, "configuration/effective-why3.conf", why3_config_data)
    add(members, "capture/capture.py", SCRIPT.read_bytes())
    optional_tools = [
        Path("why3find.json"),
        Path("verify-all.bash"),
        Path("scripts/verify-all.bash"),
        Path("verification/verify-all.bash"),
    ]
    for relative in optional_tools:
        if (CRATE / relative).is_file():
            add(members, "configuration/" + relative.as_posix(), read_member(relative))

    tool_paths = json.loads(tool_manifest_data)["binaries"]
    tool_base = Path(tool_paths["cargo-creusot"]["path"]).parents[2]
    std_root = (Path(args.std_source_root).expanduser().resolve() if args.std_source_root
                else tool_base / "creusot-source/creusot-std")
    local_config = CRATE / ".cargo/config.toml"
    if local_config.is_file():
        add(members, "configuration/cargo-config.toml", local_config.read_bytes())
        if not args.std_source_root:
            fail("local Cargo configuration requires --std-source-root to avoid capturing stock contracts")
    for name, path in [
        ("cargo-creusot-feature-flags.rs", tool_base / "creusot-source/cargo-creusot/src/main.rs"),
        ("creusot-std-features.toml", std_root / "Cargo.toml"),
        ("creusot-std-vec-contracts.rs", std_root / "src/std/vec.rs"),
    ]:
        if path.is_file():
            add(members, "configuration/" + name, path.read_bytes())
    if args.std_source_root:
        if not (std_root / "Cargo.toml").is_file():
            fail("actual Std package is missing")
        std_hashes = {}
        for path in sorted(std_root.rglob("*")):
            if path.is_file() and (path.suffix in {".rs", ".toml"}):
                relative = path.relative_to(std_root).as_posix()
                data = path.read_bytes()
                add(members, "configuration/actual-creusot-std/" + relative, data)
                std_hashes[relative] = {"sha256": sha256(data), "bytes": len(data)}
        add(members, "configuration/actual-creusot-std-source.json", json_bytes({
            "resolved_package_root": str(std_root), "files": std_hashes,
            "note": "Actual dependency source snapshot; library contracts are trusted, not body proofs."
        }))
        for relative in [Path("scripts/prepare-verified-std.py"),
                         *sorted((CRATE / "verification/std-support").glob("*"))]:
            if relative.is_absolute():
                relative = relative.relative_to(CRATE)
            if (CRATE / relative).is_file():
                add(members, "configuration/" + relative.as_posix(), read_member(relative))

    before_path = CRATE / ".verified-proof-config/source-before.json"
    if before_path.is_file():
        before = json.loads(before_path.read_text())
        # Only enforce fingerprints emitted by the new wrapper for this run.
        if args.std_source_root:
            for relative, expected in before.items():
                if sha256(read_member(Path(relative))) != expected:
                    fail(f"source differs from pre-run fingerprint: {relative}")
            add(members, "configuration/source-before.json", before_path.read_bytes())
    tool_check = verify_tool_manifest(tool_manifest_data)
    if args.kind in {"positive", "negative"} and not tool_check["binaries_match"]:
        fail("positive and negative proof captures require tool binaries matching the installation manifest")

    log_records: list[dict[str, Any]] = []
    complete_proof_file_counts: list[int] = []
    for name, path in logs:
        data = path.read_bytes()
        plain_log = re.sub(r"\x1b\[[0-9;]*m", "", data.decode("utf-8", errors="replace"))
        complete_proof_file_counts.extend(int(count) for count in re.findall(r"\bProved\s+\((\d+) files?\)", plain_log))
        member_name = f"logs/{name}.log"
        add(members, member_name, data)
        log_records.append(
            {
                "name": name,
                "archive_member": member_name,
                "captured_from": str(path),
                "sha256": sha256(data),
                "bytes": len(data),
            }
        )

    source_name_set = set(source_member_names)
    coma_records: list[dict[str, Any]] = []
    for path in coma_paths:
        relative = path.relative_to(run_dir).as_posix()
        member_name = f"proof-output/{relative}"
        data = path.read_bytes()
        add(members, member_name, data)
        refs = source_refs(data, source_name_set)
        if not refs:
            fail(f"Coma file has no source span mapped to this production source snapshot: {relative}")
        coma_records.append(
            {
                "archive_member": member_name,
                "sha256": sha256(data),
                "bytes": len(data),
                "production_source_refs": refs,
            }
        )

    proof_records: list[dict[str, Any]] = []
    failed_proofs: dict[str, list[list[str]]] = {}
    null_leaf_count = 0
    for path in proof_paths:
        relative = path.relative_to(run_dir).as_posix()
        member_name = f"proof-output/{relative}"
        data = path.read_bytes()
        add(members, member_name, data)
        document = json.loads(data)
        if not isinstance(document, dict) or "proofs" not in document:
            fail(f"proof JSON has no top-level proofs object: {relative}")
        leaves = list(null_paths(document["proofs"]))
        null_leaf_count += len(leaves)
        if leaves:
            failed_proofs[member_name] = leaves
        proof_records.append(
            {
                "archive_member": member_name,
                "sha256": sha256(data),
                "bytes": len(data),
                "null_leaves_in_proofs": leaves,
            }
        )

    if args.kind == "positive" and null_leaf_count:
        fail(f"positive capture contains {null_leaf_count} null proof-result leaves")
    if args.kind == "negative" and not null_leaf_count:
        fail("negative capture has no null proof-result leaves to substantiate rejection")

    json_to_coma: dict[str, str | None] = {}
    for item in proof_records:
        proof_relative = Path(item["archive_member"]).relative_to("proof-output")
        # A proof.json sits in `<function>/proof.json`; the matching Coma file is
        # usually `<function>.coma` beside that directory.
        coma_candidate = (
            Path("proof-output")
            / proof_relative.parent.parent
            / f"{proof_relative.parent.name}.coma"
        ).as_posix()
        json_to_coma[item["archive_member"]] = coma_candidate if coma_candidate in {x["archive_member"] for x in coma_records} else None

    if args.kind == "positive":
        if args.exit_code != 0:
            fail("positive capture requires an observed successful command exit code")
        if len(coma_records) not in complete_proof_file_counts:
            fail("positive capture requires the completed engine Proved(N files) log matching all captured Coma files")
        coma_names = {item["archive_member"] for item in coma_records}
        mapped_coma = list(json_to_coma.values())
        if None in mapped_coma or len(mapped_coma) != len(set(mapped_coma)):
            fail("positive capture requires exactly one matching Coma file per proof result")
        missing = sorted(coma_names - set(mapped_coma))
        if missing:
            fail(f"positive capture is incomplete: {len(missing)} Coma files have no proof result: {missing[:5]}")

    correspondence = {
        "crate_entry": "src/lib.rs",
        "selected_module": "src/verified/mod.rs",
        "source_files": {
            relative: {
                "archive_member": source_member_names[relative],
                "sha256": sha256(members[source_member_names[relative]]),
                "bytes": len(members[source_member_names[relative]]),
            }
            for relative in sorted(source_member_names)
        },
        "coma_sources": coma_records,
        "proof_json_to_coma": json_to_coma,
        "correspondence_method": (
            "Creusot source spans in each exact archived .coma are mapped to the exact "
            "co-captured production source file snapshots and hashes. This receipt records "
            "the captured source/output pair; the wrapper does not emit a pre-run source hash. "
            "If no .coma was emitted, source snapshots are preserved but no body correspondence "
            "is claimed."
        ),
    }
    add(members, "audit/source-correspondence.json", json_bytes(correspondence))

    if args.kind == "positive":
        run_disposition = "captured-positive-proof-results"
    elif args.kind == "negative":
        run_disposition = "captured-expected-rejection-results"
    else:
        run_disposition = "captured-failure-diagnostics-or-partial-results"

    record = {
        "evidence_kind": args.kind,
        "run_name": run_name,
        "disposition": run_disposition,
        "claim_scope": (
            "Selected modified production entry under the recorded feature configuration; "
            "the result is limited to the exact captured functions and proof leaves."
        ),
        "full_coverage": False,
        "full_coverage_note": (
            "This capture does not report full crate/API coverage or architecture admission. "
            "Only the archived Coma and proof JSON leaves support a proof-result claim."
        ),
        "feature_configuration": selected_configuration,
        "toolchain": tool_check,
        "run_directory": str(run_dir),
        "observed_exit_code": args.exit_code,
        "completed_engine_file_counts_in_logs": complete_proof_file_counts,
        "logs": log_records,
        "counts": {
            "coma_files": len(coma_records),
            "proof_json_files": len(proof_records),
            "null_proof_result_leaves": null_leaf_count,
            "proof_json_files_with_null_leaves": len(failed_proofs),
        },
        "failed_proof_json": failed_proofs,
        "toolchain_matches_manifest": tool_check["binaries_match"],
        "source_correspondence_member": "audit/source-correspondence.json",
        "members_before_run_record": {
            name: {"sha256": sha256(data), "bytes": len(data)}
            for name, data in sorted(members.items())
        },
    }
    add(members, "audit/run-record.json", json_bytes(record))
    # The run record cannot hash itself. The outer receipt covers every tar
    # member, including the exact run-record bytes.
    member_manifest = {
        name: {"sha256": sha256(data), "bytes": len(data)}
        for name, data in sorted(members.items())
    }

    output_root = Path(args.output_root).expanduser().resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    output_dir = output_root / args.kind / run_name
    output_dir.mkdir(parents=True, exist_ok=False)
    archive_path = output_dir / "evidence.tar.gz"
    with archive_path.open("wb") as raw_archive:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw_archive, mtime=0, compresslevel=9) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as archive:
                for name, data in sorted(members.items()):
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    info.mode = 0o644
                    info.uid = 0
                    info.gid = 0
                    info.uname = "root"
                    info.gname = "root"
                    info.mtime = 0
                    archive.addfile(info, io.BytesIO(data))

    archive_data = archive_path.read_bytes()
    receipt = {
        "archive": {
            "file": archive_path.name,
            "sha256": sha256(archive_data),
            "bytes": len(archive_data),
        },
        "members_checked": len(member_manifest),
        "members": member_manifest,
        "counts": record["counts"],
        "full_coverage": False,
        "all_member_hashes_match": True,
    }
    with tarfile.open(archive_path, "r:gz") as archive:
        archive_names = {item.name for item in archive.getmembers() if item.isfile()}
        if archive_names != set(member_manifest):
            fail("archive member set differs from the outer receipt")
        for name, expected in member_manifest.items():
            extracted = archive.extractfile(name)
            if extracted is None:
                fail(f"cannot read archived member {name}")
            data = extracted.read()
            if len(data) != expected["bytes"] or sha256(data) != expected["sha256"]:
                fail(f"archive integrity check failed for {name}")

    receipt_path = output_dir / "receipt.json"
    receipt_path.write_bytes(json_bytes(receipt))
    print(
        json.dumps(
            {
                "kind": args.kind,
                "run": run_name,
                "archive": str(archive_path),
                "sha256": receipt["archive"]["sha256"],
                "coma_files": record["counts"]["coma_files"],
                "proof_json_files": record["counts"]["proof_json_files"],
                "null_proof_result_leaves": null_leaf_count,
                "full_coverage": False,
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError, tarfile.TarError) as error:
        print(f"capture: {error}", file=sys.stderr)
        raise SystemExit(2)
