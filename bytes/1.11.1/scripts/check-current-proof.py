#!/usr/bin/env python3
"""Save/check direct-source proof evidence without invoking a solver."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys

TARGETS = (
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/truncate.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/from_vec.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/with_capacity.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/new.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/is_empty.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/zeroed.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/clear.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_vec_pos.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/get_vec_pos.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/set_visible_len.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/advance_vec_layout.coma",
    "verif/bytes_rlib/bytes_mut/impl_BytesMut_0/advance_unchecked.coma",
    "verif/bytes_rlib/bytes_mut/impl_Default_for_BytesMut/default.coma",
    "verif/bytes_rlib/bytes_mut/impl_Default_for_BytesMut/default__refines.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut/from.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut/from__refines.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut_0/from.coma",
    "verif/bytes_rlib/bytes_mut/impl_From_for_BytesMut_0/from__refines.coma",
    "verif/bytes_rlib/bytes_mut/original_capacity_to_repr.coma",
    "verif/bytes_rlib/bytes_mut/original_capacity_from_repr.coma",
    "verif/bytes_rlib/bytes_mut/unique_initialized_prefix_extend.coma",
    "verif/bytes_rlib/bytes_mut/unique_initialized_prefix_suffix.coma",
    "verif/bytes_rlib/bytes_mut/impl_BufMut_for_BytesMut/advance_mut.coma",
    "verif/bytes_rlib/bytes_mut/vptr.coma",
    "verif/bytes_rlib/bytes_mut/invalid_ptr.coma",
    "verif/bytes_rlib/ownership_proof/view_region/split_view_region.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/new_root.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/new_promoted_pair.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/split_owner.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/drop_owner.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_OwnerLedger/prove_ticket_is_active.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/contradict_two_tickets.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/impl_RetirePermit/contradicts_live_ticket.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/map_op_fresh_singleton.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/factor_singleton_removal.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/sole_active_id.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/map_singleton_from_len_one.coma",
    "verif/bytes_rlib/ownership_proof/ref_count_ledger/zero_length_map_has_no_ids.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/new.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/load_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/load_acquire.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/compare_exchange_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/compare_exchange_acqrel_relaxed.coma",
    "verif/bytes_rlib/ref_count_ops/impl_RefCountCell/fetch_sub_release.coma",
    "verif/bytes_rlib/ref_count_ops/proof/impl_EventBoundInvariant_T/new.coma",
    "verif/bytes_rlib/ref_count_ops/proof/observe_ref_count.coma",
    "verif/bytes_rlib/ref_count_ops/proof/update_ref_count.coma",
    "verif/bytes_rlib/ref_count_limit/next_ref_count.coma",
    "verif/bytes_rlib/ref_count_ops/try_increment.coma",
    "verif/bytes_rlib/ref_count_ops/increment.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/advance_within.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/raw_pointer.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/as_ptr.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_BoundPtr/as_non_null.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/split_at.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/bind_consumed_vec.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/slots_from_vec.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/slots_from_vec_values.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/int_from_usize.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_RawAllocation/bound_ptr_at_zero.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/split_at.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/map_compose_eq.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/from_model_ledger.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/slots.coma",
    "verif/bytes_rlib/ownership_proof/owned_region/impl_OwnedRegion/slot.coma",
    "verif/bytes_rlib/ownership_proof/raw_vec/impl_PhysicalRegion/slot.coma",
    "verif/bytes_rlib/provenance_specs/wrapping_offset.coma",
    "verif/bytes_rlib/capacity_ops/pack_vec_metadata.coma",
    "verif/bytes_rlib/capacity_ops/set_vec_pos_in_data.coma",
    "verif/bytes_rlib/vec_storage/impl_VecPartsWitness/native_parts.coma",
    "verif/bytes_rlib/vec_storage/impl_VecPartsWitness/contents_snapshot.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/native_ptr.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/logical_len.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/logical_capacity.coma",
    "verif/bytes_rlib/vec_storage/impl_VecRawParts/into_proof_authority.coma",
)
EVIDENCE_TARGETS = TARGETS
EVIDENCE = "verification/results/direct-bytesmut-len-api"
PINNED = {
    "creusot_sha": "318615be3b8bbc60d1f6d52469ba5c0bdebed4f1",
    "why3_sha": "54c92f96bb0711d6e991c18f10bfbc08d90d028b",
    "why3find_sha": "eab37557d3e24e1913a3c4f44bc5528ef497c6c9",
}
PURITY_PATCH_ID = {
    "id": "bytes-fn-ghost-purity-v1",
    "source_commit": PINNED["creusot_sha"],
    "source_path": "creusot/src/validate/purity.rs",
    "base_source_sha256": "5c56221d3f131c6ad1cfc2aebc85363ba84ae548c742a905f3d38100528b1af0",
    "patched_source_sha256": "857f74d243f068642cf278f707f9b54e93ea433575769525994f15d234eba9f1",
    "patch_sha256": "6eb53cf71094d0987605a6ec2784ef6556fa253832b30f40ff0a8a3179ba9587",
    "compiler_binary": "creusot-rustc-fn-ghost-purity",
}
PROVERS = {"alt-ergo", "z3", "cvc4", "cvc5"}
SPAN = re.compile(r'"(?P<file>/[^"\n]+\.rs)"\s+(?P<start>\d+)\s+\d+\s+(?P<end>\d+)\s+\d+')
WARNING = re.compile(r"no contract|contractless|unsupported|unimplemented|no specification|unproved external", re.I)


class ReuseMiss(ValueError):
    pass


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def path(root: Path, name: str) -> Path:
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and ".." not in relative.parts,
            f"evidence path escapes crate: {name}")
    return root.joinpath(*relative.parts)


def std_manifest(crate: Path) -> dict:
    try:
        value = json.loads((crate / "verification/std-support/manifest.json").read_bytes())
        require(isinstance(value, dict) and isinstance(value.get("patches"), dict),
                "Std support manifest has no patch inventory")
        return value
    except (OSError, json.JSONDecodeError):
        raise ReuseMiss("cannot read the Std support manifest") from None


def std_package_root(crate: Path, manifest: dict | None = None) -> Path:
    manifest = manifest or std_manifest(crate)
    tool_root = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools")).expanduser()
    package = manifest.get("package", {})
    relative = package.get("private_copy_relative_path")
    require(isinstance(relative, str), "Std manifest omits the private package path")
    return path(tool_root.resolve(), relative).resolve()


def std_tree_sha256(root: Path) -> str:
    require(root.is_dir() and not root.is_symlink(), f"missing canonical Std package: {root}")
    files = []
    for item in root.rglob("*"):
        require(not item.is_symlink(), f"canonical Std package has an unexpected symlink: {item}")
        if item.is_file():
            files.append(item)
    digest_value = hashlib.sha256()
    for item in sorted(files, key=lambda value: value.relative_to(root).as_posix()):
        digest_value.update(item.relative_to(root).as_posix().encode("utf-8"))
        digest_value.update(b"\0")
        digest_value.update(digest(item.read_bytes()).encode("ascii"))
        digest_value.update(b"\n")
    return digest_value.hexdigest()


def purity_toolpatch_identity(crate: Path) -> dict[str, str]:
    repo = crate.parents[1]
    patch = repo / "tools/creusot-toolpatch/patches/bytes-fn-ghost-purity.patch"
    builder = repo / "tools/creusot-toolpatch/scripts/build-bytes-fn-ghost-purity.sh"
    setup = crate / "scripts/setup-bytes-toolchain.sh"
    try:
        patch_sha = digest(patch.read_bytes())
        builder_sha = digest(builder.read_bytes())
        setup_sha = digest(setup.read_bytes())
    except OSError as error:
        raise ReuseMiss(f"Creusot patch/setup input is missing: {error.filename}") from None
    require(patch_sha == PURITY_PATCH_ID["patch_sha256"],
            "tracked FnGhost purity patch differs from the reviewed patch identity")
    return {**PURITY_PATCH_ID, "builder_sha256": builder_sha, "setup_sha256": setup_sha}


def inputs(crate: Path, verify_installed_std: bool = False,
           verify_installed_tool: bool = False) -> dict[str, str]:
    manifest = std_manifest(crate)
    patch_names = manifest["patches"]
    require(all(isinstance(name, str) and PurePosixPath(name).name == name
                for name in patch_names), "Std manifest has an unsafe patch path")
    actual_patches = {item.name for item in (crate / "verification/std-support").glob("*.patch")}
    require(actual_patches == set(patch_names),
            "Std support patch files differ from the canonical manifest inventory")
    fixed = (
        "Cargo.lock", "Cargo.toml", "why3find.json",
        "scripts/check-current-proof.py", "scripts/prepare-proof-std.py",
        "scripts/verify-current.bash", "scripts/setup-bytes-toolchain.sh",
        "verification/std-support/README.md",
        "verification/std-support/manifest.json",
    )
    patch_paths = tuple(f"verification/std-support/{name}" for name in patch_names)
    names = sorted((*fixed, *patch_paths, *(p.relative_to(crate).as_posix()
                              for p in (crate / "src").rglob("*.rs"))))
    try:
        values = {name: digest(path(crate, name).read_bytes()) for name in names}
    except OSError as error:
        raise ReuseMiss(f"proof input is missing: {error.filename}") from None
    expected_tree = manifest.get("candidate_tree", {}).get("sha256")
    require(isinstance(expected_tree, str) and re.fullmatch(r"[0-9a-f]{64}", expected_tree),
            "Std support manifest omits the reviewed candidate tree hash")
    if verify_installed_std:
        external_tree = std_tree_sha256(std_package_root(crate, manifest))
        require(external_tree == expected_tree,
                "canonical private Std package differs from the reviewed overlay tree")
    else:
        external_tree = expected_tree
    values["external_std_package_tree_sha256"] = external_tree
    identity = purity_toolpatch_identity(crate)
    values["creusot_toolpatch_identity_sha256"] = digest(
        json.dumps(identity, sort_keys=True).encode("utf-8"))
    if verify_installed_tool:
        installed_tool_identity(crate, identity)
    return values


def proof_leaves(node: object) -> int:
    if isinstance(node, dict) and "prover" in node:
        require(node.get("prover") in PROVERS and isinstance(node.get("time"), (int, float))
                and node["time"] >= 0, "proof result contains a failed/malformed prover leaf")
        return 1
    if isinstance(node, dict) and isinstance(node.get("children"), list):
        children = node["children"]
        require(bool(children), "proof result contains an empty goal")
        return sum(proof_leaves(child) for child in children)
    raise ValueError("proof result contains an unproved or unknown goal")


def check_result(data: bytes) -> int:
    try:
        goals = json.loads(data)["proofs"]["Coma"]
    except (json.JSONDecodeError, KeyError, TypeError):
        raise ValueError("target has no valid COMA proof result") from None
    require(isinstance(goals, dict) and bool(goals), "target has no proved COMA goals")
    return sum(proof_leaves(goal) for goal in goals.values())


def verify_feature_tree(data: bytes) -> None:
    text = data.decode("utf-8", errors="replace")
    require("creusot-std" in text, "Cargo feature tree omits creusot-std")
    require('bytes feature "default" (command-line)' in text
            and 'bytes feature "std"' in text, "proof did not use default std features")
    require("sc-drf" not in text, "sc-drf must stay disabled")


def target_cone_warnings(crate: Path, log: bytes, comas: list[bytes],
                         extra_roots: tuple[Path, ...] = ()) -> list[str]:
    spans: dict[str, list[tuple[int, int]]] = {}
    roots = (crate.resolve(), *(root.resolve() for root in extra_roots))
    for data in comas:
        for match in SPAN.finditer(data.decode("utf-8", errors="replace")):
            source = Path(match["file"]).resolve()
            for root in roots:
                try:
                    filename = source.relative_to(root).as_posix()
                except ValueError:
                    continue
                spans.setdefault(filename, []).append((int(match["start"]), int(match["end"])))
    warnings = []
    for block in re.split(r"(?m)(?=^warning: )", log.decode("utf-8", errors="replace")):
        if not WARNING.search(block):
            continue
        location = re.search(r"(?:^|[/\s])(src/[\w./-]+\.rs):(\d+):", block)
        if not location:
            continue
        filename, line = location.group(1), int(location.group(2))
        if any(start <= line <= end for start, end in spans.get(filename, [])):
            warnings.append(f"{filename}:{line}: {block.splitlines()[0]}")
    return warnings


def check(crate: Path) -> dict:
    receipt_path = crate / EVIDENCE / "proof.json"
    try:
        receipt = json.loads(receipt_path.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read direct proof receipt: {error}") from None
    require(isinstance(receipt, dict) and receipt.get("schema_version") == 1
            and receipt.get("status") == "proved", "direct proof receipt is missing/malformed")
    if receipt.get("targets") != list(EVIDENCE_TARGETS):
        raise ReuseMiss("saved target inventory differs from the direct runner")
    if receipt.get("verified_inputs_sha256") != inputs(crate):
        raise ReuseMiss("a direct source, contract, configuration, or Std input changed")
    require(receipt.get("configuration") == {
        "crate": "bytes 1.11.1", "target": "x86_64-unknown-linux-gnu",
        "features": ["default", "std"], "rustflags": "--cfg bytes_original_unique_gate",
        "sc_drf": "disabled",
    }, "saved proof used a different direct-source configuration")

    tree = receipt.get("feature_tree", {})
    log = receipt.get("translation_diagnostics", {})
    saved_log = None
    for record, field in ((tree, "feature tree"), (log, "translation log")):
        try:
            data = path(crate, record["file"]).read_bytes()
        except (OSError, KeyError, TypeError):
            raise ReuseMiss(f"saved {field} is missing") from None
        if digest(data) != record.get("sha256"):
            raise ReuseMiss(f"saved {field} changed")
        if field == "feature tree":
            verify_feature_tree(data)
        else:
            saved_log = data

    tool = receipt.get("toolchain", {})
    require(tool.get("rust_toolchain") == "nightly-2026-06-22"
            and tool.get("host") == "x86_64-unknown-linux-gnu"
            and tool.get("pinned_revisions") == PINNED,
            "receipt omits the pinned Creusot/Why3 identities")
    hashes = tool.get("binary_sha256", {})
    names = {"creusot-rustc", "creusot-rustc-fn-ghost-purity", "cargo-creusot", "why3", "why3find",
             "alt-ergo", "z3", "cvc4", "cvc5"}
    require(set(hashes) == names and all(re.fullmatch(r"[0-9a-f]{64}", str(v))
                                         for v in hashes.values()),
            "receipt has incomplete tool binary identities")
    expected_patch = purity_toolpatch_identity(crate)
    require(tool.get("creusot_toolpatch") == expected_patch,
            "receipt does not bind the exact tracked Creusot patch/setup inputs")
    selected = tool.get("selected_compiler", {})
    require(selected.get("name") == expected_patch["compiler_binary"]
            and isinstance(selected.get("path"), str)
            and Path(selected["path"]).is_absolute()
            and selected.get("sha256") == hashes[expected_patch["compiler_binary"]],
            "receipt does not identify the selected patched compiler path and hash")

    std = receipt.get("std_overlay", {})
    manifest = path(crate, "verification/std-support/manifest.json").read_bytes()
    std_manifest = json.loads(manifest)
    require(std == {"manifest_sha256": digest(manifest),
                    "candidate_tree_sha256": std_manifest["candidate_tree"]["sha256"]},
            "receipt omits the current canonical Std overlay identity")

    evidence = receipt.get("target_evidence", {})
    require(set(evidence) == set(EVIDENCE_TARGETS), "receipt omits direct target evidence")
    leaves, coma_blobs = 0, []
    for target in EVIDENCE_TARGETS:
        record = evidence[target]
        require(isinstance(record, dict), f"malformed evidence for {target}")
        expected_coma, expected_proof = evidence_paths(target)
        require(record.get("coma") == expected_coma and record.get("proof") == expected_proof,
                f"unexpected evidence path for {target}")
        for field in ("coma", "proof"):
            relative, expected = record.get(field), record.get(f"{field}_sha256")
            require(isinstance(relative, str) and re.fullmatch(r"[0-9a-f]{64}", str(expected)),
                    f"malformed {field} identity for {target}")
            try:
                data = path(crate, relative).read_bytes()
            except OSError:
                raise ReuseMiss(f"saved target evidence is missing: {relative}") from None
            if digest(data) != expected:
                raise ReuseMiss(f"saved target evidence changed: {relative}")
            if field == "coma":
                require(bool(data.strip()), f"{target} has an empty COMA body")
                coma_blobs.append(data)
            else:
                leaves += check_result(data)
    warnings = target_cone_warnings(crate, saved_log, coma_blobs,
                                    (std_package_root(crate),))
    require(log.get("target_cone_unsupported_calls") == warnings,
            "saved target-cone diagnostic record differs from COMA spans and translation log")
    require(not warnings, "selected targets have unsupported/contractless call diagnostics")
    return {"status": "pass", "mode": "direct-source-reuse", "targets": len(EVIDENCE_TARGETS),
            "prover_leaves": leaves, "solver_execution": "not performed",
            "full_api_coverage": False}


def evidence_paths(target: str) -> tuple[str, str]:
    coma = f"{EVIDENCE}/coma/{target}"
    proof = f"{EVIDENCE}/results/{PurePosixPath(target).with_suffix('')}/proof.json"
    return coma, proof


def installed_tool_identity(crate: Path,
                            expected_patch: dict[str, str] | None = None) -> dict:
    manifest = Path(os.environ.get("BYTES_TOOL_ROOT", "/workspace/bytes-proof-tools")) \
        / "installation-manifest.json"
    try:
        value = json.loads(manifest.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read installed tool identities: {error}") from None
    revisions = {name: value.get(name) for name in PINNED}
    require(revisions == PINNED, "installed tools differ from the pinned setup")
    expected_patch = expected_patch or purity_toolpatch_identity(crate)
    manifest_patch = value.get("creusot_toolpatch")
    require(manifest_patch == expected_patch,
            "installed Creusot tool patch identity differs from tracked setup inputs")
    binaries = value.get("binaries", {})
    selected_name = expected_patch["compiler_binary"]
    try:
        selected_path = Path(binaries[selected_name]["path"]).resolve(strict=True)
        selected_hash = binaries[selected_name]["sha256"]
    except (KeyError, TypeError, OSError):
        raise ValueError("installed manifest omits the selected patched Creusot compiler") from None
    require(re.fullmatch(r"[0-9a-f]{64}", str(selected_hash))
            and digest(selected_path.read_bytes()) == selected_hash,
            "selected patched Creusot compiler hash differs from its installation manifest")
    configured = os.environ.get("CREUSOT_RUSTC")
    require(configured is not None and Path(configured).resolve(strict=True) == selected_path,
            "CREUSOT_RUSTC is not the patched compiler selected by the installation manifest")
    return {"manifest_identity": manifest_patch,
            "selected_compiler": {"name": selected_name, "path": str(selected_path),
                                  "sha256": selected_hash},
            "binaries": binaries}


def tool_identity(crate: Path) -> dict:
    installed = installed_tool_identity(crate)
    binaries = installed["binaries"]
    names = ("creusot-rustc", "creusot-rustc-fn-ghost-purity", "cargo-creusot", "why3", "why3find",
             "alt-ergo", "z3", "cvc4", "cvc5")
    hashes = {name: binaries.get(name, {}).get("sha256") for name in names}
    for name, expected in hashes.items():
        try:
            binary = Path(binaries[name]["path"])
        except (KeyError, TypeError):
            raise ValueError(f"installed tool manifest omits binary: {name}") from None
        require(re.fullmatch(r"[0-9a-f]{64}", str(expected))
                and digest(binary.read_bytes()) == expected,
                f"installed tool binary differs from its pinned identity: {name}")
    rust_toolchain = os.environ.get("RUSTUP_TOOLCHAIN")
    require(rust_toolchain == "nightly-2026-06-22", "unexpected Rust toolchain")
    version = subprocess.run(["rustc", "-vV"], check=True, capture_output=True, text=True).stdout
    host = re.search(r"(?m)^host: (\S+)$", version)
    require(host is not None and host.group(1) == "x86_64-unknown-linux-gnu",
            "proof must target x86_64-unknown-linux-gnu")
    require(os.environ.get("RUSTFLAGS") == "--cfg bytes_original_unique_gate"
            and not os.environ.get("CARGO_ENCODED_RUSTFLAGS")
            and not os.environ.get("CARGO_BUILD_TARGET")
            and not any(name.startswith("CARGO_TARGET_") and name.endswith("_RUSTFLAGS")
                        for name in os.environ),
            "unexpected proof cfg or additional Rust flags")
    return {"rust_toolchain": rust_toolchain, "host": host.group(1),
            "pinned_revisions": PINNED, "binary_sha256": hashes,
            "creusot_toolpatch": installed["manifest_identity"],
            "selected_compiler": installed["selected_compiler"]}


def save(crate: Path, feature_file: Path, log_file: Path, snapshot_file: Path) -> dict:
    try:
        receipt_inputs = json.loads(snapshot_file.read_bytes())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read pre-translation source snapshot: {error}") from None
    require(isinstance(receipt_inputs, dict) and receipt_inputs == inputs(crate, True, True),
            "proof inputs changed during translation or proving; refusing stale evidence")
    feature_data, log_data = feature_file.read_bytes(), log_file.read_bytes()
    verify_feature_tree(feature_data)
    require(bool(log_data), "direct translation log is empty")
    evidence, sources, leaves, coma_blobs = {}, {}, 0, []
    for target in EVIDENCE_TARGETS:
        coma = path(crate, target)
        sources[target] = coma
        result = coma.with_suffix("") / "proof.json"
        try:
            coma_data, proof_data = coma.read_bytes(), result.read_bytes()
        except OSError:
            raise ValueError(f"missing direct output for {target}") from None
        require(bool(coma_data.strip()), f"{target} has an empty COMA body")
        coma_blobs.append(coma_data)
        leaves += check_result(proof_data)
        coma_dst, proof_dst = evidence_paths(target)
        evidence[target] = {"coma": coma_dst, "coma_sha256": digest(coma_data),
                            "proof": proof_dst, "proof_sha256": digest(proof_data)}

    warnings = target_cone_warnings(crate, log_data, coma_blobs,
                                    (std_package_root(crate),))
    require(not warnings, "selected target call cone has unsupported diagnostics: "
            + "; ".join(warnings))
    tool = tool_identity(crate)
    shutil.rmtree(path(crate, f"{EVIDENCE}/coma"), ignore_errors=True)
    shutil.rmtree(path(crate, f"{EVIDENCE}/results"), ignore_errors=True)

    for target, record in evidence.items():
        source = sources[target]
        for src, dst in ((source, record["coma"]),
                         (source.with_suffix("") / "proof.json", record["proof"])):
            output = path(crate, dst)
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, output)
    feature_dst, log_dst = f"{EVIDENCE}/feature-tree.txt", f"{EVIDENCE}/translation.log"
    path(crate, feature_dst).write_bytes(feature_data)
    path(crate, log_dst).write_bytes(log_data)
    receipt = {
        "schema_version": 1, "status": "proved", "verified_inputs_sha256": receipt_inputs,
        "configuration": {"crate": "bytes 1.11.1", "target": "x86_64-unknown-linux-gnu",
                          "features": ["default", "std"], "rustflags": "--cfg bytes_original_unique_gate",
                          "sc_drf": "disabled"},
        "toolchain": tool,
        "std_overlay": {
            "manifest_sha256": digest(path(crate, "verification/std-support/manifest.json").read_bytes()),
            "candidate_tree_sha256": json.loads(path(crate, "verification/std-support/manifest.json").read_bytes())["candidate_tree"]["sha256"],
        },
        "feature_tree": {"file": feature_dst, "sha256": digest(feature_data)},
        "translation_diagnostics": {"file": log_dst, "sha256": digest(log_data),
                                    "target_cone_unsupported_calls": warnings},
        "targets": list(EVIDENCE_TARGETS), "target_evidence": evidence,
    }
    output = crate / EVIDENCE / "proof.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(receipt, indent=2) + "\n")
    return {"status": "saved", "targets": len(EVIDENCE_TARGETS), "prover_leaves": leaves,
            "evidence": output.relative_to(crate).as_posix()}


def main() -> int:
    if sys.argv[1:] == ["--targets"]:
        print("\n".join(TARGETS))
        return 0
    if sys.argv[1:] == ["--std-root"]:
        print(std_package_root(Path(__file__).resolve().parents[1]))
        return 0
    crate = Path(__file__).resolve().parents[1]
    try:
        if sys.argv[1:] == ["--snapshot"]:
            print(json.dumps(inputs(crate, True, True), sort_keys=True))
            return 0
        if len(sys.argv) == 5 and sys.argv[1] == "--save":
            result = save(crate, Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
        elif sys.argv[1:] in ([], ["--check"]):
            result = check(crate)
        else:
            print("usage: check-current-proof.py [--targets | --std-root | --snapshot | --check | --save FEATURE_TREE TRANSLATION_LOG INPUT_SNAPSHOT]",
                  file=sys.stderr)
            return 2
        print(json.dumps(result, indent=2))
        return 0
    except ReuseMiss as error:
        print("proof reuse miss: " + str(error), file=sys.stderr)
    except (OSError, TypeError, ValueError) as error:
        print("proof check failed: " + str(error), file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
