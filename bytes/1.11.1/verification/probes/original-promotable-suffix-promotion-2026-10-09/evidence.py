#!/usr/bin/env python3
"""Capture and audit the AV full-proof origin and correspondence-based admission."""
from __future__ import annotations
import argparse, hashlib, io, json, os, pathlib, tarfile

R = pathlib.Path(__file__).resolve().parent
PROBES = R.parent
REPOSITORY = R.parents[4]
TOOLS = pathlib.Path("/workspace/bytes-proof-tools")
PACKAGE = "bytes-original-promotable-suffix-promotion"
AU_DIR = PROBES / "original-owned-view-call-summaries-2026-10-09"
AU_PREFIX = "inputs/repository/bytes/1.11.1/verification/probes/original-owned-view-call-summaries-2026-10-09"
ORIGIN_LABEL = "av-full-diagnostic-v1"
IDENTITY_PATH = pathlib.Path("/workspace/work/av-proof-reuse-identity.json")
ORIGIN_LOG = pathlib.Path("/workspace/work/av-full-diagnostic-v1.log")
CARGO_LOG = pathlib.Path("/workspace/work/av-compiled-capture.log")
CORRESPONDENCE_LOG = pathlib.Path("/workspace/work/av-full-correspondence.log")
SCOPE = ("One promote-then-first-Clone suffix client: an owned root advances by a caller-supplied amount, "
         "then performs one stored-vtable Clone; the root drops before the suffix is read and the child drops "
         "after the returned Vec is evaluated. Full crate behavior, allocator parity, unwind, general callers, "
         "and unbounded/concurrent owners are outside this evidence.")
AU_ARCHIVE_SHA = "0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701"
AU_MEMBERS = 1670
AU_RECEIPT_SHA = "7b36a3e407207433d1ae640e5439c38f519df1e9edd92b40ecf581c4a4fd5ef1"
AU_AUDIT_SHA = "629b366fe271a6c3bf676c657cd4d307ca38cbed45b74b7e3c0bd330d031d51f"
AU_AUDIT_MD_SHA = "faab417cb546718aae214b98ee3ec6f6f846bb3dae2a9f566fba69c66fcdcd0b"
AU_REPLAY_SHA = "70f0d0bb2facd57466f74fb938fe55fbed3ca22c7736a56ebcb2fc1650b4ab71"
AU_CHECKER_SHA = "c4cea83526fce3743c76b087e8d86cce901ca17167d5bb4ef239be6bfaa4b37f"
AV_CHECKER = R / "check_correspondence.py"
AV_NATIVE = R / "check_native.py"

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def regular(path: pathlib.Path) -> bytes:
    assert path.is_file() and not path.is_symlink(), f"expected a regular file: {path}"
    return path.read_bytes()

def put(files: dict[str, bytes], name: str, data: bytes) -> None:
    assert name and not pathlib.PurePosixPath(name).is_absolute() and ".." not in pathlib.PurePosixPath(name).parts
    if name in files:
        assert files[name] == data, f"conflicting duplicate archive member: {name}"
        return
    files[name] = data

def add_path(files: dict[str, bytes], name: str, path: pathlib.Path) -> None:
    put(files, name, regular(path))

def add_tree(files: dict[str, bytes], base: pathlib.Path, prefix: str, skip=()) -> None:
    assert base.is_dir() and not base.is_symlink(), f"missing/redirected tree: {base}"
    for path in base.rglob("*"):
        assert not path.is_symlink(), f"symlink in archived tree: {path}"
        if path.is_dir():
            continue
        assert path.is_file(), f"nonregular archive input: {path}"
        rel = path.relative_to(base)
        if not any(part in skip for part in rel.parts):
            put(files, prefix + "/" + rel.as_posix(), path.read_bytes())

def archive_files(data: bytes, expected_sha: str, expected_members: int, label: str) -> dict[str, bytes]:
    assert sha(data) == expected_sha, f"{label} archive SHA-256 mismatch"
    result = {}
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
        members = tar.getmembers()
        names = [m.name for m in members]
        assert len(members) == expected_members and len(set(names)) == expected_members
        for member in members:
            p = pathlib.PurePosixPath(member.name)
            assert member.isfile() and not p.is_absolute() and ".." not in p.parts
            stream = tar.extractfile(member)
            assert stream is not None
            result[member.name] = stream.read()
    return result

def proof_stats(proof_files: list[bytes]) -> dict[str, int]:
    out = {"files": len(proof_files), "prover": 0, "null": 0, "structural": 0}
    def visit(node):
        if node is None: out["null"] += 1
        elif isinstance(node, dict) and "children" in node:
            if not node["children"]: out["structural"] += 1
            for child in node["children"]: visit(child)
        elif isinstance(node, dict) and "prover" in node: out["prover"] += 1
        else: raise ValueError(f"unknown proof node: {node!r}")
    for data in proof_files:
        coma = json.loads(data).get("proofs", {}).get("Coma")
        assert isinstance(coma, dict)
        for node in coma.values(): visit(node)
    return out

