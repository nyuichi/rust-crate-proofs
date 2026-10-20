#!/usr/bin/env python3
"""Offline integrity check for the frozen bytes 1.11.1 Aeneas evidence snapshot.

This checks copied-file integrity and receipt consistency only. It does not run
Lean, Aeneas, Charon, Cargo, or cargo-elicit and does not create proof evidence.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import tarfile
import tomllib
from collections import Counter
from pathlib import Path, PurePosixPath


HERE = Path(__file__).resolve().parent
SNAPSHOT = HERE / "snapshot"
MANIFEST_PATH = HERE / "snapshot-manifest.json"
WORKSPACE_PREFIX = "/workspace/bytes-aeneas-fresh/"
FORBIDDEN_AXIOM_MARKERS = ("sorryAx", "._native.")
EXPECTED_EXTENSION_FILES = {
    "Generated.lean",
    "Proofs.lean",
    "manifest.json",
    "extraction.json",
    "Audit.lean",
    "audit.log",
    "build.log",
}


def fail(message: str) -> None:
    raise SystemExit(f"snapshot checkpoint: FAIL: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def read_json(path: Path, label: str) -> dict:
    try:
        value = json.loads(path.read_text())
    except Exception as exc:
        fail(f"cannot read {label} ({path.relative_to(HERE)}): {exc}")
    require(isinstance(value, dict), f"{label} is not a JSON object")
    return value


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def safe_relative_path(base: Path, relative: str, label: str) -> Path:
    rel = PurePosixPath(relative)
    require(
        not rel.is_absolute() and bool(rel.parts) and ".." not in rel.parts,
        f"unsafe {label} path: {relative!r}",
    )
    require(rel.as_posix() == relative, f"non-canonical {label} path: {relative!r}")
    path = base.joinpath(*rel.parts)
    cursor = base
    for part in rel.parts:
        cursor = cursor / part
        require(not cursor.is_symlink(), f"symlink in {label} path: {relative}")
    require(path.is_file(), f"missing {label}: {relative}")
    return path


def check_snapshot_inventory() -> tuple[dict, int]:
    require(SNAPSHOT.is_dir() and not SNAPSHOT.is_symlink(), "snapshot directory missing or symlinked")
    manifest = read_json(MANIFEST_PATH, "snapshot manifest")
    expected = manifest.get("files")
    require(isinstance(expected, dict) and expected, "manifest has no files map")

    actual: set[str] = set()
    for directory, dirnames, filenames in os.walk(SNAPSHOT, followlinks=False):
        current = Path(directory)
        for name in list(dirnames):
            child = current / name
            require(not child.is_symlink(), f"snapshot contains symlink: {child.relative_to(SNAPSHOT)}")
            require(child.is_dir(), f"snapshot contains non-directory entry: {child.relative_to(SNAPSHOT)}")
        for name in filenames:
            child = current / name
            rel = child.relative_to(SNAPSHOT).as_posix()
            require(not child.is_symlink(), f"snapshot contains symlink: {rel}")
            require(child.is_file(), f"snapshot contains non-regular entry: {rel}")
            actual.add(rel)

    expected_names = set(expected)
    missing = sorted(expected_names - actual)
    extra = sorted(actual - expected_names)
    require(not missing, f"manifest files missing from snapshot: {missing[:5]}")
    require(not extra, f"unmanifested snapshot files: {extra[:5]}")

    for relative, record in expected.items():
        require(isinstance(record, dict), f"bad manifest record for {relative}")
        path = safe_relative_path(SNAPSHOT, relative, "manifest")
        require(path.stat().st_size == record.get("size"), f"size mismatch: {relative}")
        require(digest(path) == record.get("sha256"), f"SHA-256 mismatch: {relative}")
    return manifest, len(actual)


def check_no_forbidden_axioms(path: Path, label: str) -> None:
    text = path.read_text(errors="replace")
    for marker in FORBIDDEN_AXIOM_MARKERS:
        require(marker not in text, f"{label} contains forbidden axiom marker {marker!r}")


def check_native_receipt(verification: Path, adapted: Path, baseline: dict) -> int:
    receipt = read_json(verification / "evidence/native-result.json", "native receipt")
    require(receipt.get("status") == "kernel_checked", "native receipt is not kernel_checked")
    proof_files = receipt.get("proof_files")
    require(isinstance(proof_files, dict), "native receipt has no proof_files map")
    total = 0
    for relative, record in proof_files.items():
        path = safe_relative_path(verification, relative, "native proof file")
        require(digest(path) == record.get("sha256"), f"native receipt hash mismatch: {relative}")
        theorem_names = re.findall(r"^theorem\s+(\w+)", path.read_text(), re.MULTILINE)
        require(theorem_names == record.get("theorems"), f"native theorem list mismatch: {relative}")
        total += len(theorem_names)

    source_hashes = receipt.get("rust_source_hashes")
    require(source_hashes == baseline.get("rust_hashes"), "native/adapted Rust hash maps differ")
    for relative, expected in source_hashes.items():
        path = safe_relative_path(adapted, relative, "native Rust source")
        require(digest(path) == expected, f"native Rust source hash mismatch: {relative}")
    cargo = safe_relative_path(adapted, "Cargo.toml", "adapted Cargo manifest")
    require(digest(cargo) == receipt.get("cargo_manifest_sha256"), "native Cargo.toml hash mismatch")

    logs = receipt.get("axiom_audit_logs", [])
    require(len(logs) == 2, "native receipt does not name both axiom-audit logs")
    for relative in logs:
        check_no_forbidden_axioms(
            safe_relative_path(verification, relative, "native axiom log"),
            f"native axiom log {relative}",
        )
    require(total == 26, f"native receipt has {total} theorem claims, expected 26")
    return total


def raw_llbc_path(group_dir: Path, extraction: dict, snapshot: Path) -> Path:
    reference = extraction.get("raw_llbc")
    if reference:
        return safe_relative_path(snapshot, reference, f"{group_dir.name} raw LLBC")

    raw_generated = PurePosixPath(extraction.get("raw_generated", ""))
    parts = raw_generated.parts
    require(
        len(parts) >= 3 and parts[0] == "evidence" and parts[1] == f"extension-{group_dir.name}",
        f"{group_dir.name} extraction has no explicit raw LLBC reference",
    )
    return safe_relative_path(
        snapshot,
        PurePosixPath(*parts[:2], "crate.llbc").as_posix(),
        f"{group_dir.name} raw LLBC",
    )


def check_extensions(verification: Path) -> tuple[int, int, set[str]]:
    manifests = sorted((verification / "extensions").glob("*/manifest.json"))
    require(len(manifests) == 17, f"found {len(manifests)} extension manifests, expected 17")
    total = 0
    names: set[str] = set()

    for manifest_path in manifests:
        folder = manifest_path.parent
        group = folder.name
        receipt = read_json(folder / "receipt.json", f"{group} receipt")
        manifest = read_json(manifest_path, f"{group} manifest")
        extraction = read_json(folder / "extraction.json", f"{group} extraction metadata")
        require(receipt.get("status") == "kernel_checked", f"{group} receipt is not kernel_checked")
        require(receipt.get("group") == group, f"{group} receipt group mismatch")

        checksums = receipt.get("files")
        require(isinstance(checksums, dict), f"{group} receipt has no file hashes")
        require(set(checksums) == EXPECTED_EXTENSION_FILES, f"{group} receipt file list changed")
        for relative, expected in checksums.items():
            path = safe_relative_path(folder, relative, f"{group} receipt file")
            require(digest(path) == expected, f"{group} receipt hash mismatch: {relative}")

        theorem_names = sorted({proof["theorem"] for proof in manifest.get("proofs", [])})
        require(theorem_names and len(theorem_names) == len(set(theorem_names)), f"{group} has no unique claims")
        require(theorem_names == receipt.get("theorems"), f"{group} receipt theorem list mismatch")
        for theorem in theorem_names:
            require(theorem not in names, f"duplicate extension theorem claim: {theorem}")
            names.add(theorem)
        total += len(theorem_names)

        audit_log = safe_relative_path(folder, "audit.log", f"{group} axiom log")
        check_no_forbidden_axioms(audit_log, f"{group} axiom log")

        raw_generated = extraction.get("raw_generated")
        require(isinstance(raw_generated, str), f"{group} has no raw_generated path")
        raw_path = safe_relative_path(SNAPSHOT, raw_generated, f"{group} raw generated Lean")
        require(digest(raw_path) == extraction.get("raw_sha256"), f"{group} raw generated hash mismatch")
        raw_text = raw_path.read_text()
        import_line = "public import Aeneas\n"
        require(raw_text.count(import_line) == 1, f"{group} raw extraction import is ambiguous")
        expected_generated = raw_text.replace(
            import_line,
            import_line + "public import FloatTypes\n",
            1,
        )
        require((folder / "Generated.lean").read_text() == expected_generated,
                f"{group} Generated.lean differs beyond the recorded FloatTypes import")

        raw_llbc = raw_llbc_path(folder, extraction, SNAPSHOT)
        llbc_hash = extraction.get("raw_llbc_sha256")
        if llbc_hash is not None:
            require(digest(raw_llbc) == llbc_hash, f"{group} raw LLBC hash mismatch")

    return len(manifests), total, names


def check_coverage_and_results(verification: Path, native_total: int, extension_total: int) -> None:
    results = read_json(verification / "results.json", "results")
    native = results.get("native", {})
    require(native.get("theorem_count_including_helpers") == native_total, "results native claim count mismatch")
    require(native.get("extension_mapped_theorem_claim_count") == extension_total,
            "results extension claim count mismatch")
    total = native_total + extension_total
    require(total == 272, f"receipt total is {total}, expected 272")
    require(native.get("total_theorem_claims_including_helpers") == total,
            "results total claim count mismatch")

    result_groups = {
        entry["group"]: entry["mapped_theorem_claim_count"]
        for entry in native.get("extensions", [])
    }
    actual_groups = {
        manifest_path.parent.name: len(
            read_json(manifest_path.parent / "receipt.json", f"{manifest_path.parent.name} receipt")["theorems"]
        )
        for manifest_path in sorted((verification / "extensions").glob("*/manifest.json"))
    }
    require(result_groups == actual_groups, "results extension group counts do not match current receipts")

    coverage = results.get("coverage", {})
    summary = read_json(verification / "coverage/summary.json", "coverage summary")
    ledger = read_json(verification / "coverage/ledger.json", "coverage ledger")
    for key in (
        "local_functions", "transparent", "opaque", "source_function_transparent",
        "source_function_opaque", "proved", "conditional_proved", "blocked",
        "not_yet_specified", "mapped_native_theorems", "mapped_extension_theorems",
    ):
        require(coverage.get(key) == summary.get(key), f"results/coverage summary mismatch: {key}")
    require(summary.get("mapped_native_theorems") == 24, "coverage native body mapping count changed")
    require(summary.get("mapped_extension_theorems") == extension_total,
            "coverage extension mapping count mismatch")
    receipts = ledger.get("coverage_receipts", {})
    require(receipts.get("extension_receipted_theorems_mapped_to_rust_functions") == extension_total,
            "coverage ledger extension theorem total mismatch")
    require(receipts.get("native_receipted_body_theorems_mapped_to_rust_functions") == 24,
            "coverage ledger native body theorem total mismatch")
    require(receipts.get("strict_api_audit_successes_mapped") == 22,
            "coverage ledger strict audit mapping count mismatch")
    require(receipts.get("unique_rust_functions_with_receipted_coverage") == 218,
            "coverage ledger unique Rust function count mismatch")
    require(not receipts.get("unmapped_extension_claims"), "coverage ledger has unmapped extension claims")
    require(not receipts.get("unmapped_native_claims"), "coverage ledger has unmapped native claims")
    validations = receipts.get("extension_receipt_validation", [])
    require(len(validations) == 17, "coverage ledger receipt validation count mismatch")
    require(all(v.get("status") == "kernel_checked_current" and not v.get("mismatches")
                for v in validations), "coverage ledger records stale extension receipts")


def check_elicit(verification: Path, adapted: Path) -> tuple[int, int]:
    receipt_path = verification / "elicit-integration/audit-selected-artifacts/receipt.json"
    receipt = read_json(receipt_path, "strict Elicit receipt")
    final_receipt = verification / "elicit-integration/audit-expanded-final-receipt.json"
    require(final_receipt.is_file() and final_receipt.read_bytes() == receipt_path.read_bytes(),
            "strict audit receipt copies differ")
    require(receipt.get("kind") == "audit", "Elicit receipt is not an audit")

    report = receipt.get("report", {})
    require(report.get("stable") is True and report.get("environment_error") is False,
            "strict Elicit audit is not stable")
    results = report.get("results", [])
    outcomes = Counter(item.get("outcome") for item in results)
    require(len(results) == 24 and outcomes == Counter({"success": 22, "unresolved": 2}),
            f"strict Elicit outcomes changed: {dict(outcomes)}")
    require(len(receipt.get("goals", [])) == 24, "strict Elicit receipt goal inventory changed")

    cache_name = report.get("cache")
    require(isinstance(cache_name, str) and cache_name.startswith(".elicit/evidence/"),
            "strict Elicit report has no relative cache reference")
    source_cache = adapted / cache_name
    if source_cache.is_dir():
        cache_dir = source_cache
        raw_receipt = adapted / (cache_name + ".json")
        require(raw_receipt.is_file() and raw_receipt.read_bytes() == receipt_path.read_bytes(),
                "authoritative .elicit receipt differs from verification copy")
    else:
        # The frozen archive preserves the byte-identical Elicit receipt and its
        # generated artifacts in the integration directory, not the duplicate
        # cargo-elicit cache directory under bytes-adapted/.elicit/evidence.
        cache_dir = verification / "elicit-integration/audit-selected-artifacts"
    for name in (
        "Audit.lean", "ElicitAdapter.lean", "ElicitContracts.lean", "ElicitGenerated.lean",
        "Proofs.lean", "crate.llbc", "logs/Audit.log", "logs/aeneas.log", "logs/charon.log",
    ):
        safe_relative_path(cache_dir, name, "strict Elicit cached artifact")
        selected = verification / "elicit-integration/audit-selected-artifacts" / name
        if name in {"Audit.lean", "ElicitAdapter.lean", "ElicitContracts.lean", "ElicitGenerated.lean", "Proofs.lean", "crate.llbc"}:
            final = verification / "elicit-integration/audit-expanded-final-artifacts" / name
            require(selected.is_file() and final.is_file() and digest(selected) == digest(final),
                    f"strict Elicit artifact copies differ: {name}")
            if source_cache.is_dir():
                require(digest(selected) == digest(cache_dir / name),
                        f"strict Elicit cached artifact differs: {name}")

    inputs = receipt.get("input_files", {})
    require(isinstance(inputs, dict), "strict Elicit receipt has no input hash map")
    file_inputs = 0
    for original, expected in inputs.items():
        if original.startswith("@"):
            continue
        require(original.startswith(WORKSPACE_PREFIX), f"unmapped strict Elicit input path: {original}")
        relative = original[len(WORKSPACE_PREFIX):]
        path = safe_relative_path(SNAPSHOT, relative, "strict Elicit input")
        require(digest(path) == expected, f"strict Elicit input hash mismatch: {relative}")
        file_inputs += 1
    require(file_inputs > 0, "strict Elicit input hash map has no physical files")
    return outcomes["success"], len(results)


def check_source_provenance(snapshot: Path, verification: Path, adapted: Path, baseline: dict) -> int:
    provenance = read_json(snapshot / "source-provenance.json", "source provenance")
    archive = safe_relative_path(snapshot, "upstream/bytes-1.11.1.crate", "upstream crate archive")
    require(digest(archive) == provenance.get("archive_sha256"), "upstream crate archive SHA-256 mismatch")

    pristine = snapshot / "bytes-1.11.1"
    adapted = snapshot / "bytes-adapted"
    pristine_rust = {p.relative_to(pristine).as_posix(): p for p in pristine.rglob("*.rs")}
    adapted_rust = {p.relative_to(adapted).as_posix(): p for p in adapted.rglob("*.rs")}
    require(set(pristine_rust) == set(adapted_rust), "pristine/adapted Rust file inventories differ")

    with tarfile.open(archive, mode="r:gz") as package:
        archive_rust = {
            member.name.removeprefix("bytes-1.11.1/"): member
            for member in package.getmembers()
            if member.isfile() and member.name.startswith("bytes-1.11.1/") and member.name.endswith(".rs")
        }
        require(set(archive_rust) == set(pristine_rust), "crate archive/pristine Rust file inventories differ")
        for relative, path in pristine_rust.items():
            stream = package.extractfile(archive_rust[relative])
            require(stream is not None and stream.read() == path.read_bytes(),
                    f"pristine Rust file differs from crate archive: {relative}")

    source_files = sorted(p.relative_to(pristine).as_posix() for p in pristine.glob("src/**/*.rs"))
    require(len(source_files) == provenance.get("src_file_count") == 19,
            "pristine src Rust file count differs from provenance")
    require(provenance.get("src_identical_to_archive") is True, "provenance denies pristine source identity")

    changed = sorted(
        relative for relative in pristine_rust
        if pristine_rust[relative].read_bytes() != adapted_rust[relative].read_bytes()
    )
    ledger = read_json(verification / "coverage/ledger.json", "coverage ledger")
    declared = sorted(ledger.get("source_correspondence", {}).get("adapted_rust_files_with_changes", []))
    expected_changes = sorted(["src/buf/buf_impl.rs", "src/buf/buf_mut.rs", "src/bytes.rs"])
    require(set(changed) == set(declared) == set(expected_changes),
            f"adapted Rust differences are not the recorded three files: {changed}")

    rust_hashes = baseline.get("rust_hashes", {})
    require(set(rust_hashes) == set(source_files), "opaque baseline Rust source inventory changed")
    for relative, expected in rust_hashes.items():
        require(digest(adapted / relative) == expected, f"frozen adapted source hash mismatch: {relative}")

    patterns = baseline.get("opaque_patterns", [])
    require(len(patterns) == 209 and len(set(patterns)) == 209, "frozen opacity pattern count is not 209")
    cargo = tomllib.loads((adapted / "Cargo.toml").read_text())
    configured = cargo["package"]["metadata"]["charon"]["opaque"]
    require(sorted(configured) == sorted(patterns), "adapted Cargo opacity differs from frozen 209-pattern set")
    return len(patterns)


def main() -> None:
    manifest, file_count = check_snapshot_inventory()
    verification = SNAPSHOT / "verification"
    adapted = SNAPSHOT / "bytes-adapted"
    baseline = read_json(verification / "opaque-baseline.json", "opaque baseline")

    native_total = check_native_receipt(verification, adapted, baseline)
    extension_groups, extension_total, _ = check_extensions(verification)
    check_coverage_and_results(verification, native_total, extension_total)
    accepted, goals = check_elicit(verification, adapted)
    opacity_patterns = check_source_provenance(SNAPSHOT, verification, adapted, baseline)

    print(
        "snapshot checkpoint: PASS "
        f"({file_count} manifested files; native {native_total}; "
        f"extensions {extension_groups}/{extension_total}; total {native_total + extension_total}; "
        f"strict Elicit {accepted}/{goals}; {opacity_patterns} frozen opacity patterns)"
    )
    print("Integrity validation only; no proofs were rerun and no fresh proof claim was made.")


if __name__ == "__main__":
    main()
