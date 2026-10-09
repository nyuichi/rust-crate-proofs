#!/usr/bin/env python3
"""Rebuild AX's source/native mapping from frozen proof and MIR inputs.

This deliberately emits metadata only: promotion.rs is the selected source
module, and native facts are rederived by check_native.py from the capture.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROBES = ROOT.parent
AV = PROBES / "original-promotable-suffix-promotion-2026-10-09"
PREFIX_SHA = "67b6643e09f88fd4129db990418207aeecf9c81a90d51113b6b8759f513f2e5e"
PROMOTION_SHA = "b1bc6b0c18377e88e5ae3acd130cad6d5cde8f9af48a457bc12b3ac1c87789a8"
AV_ARCHIVE_SHA = "a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489"
AV_RECEIPT_SHA = "a9e2852bfec95740eb2e7a2751e185a198267cc89f4ff829bf4724aba5c40f64"
FEATURES = ("",)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_native():
    spec = importlib.util.spec_from_file_location("ax_native_for_generator", ROOT / "check_native.py")
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_source(feature: str) -> tuple[bytes, bytes, dict[str, str]]:
    source = (ROOT / "src/promotion.rs").read_bytes()
    assert sha(source) == PROMOTION_SHA, "frozen AX promotion source changed"
    ancestor = (AV / "generated/positive.rs").read_bytes()
    assert sha(ancestor) == PREFIX_SHA and source.startswith(ancestor), "AX lost its complete AV prefix"
    suffix = source[len(ancestor):]
    av_src = {p.name: p.read_bytes() for p in (AV / "src").glob("*.rs")}
    ax_src = {p.name: p.read_bytes() for p in (ROOT / "src").glob("*.rs")}
    assert set(av_src) == set(ax_src), "AX source inventory differs from its reviewed AV parent"
    for name, data in av_src.items():
        if name == "promotion.rs":
            continue
        if name == "lib.rs":
            expected = data.replace(b'#[cfg(creusot)] #[path = "../generated/active.rs"] mod promotion;',
                                    b'#[cfg(creusot)] mod promotion;')
            assert expected != data and ax_src[name] == expected, "AX lib route is not the one reviewed module-route change"
        else:
            assert ax_src[name] == data, f"AX changed inherited support source src/{name}"
    source_bindings = {
        "promotion.rs": sha(source),
        "lib.rs": sha(ax_src["lib.rs"]),
        "native.rs": sha((ROOT / "native.rs").read_bytes()),
    }
    return ancestor, suffix, source_bindings


def derive_inherited_targets() -> list[str]:
    receipt_path = AV / "evidence/av-positive-reuse-canonical-v1.json"
    raw = receipt_path.read_bytes()
    assert sha(raw) == AV_RECEIPT_SHA, "published AV canonical receipt changed"
    receipt = json.loads(raw)
    assert receipt.get("status") == "admitted_reuse"
    assert receipt.get("archive_sha256") == AV_ARCHIVE_SHA
    assert receipt.get("statistics") == {"files": 167, "prover": 1683, "null": 0, "structural": 0}
    assert receipt.get("target_policy", {}).get("excluded") == {}
    assert receipt.get("target_policy", {}).get("features") == []
    targets = receipt.get("targets")
    assert isinstance(targets, list) and len(targets) == 167
    prefix = "probe/verif/bytes_original_promotable_suffix_promotion_rlib/"
    relative = []
    for row in targets:
        path = row.get("coma")
        assert isinstance(path, str) and path.startswith(prefix)
        relative.append(path[len(prefix):])
    assert len(set(relative)) == len(relative)
    policy = {
        "schema": "ax-inherited-targets-v1",
        "ancestor": AV.name,
        "ancestor_archive_sha256": AV_ARCHIVE_SHA,
        "ancestor_receipt_sha256": AV_RECEIPT_SHA,
        "source_prefix_sha256": PREFIX_SHA,
        "target_count": len(relative),
        "relative_targets": sorted(relative),
    }
    (ROOT / "inherited-targets.json").write_text(json.dumps(policy, indent=2) + "\n")
    return sorted(relative)


def build(feature: str) -> dict:
    ancestor, suffix, source_bindings = verify_source(feature)
    inherited = derive_inherited_targets()
    native = load_native()
    report = native.audit_bundle(native.load_bundle())
    client = report["native_audit"]["client"]
    production = report["native_audit"]["production"]
    operations = client["operations"]
    source = (ROOT / "src/promotion.rs").read_text()
    callbacks = {
        "from_box": "from_box_scoped",
        "advance": "advance_raw_suffix",
        "advance_mutation": "inc_start_raw_suffix",
        "chunk": "chunk_raw_suffix",
        "slice_projection": "raw_suffix_as_slice",
        "normal_drop": "bytes_raw_suffix_terminal_drop",
        "drop_even": "even_raw_suffix_drop_checked",
        "drop_odd": "odd_raw_suffix_drop_checked",
        "physical_free": "free_raw_suffix_checked",
        "client": "raw_suffix_scope",
    }
    item_hashes = {}
    for name in callbacks.values():
        # Full-source hash is independently pinned; this digest is only an
        # index into a body that the correspondence checker parses again.
        start = source.find(f"fn {name}")
        assert start >= 0, f"missing AX source item {name}"
        item_hashes[name] = sha(source[start:].encode())
    bindings = [
        {"native_operation": "From<Box<[u8]>>::from", "native_block": operations["from_box"]["block"],
         "native_input": operations["from_box"]["input"], "native_result": operations["from_box"]["result"],
         "proof_operation": "from_box_scoped", "proof_input": "input", "proof_result": "value"},
        {"native_operation": "Buf::advance", "native_block": operations["advance"]["block"],
         "native_receiver": operations["advance"]["receiver"], "native_amount": operations["advance"]["amount"],
         "proof_operation": "advance_raw_suffix", "proof_receiver": "value", "proof_amount": "amount",
         "mutation_operation": "inc_start_raw_suffix"},
        {"native_operation": "Buf::chunk", "native_block": operations["chunk"]["block"],
         "native_receiver": operations["chunk"]["receiver"], "native_result": operations["chunk"]["result"],
         "proof_operation": "chunk_raw_suffix", "proof_receiver": "value",
         "read_operation": "raw_suffix_as_slice"},
        {"native_operation": "slice::to_vec", "native_block": operations["to_vec"]["block"],
         "native_result": operations["to_vec"]["result"], "saved_return_before_drop": True,
         "proof_operation": "Vec::to_vec"},
        {"native_operation": "Bytes normal Drop", "native_block": client["normal_edges"][0]["block"],
         "native_place": client["normal_edges"][0]["place"], "native_successor": client["normal_edges"][0]["successor"],
         "proof_operation": "bytes_raw_suffix_terminal_drop", "parity_selector": "immutable_original_base_in_ghost",
         "callbacks": ["even_raw_suffix_drop_checked", "odd_raw_suffix_drop_checked"],
         "physical_free": "free_raw_suffix_checked", "receipt": "physical_projection::FreeReceipt"},
    ]
    capture_bundle = native.load_bundle()
    client_row = next(row for row in capture_bundle["capture"]["selected"] if row["label"] == "client")
    selected_mir = sorted(({"path": row["path"], "sha256": row["sha256"]}
                           for row in capture_bundle["capture"]["selected"]), key=lambda row: row["path"])
    return {
        "schema": "ax-raw-suffix-source-native-map-v1",
        "status": "generated_unchecked",
        "feature": feature,
        "stage": "2-2-004.ElaborateDrops.after.mir",
        "package": "bytes-original-raw-suffix-drop",
        "ancestor": {"probe": AV.name, "active_sha256": PREFIX_SHA,
                     "archive_sha256": AV_ARCHIVE_SHA, "receipt_sha256": AV_RECEIPT_SHA,
                     "source_prefix_sha256": sha(ancestor), "inherited_target_count": len(inherited),
                     "inherited_targets_sha256": sha((ROOT / "inherited-targets.json").read_bytes())},
        "source": {"promotion_sha256": source_bindings["promotion.rs"],
                   "inherited_prefix_bytes": len(ancestor), "appended_bytes_sha256": sha(suffix),
                   "bindings": source_bindings, "item_suffix_hashes": item_hashes},
        "shadow": "src/promotion.rs",
        "native_source": "native.rs",
        "native_source_sha256": client["source_sha256"],
        "native_client_mir": client_row["path"],
        "native_client_mir_sha256": client["mir_sha256"],
        "native_mir_ready": True,
        "client_function": client["function"],
        "debug_places": client["debug_places"],
        "operations": operations,
        "normal_edges": client["normal_edges"],
        "mir_blocks": client["mir_blocks"],
        "normal_drop_order": client["normal_drop_order"],
        "saved_return_precedes_drop": client["saved_return_precedes_drop"],
        "mir": selected_mir,
        "selected_mir": report["capture"]["selected_paths"],
        "selected_mir_count": report["capture"]["selected_mir_count"],
        "production_mir_count": report["capture"]["production_mir_count"],
        "inherited_targets": inherited,
        "production": production,
        "bindings": bindings,
        "mapping_claims": {
            "from_box_raw_constructor": "nonempty Box ownership is detached into one raw owner and one native data AtomicPtr",
            "advance": "same receiver and amount; len guard then len subtraction and pointer add preserve data/vtable",
            "read": "current pointer and remaining length use the physical borrow path",
            "drop": "stored native vtable callback selected by immutable original base parity; callback returns exact base/capacity physical receipt",
            "normal_path": "saved Vec return is evaluated before the sole raw Bytes normal Drop",
        },
        "full_original_admitted": False,
    }


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--feature", default="", choices=FEATURES)
    args = p.parse_args()
    mapping = build(args.feature)
    out = ROOT / "generated"
    out.mkdir(exist_ok=True)
    (out / "mapping.json").write_text(json.dumps(mapping, indent=2) + "\n")
    print(json.dumps({"status": mapping["status"], "feature": args.feature,
                      "native_mir_ready": mapping["native_mir_ready"],
                      "client_mir_sha256": mapping["native_client_mir_sha256"]}))


if __name__ == "__main__":
    main()