def current_pairs() -> tuple[dict[str, bytes], list[dict], dict[str, int], dict]:
    policy = json.loads(regular(R / "generated/proof-targets.json"))
    included = policy.get("included")
    assert isinstance(included, list) and included == sorted(included) and len(included) == len(set(included))
    assert policy.get("excluded") == {} and policy.get("features") == []
    assert policy.get("terminal_feature", "") == "" and policy.get("source_control", "") == ""
    comas = {p.relative_to(R).as_posix() for p in (R / "verif").rglob("*.coma")}
    proofs = {p.relative_to(R).as_posix() for p in (R / "verif").rglob("proof.json")}
    assert set(included) == comas
    expected_proofs = {str(pathlib.PurePosixPath(c).with_suffix("") / "proof.json") for c in included}
    assert proofs == expected_proofs
    files = {}
    rows = []
    proof_data = []
    for coma in included:
        proof = (pathlib.PurePosixPath(coma).with_suffix("") / "proof.json").as_posix()
        cdata = regular(R / coma); pdata = regular(R / proof)
        assert cdata and pdata
        files[coma] = cdata; files[proof] = pdata; proof_data.append(pdata)
        rows.append({"coma": "probe/" + coma, "coma_sha256": sha(cdata),
                     "proof": "probe/" + proof, "proof_sha256": sha(pdata)})
    stats = proof_stats(proof_data)
    assert stats["null"] == 0 and stats["structural"] == 0
    assert stats["files"] == len(included)
    return files, rows, stats, policy

def add_au_canonical(files: dict[str, bytes]) -> None:
    ev = AU_DIR / "evidence"
    archive = regular(ev / "au-positive-reuse-canonical-v1.tar.gz")
    receipt = regular(ev / "au-positive-reuse-canonical-v1.json")
    audit = regular(ev / "AU_CANONICAL_AUDIT.json")
    audit_md = regular(ev / "AU_CANONICAL_AUDIT.md")
    replay = regular(ev / "au-positive-reuse-canonical-v1-audit.json")
    assert sha(archive) == AU_ARCHIVE_SHA and sha(receipt) == AU_RECEIPT_SHA
    assert sha(audit) == AU_AUDIT_SHA and sha(audit_md) == AU_AUDIT_MD_SHA and sha(replay) == AU_REPLAY_SHA
    members = archive_files(archive, AU_ARCHIVE_SHA, AU_MEMBERS, "published AU")
    au_receipt = json.loads(receipt)
    table = {row["path"]: row["sha256"] for row in au_receipt["members"]}
    assert len(table) == AU_MEMBERS and set(table) == set(members)
    assert all(sha(data) == table[name] for name, data in members.items())
    assert au_receipt.get("status") == "admitted_reuse"
    assert au_receipt.get("statistics") == {"files": 157, "prover": 1447, "null": 0, "structural": 0}
    assert sha(members["probe/check_correspondence.py"]) == AU_CHECKER_SHA
    # Rehydrate the AU canonical probe at its real sibling path in the archived repository layout.
    for name, data in members.items():
        if name.startswith("probe/"):
            put(files, AU_PREFIX + "/" + name[len("probe/"):], data)
        elif name.startswith("inputs/repository/"):
            put(files, name, data)
        elif name.startswith("inputs/private-std/"):
            put(files, name, data)
        elif name.startswith("inputs/tools/"):
            put(files, "inputs/au-canonical/" + name, data)
        elif name.startswith("admission/"):
            put(files, "inputs/au-canonical/" + name, data)
    for name, data in [
        ("au-positive-reuse-canonical-v1.tar.gz", archive),
        ("au-positive-reuse-canonical-v1.json", receipt),
        ("AU_CANONICAL_AUDIT.json", audit),
        ("AU_CANONICAL_AUDIT.md", audit_md),
        ("au-positive-reuse-canonical-v1-audit.json", replay),
    ]:
        put(files, AU_PREFIX + "/evidence/" + name, data)

def collect_files(*, origin_log: pathlib.Path | None, admission_log: pathlib.Path | None = None,
                  identity: pathlib.Path | None = None, skip_origin_duplicate: bool = False) -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    add_tree(files, R, "probe", ("evidence", "target", ".git", "__pycache__", ".proof-config",
                                  ".why3find", ".why3findcache"))
    # Preserve any earlier frontend/native diagnostic captures without including this archive recursively.
    ev = R / "evidence"
    if ev.is_dir():
        for path in ev.iterdir():
            if path.is_file() and path.suffix in {".json", ".md", ".log", ".tar.gz"}:
                if skip_origin_duplicate and path.name in {
                    ORIGIN_LABEL + ".tar.gz", ORIGIN_LABEL + ".json", ORIGIN_LABEL + "-audit.json"
                }:
                    continue
                add_path(files, "probe/evidence/" + path.name, path)
    add_tree(files, REPOSITORY / "bytes/1.11.1/src", "inputs/repository/bytes/1.11.1/src")
    for name in ("Cargo.toml", "Cargo.lock", "Cargo.toml.orig"):
        path = REPOSITORY / "bytes/1.11.1" / name
        if path.exists(): add_path(files, "inputs/repository/bytes/1.11.1/" + name, path)
    add_tree(files, TOOLS / "bytes-proof-std", "inputs/private-std", ("target", ".git", "__pycache__"))
    for name, path in [
        ("installation-manifest.json", TOOLS / "installation-manifest.json"),
        ("activate.sh", TOOLS / "activate.sh"),
        ("creusot_why3.conf", TOOLS / "creusot-data/creusot_why3.conf"),
        ("why3-main.conf", TOOLS / "config/creusot/why3.conf"),
        ("cargo-config.toml", TOOLS / "cargo/config.toml"),
        ("base-activate.sh", pathlib.Path("/workspace/proof-tools/activate.sh")),
        ("why3-local-derived.conf", R / ".proof-config/creusot/why3.conf"),
    ]:
        if path.exists(): add_path(files, "inputs/tools/" + name, path)
    add_au_canonical(files)
    if origin_log is not None:
        add_path(files, "admission/av-full-diagnostic-v1.log", origin_log)
    if admission_log is not None: add_path(files, "admission/av-full-correspondence.log", admission_log)
    if identity is not None: add_path(files, "admission/av-proof-reuse-identity.json", identity)
    if CARGO_LOG.exists(): add_path(files, "admission/av-compiled-capture.log", CARGO_LOG)
    return files

def write_archive(label: str, files: dict[str, bytes], status: str, targets: list[dict],
                  stats: dict, policy: dict, scope: str, extra: dict | None = None) -> dict:
    ev = R / "evidence"; ev.mkdir(exist_ok=True)
    archive_path = ev / (label + ".tar.gz"); receipt_path = ev / (label + ".json")
    assert not archive_path.exists() and not receipt_path.exists(), "refusing to overwrite immutable evidence label"
    with tarfile.open(archive_path, "w:gz") as tar:
        for name, data in sorted(files.items()):
            info = tarfile.TarInfo(name); info.size = len(data); info.mode = 0o644; info.mtime = 0
            tar.addfile(info, io.BytesIO(data))
    raw = regular(archive_path)
    members = [{"path": name, "sha256": sha(data)} for name, data in sorted(files.items())]
    receipt = {"schema": "av-proof-evidence-v1", "archive": archive_path.name,
        "archive_sha256": sha(raw), "status": status, "statistics": stats, "targets": targets,
        "target_policy": policy, "members": members, "scope": scope,
        "checker_scope": ("full AV diagnostic proof origin; correspondence was skipped and full-crate admission is not claimed"
            if status == "diagnostic" else
            "completed AV full proof reused without solver rerun, plus fresh source/native/Cargo correspondence; full-crate admission is not claimed"),
        "target_policy_role": ("recorded proof-origin diagnostic policy; later correspondence admission is a separate artifact"
            if status == "diagnostic" else "proof-origin policy retained as provenance; current correspondence admission is recorded separately")}
    if extra: receipt.update(extra)
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt

def capture_origin(label: str, log_path: pathlib.Path) -> dict:
    assert label == ORIGIN_LABEL, "origin label is fixed and immutable"
    log = regular(log_path)
    target_files, targets, stats, policy = current_pairs()
    assert policy.get("diagnostic") is True and policy.get("correspondence_exit_status") == 2
    assert stats == {"files": 167, "prover": 1683, "null": 0, "structural": 0}, \
        "origin proof inventory does not match the observed full AV proof"
    assert len(targets) == 167 and len(target_files) == 334
    # Capture the already completed fresh Cargo build identity before later controls
    # can replace its OUT_DIR artifacts. These are snapshots, not claims of proof.
    compiled_path = R / "generated/compiled-capture-summary.json"
    compiled = json.loads(regular(compiled_path))
    compiled_input = compiled.get("AV_compiled_production_input", {})
    assert compiled.get("status") == "pass"
    assert compiled_input.get("status") == "pass"
    assert compiled_input.get("captured_artifact_count") == 4
    assert compiled_input.get("snapshot_written") is True
    assert compiled_input.get("fingerprint_inputs_exact") is True
    assert compiled_input.get("build_output_exact") is True
    assert compiled_input.get("OUT_DIR_root_output_join_exact") is True
    assert compiled_input.get("public_records_reconstructed") is True
    assert compiled_input.get("live_fingerprint_and_output_checked") is True
    marker = f"Proved ({stats['files']} files)".encode()
    assert marker in log, "origin log is missing the full proof completion marker"
    files = collect_files(origin_log=log_path)
    for name, data in target_files.items(): assert files["probe/" + name] == data
    assert files["probe/generated/compiled-capture-summary.json"] == regular(compiled_path)
    receipt = write_archive(label, files, "diagnostic", targets, stats, policy, SCOPE,
        {"proof_execution": {"prover_process_exit_status": "not-recorded-in-archive",
             "completion_marker": marker.decode(), "completion_marker_present": True,
             "correspondence_exit_status": 2, "diagnostic_origin": True}})
    return receipt

def read_bundle(path: pathlib.Path) -> tuple[dict[str, bytes], dict]:
    raw = regular(path)
    with tarfile.open(path, "r:gz") as tar:
        names = [m.name for m in tar.getmembers()]
        assert len(names) == len(set(names)) and all(m.isfile() for m in tar.getmembers())
        files = {}
        for member in tar.getmembers():
            p = pathlib.PurePosixPath(member.name)
            assert not p.is_absolute() and ".." not in p.parts
            stream = tar.extractfile(member); assert stream is not None; files[member.name] = stream.read()
    receipt_path = path.with_suffix("").with_suffix(".json")
    receipt = json.loads(regular(receipt_path))
    assert sha(raw) == receipt.get("archive_sha256")
    table = {row["path"]: row["sha256"] for row in receipt.get("members", [])}
    assert len(table) == len(files) and set(table) == set(files)
    assert all(sha(data) == table[name] for name, data in files.items())
    return files, receipt

def audit_origin(label: str) -> dict:
    archive_path = R / "evidence" / (label + ".tar.gz")
    files, receipt = read_bundle(archive_path)
    assert label == ORIGIN_LABEL and receipt.get("status") == "diagnostic"
    assert receipt.get("proof_execution", {}).get("prover_process_exit_status") == "not-recorded-in-archive"
    origin_log = files["admission/av-full-diagnostic-v1.log"]
    marker = receipt["proof_execution"]["completion_marker"].encode()
    assert receipt["proof_execution"]["completion_marker_present"] is True and marker in origin_log
    targets = receipt["targets"]
    proof_blobs = []
    seen = set()
    for row in targets:
        for field, digest_field in (("coma", "coma_sha256"), ("proof", "proof_sha256")):
            name = row[field]; assert name in files and name not in seen
            seen.add(name); assert sha(files[name]) == row[digest_field]
            if field == "proof": proof_blobs.append(files[name])
    assert proof_stats(proof_blobs) == receipt["statistics"]
    au_tar = files[AU_PREFIX + "/evidence/au-positive-reuse-canonical-v1.tar.gz"]
    archive_files(au_tar, AU_ARCHIVE_SHA, AU_MEMBERS, "embedded AU")
    report = {"status": "pass", "archive_sha256": receipt["archive_sha256"],
        "members_verified": len(files), "statistics": receipt["statistics"],
        "proof_run_claim": "completion marker recorded; numeric process exit not archived",
        "full_original_admitted": False}
    (R / "evidence" / (label + "-audit.json")).write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return report

def capture_admitted(label: str, log_path: pathlib.Path, identity_path: pathlib.Path,
                     *, preflight: bool = False) -> dict:
    origin_path = R / "evidence" / (ORIGIN_LABEL + ".tar.gz")
    origin_receipt_path = R / "evidence" / (ORIGIN_LABEL + ".json")
    if preflight:
        assert not (R / "evidence" / (label + ".tar.gz")).exists()
        assert not (R / "evidence" / (label + ".json")).exists()
    origin_files, origin_receipt = read_bundle(origin_path)
    origin_receipt_bytes = regular(origin_receipt_path)
    assert sha(origin_receipt_bytes) == sha(json.dumps(origin_receipt, indent=2).encode() + b"\n") or json.loads(origin_receipt_bytes) == origin_receipt
    target_files, targets, stats, policy = current_pairs()
    origin_targets = {x[f] .removeprefix("probe/"): x[f + "_sha256"] for x in origin_receipt["targets"] for f in ("coma", "proof")}
    current_targets = {row[field].removeprefix("probe/"): row[field + "_sha256"] for row in targets for field in ("coma", "proof")}
    assert origin_targets == current_targets, "current AV COMA/proof bytes differ from the completed full origin"
    assert stats == origin_receipt["statistics"]
    assert policy.get("excluded") == {} and policy.get("features") == []
    # An admitted reuse receipt can retain the diagnostic origin policy; current correspondence is separate.
    assert policy.get("diagnostic") is True and policy.get("correspondence_exit_status") == 2
    identity_bytes = regular(identity_path); identity = json.loads(identity_bytes)
    identity_rows = identity.get("all_target_files_sha256")
    assert isinstance(identity_rows, list) and len(identity_rows) == len(current_targets)
    identity_table = {row.get("path"): row.get("sha256") for row in identity_rows}
    assert len(identity_table) == len(identity_rows) and identity_table == current_targets
    assert identity.get("status") == "pass"
    assert identity.get("scope") == "Exact completed AV full proof reuse; full original architecture remains NOT ADMITTED."
    assert identity.get("origin_archive") == ORIGIN_LABEL + ".tar.gz"
    assert identity.get("origin_archive_sha256") == origin_receipt["archive_sha256"]
    assert identity.get("statistics") == origin_receipt["statistics"]
    assert identity.get("prover_reexecuted") is False
    assert identity.get("Rust_active_sha256") == sha(origin_files["probe/generated/active.rs"])
    assert identity.get("allowed_changes") == (
        "Only noncompiled correspondence/audit/documentation metadata; Rust/Cargo/Std/prover inputs and all334 target bytes unchanged."
    )
    correspondence_bytes = regular(R / "generated/correspondence.json")
    correspondence = json.loads(correspondence_bytes)
    assert correspondence.get("status") == "pass" and correspondence.get("full_original_admitted") is False
    assert correspondence.get("AV_proof_inventory", {}).get("target_count") == 167
    assert correspondence.get("AV_proof_inventory", {}).get("prover_leaves") == 1683
    assert correspondence.get("AV_proof_inventory", {}).get("null") == 0
    assert correspondence.get("AV_proof_inventory", {}).get("structural") == 0
    assert correspondence.get("AV_structural_control", {}).get("control_id") == "refreshed_suffix_capacity_recovery_claim"
    assert correspondence.get("AV_structural_control", {}).get("status") == "rejected_as_expected"
    summary_bytes = regular(R / "generated/compiled-capture-summary.json")
    summary = json.loads(summary_bytes)
    assert summary.get("status") == "pass" and summary.get("AV_compiled_production_input", {}).get("captured_artifact_count") == 4
    control_bytes = regular(R / "generated/checker-control-receipt.json")
    controls = json.loads(control_bytes)
    assert controls.get("status") == "pass" and controls.get("control_id") == "refreshed_suffix_capacity_recovery_claim"
    assert controls.get("rejected_as_expected") == 1 and controls.get("accepted") == [] and controls.get("errors") == []
    assert controls.get("checker_sha256") == sha(regular(AV_CHECKER))
    assert controls.get("expected_rejection") == (
        "AV helper inventory, ownership frame, allocation recovery, callback selection, return order, or exclusions changed"
    )
    assert controls.get("control_count") == 1
    cap_dir = R / "evidence/av-capacity-diagnostic-v1"
    cap_receipt_bytes = regular(cap_dir / "receipt.json")
    cap_control = json.loads(cap_receipt_bytes)
    assert cap_control.get("status") == "semantic_control_rejected"
    assert cap_control.get("process_exit_status") == 1
    assert cap_control.get("statistics") == {"files": 1, "prover": 149, "null": 2, "structural": 0}
    assert cap_control.get("translated") == 167 and cap_control.get("included") == 1 and cap_control.get("excluded") == 166
    assert cap_control.get("positive_origin_archive_sha256") == origin_receipt["archive_sha256"]
    assert cap_control.get("primary_366_critical_source_and_proof_hashes_unchanged") is True
    assert cap_control.get("primary_source_or_proofs_regenerated") is False
    assert len(cap_control.get("actual_null_tasks", [])) == 2
    cap_active = regular(cap_dir / "generated/active.rs")
    origin_active = origin_files["probe/generated/active.rs"]
    cap_line = b"    let cap=len;"
    origin_line = b"    let cap=distance as usize + len;"
    assert cap_active.count(cap_line) == 1
    assert cap_active.replace(cap_line, origin_line, 1) == origin_active
    cap_log_meta = json.loads(regular(cap_dir / "av-capacity-diagnostic-v1.log").splitlines()[0])
    assert cap_log_meta.get("active_sha256") == sha(cap_active)
    cap_mapping = json.loads(regular(cap_dir / "generated/mapping.json"))
    assert cap_mapping.get("active_sha256") == sha(cap_active)
    for task in cap_control["actual_null_tasks"]:
        task_bytes = regular(cap_dir / pathlib.PurePosixPath(task["path"]).name)
        assert sha(task_bytes) == task["sha256"]
    log_bytes = regular(log_path); assert b'"status": "pass"' in log_bytes or b'"status":"pass"' in log_bytes
    files = collect_files(origin_log=ORIGIN_LOG, admission_log=log_path, identity=identity_path,
                          skip_origin_duplicate=True)
    add_tree(files, cap_dir, "admission/av-capacity-diagnostic-v1")
    # The immutable completed proof is the reuse origin. Critical current proof
    # inputs must still be the exact bytes captured with it; admission metadata
    # and this one control harness may be added afterward.
    origin_critical = [
        "probe/Cargo.toml", "probe/Cargo.lock", "probe/build.rs", "probe/elaborate.py", "probe/run-proof.sh",
        "probe/inherited-targets.json", "probe/generated/active.rs", "probe/generated/positive.rs",
        "probe/generated/elaborated-client.rs", "probe/generated/mapping.json", "probe/why3find.json",
    ]
    for name in origin_critical:
        assert files[name] == origin_files[name], f"critical input changed since full proof origin: {name}"
    origin_rust = {n: d for n, d in origin_files.items() if n.startswith("probe/src/") and n.endswith(".rs")}
    current_rust = {n: d for n, d in files.items() if n.startswith("probe/src/") and n.endswith(".rs")}
    assert current_rust == origin_rust, "AV Rust module tree changed since full proof origin"
    production_prefix = "inputs/repository/bytes/1.11.1/"
    production = {n for n in origin_files if n.startswith(production_prefix) and
                  (n.startswith(production_prefix + "src/") or n in {
                      production_prefix + "Cargo.toml", production_prefix + "Cargo.lock",
                      production_prefix + "Cargo.toml.orig"})}
    private_std = {n for n in origin_files if n.startswith("inputs/private-std/")}
    tool_config = {n for n in origin_files if n.startswith("inputs/tools/")}
    critical_probe = {"probe/" + x for x in [
        "Cargo.toml", "Cargo.lock", "build.rs", "elaborate.py", "run-proof.sh", "inherited-targets.json",
        "generated/active.rs", "generated/positive.rs", "generated/elaborated-client.rs",
        "generated/mapping.json", "why3find.json"]}
    target_paths = {"probe/" + name for name in current_targets}
    critical_paths = target_paths | set(origin_rust) | production | private_std | tool_config | critical_probe
    assert len(target_paths) == 334 and len(origin_rust) == 24 and len(production) == 64
    assert len(private_std) == 110 and len(tool_config) == 7 and len(critical_probe) == 11
    assert len(critical_paths) == 550
    identity_critical_rows = identity.get("critical_input_identity")
    assert isinstance(identity_critical_rows, list) and len(identity_critical_rows) == 550
    identity_critical_table = {row.get("path"): row.get("sha256") for row in identity_critical_rows}
    expected_critical_table = {name: sha(origin_files[name]) for name in critical_paths}
    assert len(identity_critical_table) == 550 and identity_critical_table == expected_critical_table
    assert identity.get("critical_input_count") == 550
    for name in critical_paths:
        assert name in files and name in origin_files and files[name] == origin_files[name], \
            f"critical proof input changed since full origin: {name}"
    # Verify that the origin's four actual Cargo artifacts are still the exact
    # bytes being admitted; the archive contains snapshots under this namespace.
    for name in [
        "probe/generated/compiled-inputs/public_records.rs",
        "probe/generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "probe/generated/compiled-inputs/cargo-build-output.txt",
        "probe/generated/compiled-inputs/cargo-root-output.txt",
        "probe/generated/compiled-inputs/public-records-build-receipt.json",
    ]:
        assert files[name] == origin_files[name], f"compiled artifact changed since proof origin: {name}"
    add_path(files, "inputs/av-proof-origin/av-full-diagnostic-v1.tar.gz", origin_path)
    add_path(files, "inputs/av-proof-origin/av-full-diagnostic-v1.json", origin_receipt_path)
    add_path(files, "inputs/av-proof-origin/av-full-diagnostic-v1.log", ORIGIN_LOG)
    add_path(files, "admission/av-compiled-capture-summary.json", R / "generated/compiled-capture-summary.json")
    add_path(files, "admission/av-correspondence.json", R / "generated/correspondence.json")
    add_path(files, "admission/av-checker-control-receipt.json", R / "generated/checker-control-receipt.json")
    stats_now = stats
    proof_reuse = {"reused_full_proof": True, "prover_rerun_claimed": False,
        "origin_archive_sha256": origin_receipt["archive_sha256"], "origin_receipt_sha256": sha(origin_receipt_bytes),
        "origin_statistics": origin_receipt["statistics"], "current_target_pairs_match_origin": True,
        "identity_sha256": sha(identity_bytes), "identity_path": "admission/av-proof-reuse-identity.json",
        "origin_process_exit_status": "not-recorded-in-archive",
        "origin_completion_marker": origin_receipt["proof_execution"]["completion_marker"],
        "critical_inputs_exactly_reused": True, "critical_input_identity_count": 550}
    new_admission = {"correspondence_status": "pass", "correspondence_exit_status": 0,
        "correspondence_sha256": sha(correspondence_bytes), "correspondence_log_sha256": sha(log_bytes),
        "compiled_artifact_count": 4, "compiled_summary_sha256": sha(summary_bytes),
        "structural_control_count": 1, "structural_control_id": controls["control_id"],
        "structural_control_receipt_sha256": sha(control_bytes),
        "focused_semantic_control": {"label": "av-capacity-diagnostic-v1",
            "receipt_sha256": sha(cap_receipt_bytes), "status": cap_control["status"],
            "statistics": cap_control["statistics"], "included_targets": cap_control["included"],
            "excluded_targets": cap_control["excluded"],
            "classification_pending_independent_review": cap_control.get("classification_pending_independent_review")},
        "full_prover_rerun": False}
    if preflight:
        return {"schema": "av-proof-evidence-v1", "archive": label + ".tar.gz",
            "archive_sha256": "not-written-preflight", "status": "preflight_pass",
            "statistics": stats_now, "target_count": len(targets), "member_count": len(files),
            "proof_reuse": proof_reuse, "new_admission": new_admission,
            "full_original_admitted": False}
    return write_archive(label, files, "admitted_reuse", targets, stats_now, policy, SCOPE,
        {"proof_reuse": proof_reuse, "new_admission": new_admission,
         "origin_label": ORIGIN_LABEL, "full_original_admitted": False})

def audit(label: str) -> dict:
    archive_path = R / "evidence" / (label + ".tar.gz")
    files, receipt = read_bundle(archive_path)
    assert receipt.get("scope") == SCOPE
    if receipt.get("status") == "diagnostic":
        return audit_origin(label)
    assert receipt.get("status") == "admitted_reuse" and receipt.get("full_original_admitted") is False
    origin_tar = files["inputs/av-proof-origin/av-full-diagnostic-v1.tar.gz"]
    origin_receipt = json.loads(files["inputs/av-proof-origin/av-full-diagnostic-v1.json"])
    origin_members = archive_files(origin_tar, receipt["proof_reuse"]["origin_archive_sha256"],
        len(origin_receipt["members"]), "AV full origin")
    origin_member_table = {row.get("path"): row.get("sha256") for row in origin_receipt.get("members", [])}
    assert len(origin_member_table) == len(origin_members) and set(origin_member_table) == set(origin_members)
    assert all(sha(data) == origin_member_table[name] for name, data in origin_members.items())
    # Proof rows in the final archive must exactly match the embedded diagnostic origin.
    origin_receipt = json.loads(files["inputs/av-proof-origin/av-full-diagnostic-v1.json"])
    origin_table = {x[f].removeprefix("probe/"): x[f+"_sha256"] for x in origin_receipt["targets"] for f in ("coma","proof")}
    final_table = {x[f].removeprefix("probe/"): x[f+"_sha256"] for x in receipt["targets"] for f in ("coma","proof")}
    assert origin_table == final_table == {name.removeprefix("probe/"): sha(data) for name,data in files.items()
        if name.startswith("probe/verif/") and (name.endswith(".coma") or name.endswith("/proof.json"))}
    identity = json.loads(files["admission/av-proof-reuse-identity.json"])
    assert identity.get("prover_reexecuted") is False and receipt["proof_reuse"].get("reused_full_proof") is True
    assert receipt["proof_reuse"].get("current_target_pairs_match_origin") is True
    assert identity.get("scope") == "Exact completed AV full proof reuse; full original architecture remains NOT ADMITTED."
    assert identity.get("origin_archive_sha256") == origin_receipt["archive_sha256"]
    assert identity.get("statistics") == receipt["statistics"]
    assert identity.get("Rust_active_sha256") == sha(origin_members["probe/generated/active.rs"])
    target_paths = {row[field] for row in receipt["targets"] for field in ("coma", "proof")}
    origin_rust = {n for n in origin_members if n.startswith("probe/src/") and n.endswith(".rs")}
    production_prefix = "inputs/repository/bytes/1.11.1/"
    production = {n for n in origin_members if n.startswith(production_prefix) and
                  (n.startswith(production_prefix + "src/") or n in {
                      production_prefix + "Cargo.toml", production_prefix + "Cargo.lock",
                      production_prefix + "Cargo.toml.orig"})}
    private_std = {n for n in origin_members if n.startswith("inputs/private-std/")}
    tool_config = {n for n in origin_members if n.startswith("inputs/tools/")}
    critical_probe = {"probe/" + x for x in [
        "Cargo.toml", "Cargo.lock", "build.rs", "elaborate.py", "run-proof.sh", "inherited-targets.json",
        "generated/active.rs", "generated/positive.rs", "generated/elaborated-client.rs",
        "generated/mapping.json", "why3find.json"]}
    critical_paths = target_paths | origin_rust | production | private_std | tool_config | critical_probe
    identity_critical_rows = identity.get("critical_input_identity")
    assert isinstance(identity_critical_rows, list)
    identity_critical_table = {row.get("path"): row.get("sha256") for row in identity_critical_rows}
    assert len(target_paths) == 334 and len(origin_rust) == 24 and len(production) == 64
    assert len(private_std) == 110 and len(tool_config) == 7 and len(critical_probe) == 11
    assert len(critical_paths) == 550 and len(identity_critical_rows) == 550
    assert identity.get("critical_input_count") == 550
    assert identity_critical_table == {name: sha(origin_members[name]) for name in critical_paths}
    assert all(files[name] == origin_members[name] for name in critical_paths)
    assert receipt["proof_reuse"].get("critical_inputs_exactly_reused") is True
    assert receipt["proof_reuse"].get("critical_input_identity_count") == 550
    assert len(origin_members) == len(origin_receipt["members"])
    correspondence = json.loads(files["admission/av-correspondence.json"])
    assert correspondence.get("status") == "pass" and correspondence.get("full_original_admitted") is False
    assert correspondence.get("AV_proof_inventory", {}).get("target_count") == 167
    assert correspondence.get("AV_proof_inventory", {}).get("prover_leaves") == 1683
    assert correspondence.get("AV_structural_control", {}).get("control_id") == "refreshed_suffix_capacity_recovery_claim"
    controls = json.loads(files["admission/av-checker-control-receipt.json"])
    assert controls.get("status") == "pass" and controls.get("control_id") == "refreshed_suffix_capacity_recovery_claim"
    assert controls.get("rejected_as_expected") == 1 and controls.get("accepted") == [] and controls.get("errors") == []
    assert controls.get("expected_rejection") == (
        "AV helper inventory, ownership frame, allocation recovery, callback selection, return order, or exclusions changed"
    )
    cap_receipt = json.loads(files["admission/av-capacity-diagnostic-v1/receipt.json"])
    assert cap_receipt.get("positive_origin_archive_sha256") == origin_receipt["archive_sha256"]
    assert cap_receipt.get("statistics") == {"files": 1, "prover": 149, "null": 2, "structural": 0}
    assert cap_receipt.get("primary_366_critical_source_and_proof_hashes_unchanged") is True
    cap_active = files["admission/av-capacity-diagnostic-v1/generated/active.rs"]
    assert cap_active.count(b"    let cap=len;") == 1
    assert cap_active.replace(b"    let cap=len;", b"    let cap=distance as usize + len;", 1) == \
        origin_members["probe/generated/active.rs"]
    cap_log_meta = json.loads(files["admission/av-capacity-diagnostic-v1/av-capacity-diagnostic-v1.log"].splitlines()[0])
    assert cap_log_meta.get("active_sha256") == sha(cap_active)
    cap_mapping = json.loads(files["admission/av-capacity-diagnostic-v1/generated/mapping.json"])
    assert cap_mapping.get("active_sha256") == sha(cap_active)
    for task in cap_receipt.get("actual_null_tasks", []):
        task_bytes = files["admission/av-capacity-diagnostic-v1/" + pathlib.PurePosixPath(task["path"]).name]
        assert sha(task_bytes) == task["sha256"]
    assert receipt["new_admission"].get("focused_semantic_control", {}).get("receipt_sha256") == \
        sha(files["admission/av-capacity-diagnostic-v1/receipt.json"])
    report = {"status": "pass", "archive_sha256": receipt["archive_sha256"],
        "members_verified": len(files), "statistics": receipt["statistics"],
        "proof_reuse": receipt["proof_reuse"], "new_admission": receipt["new_admission"],
        "full_original_admitted": False}
    (R / "evidence" / (label + "-audit.json")).write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2)); return report

def main() -> int:
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="command", required=True)
    c = sub.add_parser("capture"); c.add_argument("label"); c.add_argument("log")
    c.add_argument("--status", choices=("diagnostic","admitted_reuse"), required=True)
    c.add_argument("--identity"); c.add_argument("--preflight", action="store_true")
    a = sub.add_parser("audit"); a.add_argument("label")
    args = p.parse_args()
    try:
        if args.command == "capture":
            if args.preflight:
                if args.status != "admitted_reuse":
                    raise ValueError("--preflight is defined only for admitted-reuse capture")
                result = capture_admitted(args.label, pathlib.Path(args.log),
                    pathlib.Path(args.identity) if args.identity else IDENTITY_PATH, preflight=True)
                print(json.dumps(result, indent=2))
                return 0
            if args.status == "diagnostic":
                result = capture_origin(args.label, pathlib.Path(args.log))
            else:
                result = capture_admitted(args.label, pathlib.Path(args.log),
                    pathlib.Path(args.identity) if args.identity else IDENTITY_PATH)
            print(json.dumps({k: result[k] for k in ("archive", "archive_sha256", "status", "statistics")}, indent=2))
        else: audit(args.label)
    except Exception as exc:
        print(json.dumps({"status":"error","error":f"{type(exc).__name__}: {exc}"}, indent=2))
        return 1
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

