#!/usr/bin/env python3
"""Independent AP proof-shadow, native loop, and source correspondence gate.

This binds the reviewed AO prefix to one runtime-count finite shared-owner
client. It is not a mathematical proof, a general Rust Drop semantics proof,
or full original-crate verification.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
PROBES = ROOT.parent
AO_ROOT = PROBES / "original-promotable-surviving-child-2026-10-09"
ACTIVE = ROOT / "generated/active.rs"
CLIENT = ROOT / "generated/elaborated-client.rs"
EXTENSION = ROOT / "src/finite_extension.rs"
MAPPING = ROOT / "generated/mapping.json"
CLIENT_MIR_REL = "native-mir/bytes_shared_finite_owners_native.finite_shared_scope.2-2-004.ElaborateDrops.after.mir"
CLIENT_MIR = ROOT / CLIENT_MIR_REL

# Verify ancestors before importing them: the imported module performs its own
# pre-import checks for AN, AM, AL, AI and the corresponding native checkers.
AO_CHECKER_SHA256 = "b6d0a002d15c3407891678127659b604e66ece475afe022aa040193e5be21207"
AO_NATIVE_CHECKER_SHA256 = "efbc05ff8643606a822da10dae8eed139679a51e2c67e2613a608c88f6d7506b"
AP_NATIVE_CHECKER_SHA256 = "5c6b0d399114bf3f0d7fd7149573f506faba14043b60c5acb5018f75a16524cc"
AP_NATIVE_CONTROLS_SHA256 = "662ececd9eb1e4057f9fd406ffe4028fcce7024297fa2eaba9a0c5d669e68b5e"
AO_CONTROLS_CHECKER_SHA256 = "b2b1a17ef852ff412513c023b0f873481a172d43132755972a6628e063cf6069"
AO_CONTROLS_MANIFEST_SHA256 = "27b771520746b5597b837a1c0a42f1954a874ab6b92bc8276f614eea0df67d91"
AO_CONTROLS_RECEIPT_SHA256 = "fe6752c3859afcfb52aec033ba1064884257d0790218a205acd07802ed52fc35"
AO_POSITIVE_SHA256 = "e607bc1c5587b3d5e0a8260a3891807db3ade7ab331c264a3e49d1de443ca984"
AP_EXTENSION_SHA256 = "92998cd2ffb7e4fff780c75163394506cf4806a73b23e5f71099adc33bf6cd40"
AP_CLIENT_SHA256 = "f75b08098021b88d72660357d07607200418dad758781e07c30c38821389d84f"
AP_ACTIVE_SHA256 = "cf05c10ecfe38af6ec999a4577a5de5bd5a8bca1d4422c8b94737b2349ef428b"
AP_CLIENT_MIR_SHA256 = "755b695c4410a11881e9b026891d72eace1318e66640a15f597241f3fc1ea94f"
AP_SOURCE_FIXTURE = ROOT / "fixtures/expected-correspondence-sources.json"
AP_SOURCE_FIXTURE_SHA256 = "37e376ac278f8032e0b8fa8fb4b8b1ffc8cc474409be17edf1e4373cae0d3cbc"

class CheckError(RuntimeError):
    pass

def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def function_body(source: str, name: str) -> str:
    """Extract one Rust function body while ignoring braces in comments/strings."""
    masked = AI.mask_noncode(source)
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\b", masked))
    require(len(matches) == 1, f"expected one function body for {name}")
    opening = masked.find("{", matches[0].end())
    require(opening >= 0, f"function body missing for {name}")
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1:index]
    raise CheckError(f"unclosed function body for {name}")

def proof_assert_only_body(masked_body: str) -> tuple[str, int]:
    """Blank proof_assert! statements and return remaining code plus count."""
    result = list(masked_body)
    cursor = count = 0
    pattern = re.compile(r"\bproof_assert\s*!\s*\(")
    while True:
        match = pattern.search(masked_body, cursor)
        if match is None:
            break
        opening = masked_body.find("(", match.start(), match.end())
        depth = 0
        closing = None
        for index in range(opening, len(masked_body)):
            if masked_body[index] == "(":
                depth += 1
            elif masked_body[index] == ")":
                depth -= 1
                if depth == 0:
                    closing = index
                    break
        require(closing is not None, "unterminated proof_assert! in snapshot lemma")
        end = closing + 1
        while end < len(masked_body) and masked_body[end].isspace():
            end += 1
        require(end < len(masked_body) and masked_body[end] == ";",
                "snapshot lemma proof_assert! is not a statement")
        end += 1
        for index in range(match.start(), end):
            if result[index] not in "\r\n":
                result[index] = " "
        cursor = end
        count += 1
    return "".join(result).strip(), count

def import_pinned(name: str, path: pathlib.Path, expected_sha: str) -> Any:
    require(path.is_file() and sha(path.read_bytes()) == expected_sha,
            f"pinned checker changed before import: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot load pinned checker: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module

AO = import_pinned("ap_pinned_ao_correspondence", AO_ROOT / "check_correspondence.py", AO_CHECKER_SHA256)
AI = AO.AN.AI


def load_fixture() -> dict[str, str]:
    raw = AP_SOURCE_FIXTURE.read_bytes()
    require(sha(raw) == AP_SOURCE_FIXTURE_SHA256, "AP reviewed source-token fixture changed")
    fixture = json.loads(raw)
    require(set(fixture) == {"extension", "client", "native_client", "module_route"},
            "AP reviewed source-token fixture fields changed")
    return fixture


def exact_source(actual: str, expected: str, expected_sha: str, label: str) -> None:
    require(sha(actual.encode()) == expected_sha, f"{label} raw source identity changed")
    require(AI.rust_tokens(actual) == AI.rust_tokens(expected),
            f"{label} complete token stream changed")


def assert_ao_published_baseline() -> dict[str, Any]:
    # This replays AO's source/module lineage checks, but deliberately does not
    # call its compiled-input routine because that routine writes into AO's
    # frozen generated directory. The published AO compiled artifacts are
    # separately hash-checked below; the AP artifacts are reconstructed fresh.
    inherited = AO.audit_inherited_an_sources()
    require(sha((AO_ROOT / "check_native.py").read_bytes()) == AO_NATIVE_CHECKER_SHA256,
            "published AO native checker changed")
    control_py = AO_ROOT / "check_checker_controls.py"
    control_manifest = AO_ROOT / "fixtures/checker-controls.json"
    control_receipt = AO_ROOT / "generated/checker-controls-receipt.json"
    require(sha(control_py.read_bytes()) == AO_CONTROLS_CHECKER_SHA256 and
            sha(control_manifest.read_bytes()) == AO_CONTROLS_MANIFEST_SHA256 and
            sha(control_receipt.read_bytes()) == AO_CONTROLS_RECEIPT_SHA256,
            "published AO 31-control source/manifest/receipt changed")
    control_data = json.loads(control_receipt.read_text())
    require(control_data.get("status") == "pass" and control_data.get("control_count") == 31 and
            control_data.get("rejected_as_expected") == 31 and control_data.get("source_mutated_on_disk") is False and
            control_data.get("solver_invoked") is False,
            "published AO structural controls are not a complete immutable 31/31 receipt")
    # Check the four captured AO Cargo artifacts and their receipt without
    # invoking the old writer. AP then independently checks fresh selected
    # Cargo artifacts below.
    ao_receipt_path = AO_ROOT / "generated/compiled-inputs/public-records-build-receipt.json"
    ao_capture = json.loads(ao_receipt_path.read_text())
    ao_inputs = {
        "public_records.rs": "captured_input_sha256",
        "cargo-run-build-fingerprint.json": "captured_cargo_build_fingerprint_sha256",
        "cargo-build-output.txt": "captured_build_output_sha256",
        "cargo-root-output.txt": "captured_root_output_sha256",
    }
    for filename, receipt_key in ao_inputs.items():
        data = (AO_ROOT / "generated/compiled-inputs" / filename).read_bytes()
        require(sha(data) == ao_capture.get(receipt_key), f"published AO compiled input changed: {filename}")
    return {"inherited_source_lineage": inherited,
            "published_main_controls": {"control_count": 31, "rejected_as_expected": 31,
                                      "checker_sha256": AO_CONTROLS_CHECKER_SHA256,
                                      "manifest_sha256": AO_CONTROLS_MANIFEST_SHA256,
                                      "receipt_sha256": AO_CONTROLS_RECEIPT_SHA256},
            "published_compiled_artifacts": {name: ao_capture[key] for name, key in ao_inputs.items()}}


def assert_inherited_ap_sources() -> dict[str, Any]:
    baseline = assert_ao_published_baseline()
    ao_positive = AO_ROOT / "generated/positive.rs"
    require(ao_positive.is_file() and sha(ao_positive.read_bytes()) == AO_POSITIVE_SHA256,
            "published complete AO positive prefix changed")
    prefix = (ROOT / "src/promotion.rs").read_bytes()
    require(prefix == ao_positive.read_bytes() and sha(prefix) == AO_POSITIVE_SHA256,
            "AP source does not start with the byte-exact complete AO positive prefix")
    expected_names = {p.name for p in (AO_ROOT / "src").glob("*.rs")} | {"finite_extension.rs"}
    actual_names = {p.name for p in (ROOT / "src").glob("*.rs")}
    require(actual_names == expected_names, "AP source-module inventory differs from frozen AO plus one reviewed extension")
    for name in sorted(expected_names - {"promotion.rs", "finite_extension.rs"}):
        require((ROOT / "src" / name).read_bytes() == (AO_ROOT / "src" / name).read_bytes(),
                f"AP inherited proof/support module differs from published AO: {name}")
    lib = (ROOT / "src/lib.rs").read_text()
    require(lib == (AO_ROOT / "src/lib.rs").read_text(), "AP module/build route differs from published AO lib.rs")
    support_hashes = AO.AN.assert_imported_support_sources(lib)
    return {"ao_prefix_sha256": AO_POSITIVE_SHA256,
            "inherited_module_count": len(expected_names) - 1,
            "all_inherited_modules_byte_exact": True,
            "lib_route_exact": True,
            "imported_support_sha256": support_hashes,
            "published_ao_audit": baseline}


def assert_extension_source(extension: str, fixture: dict[str, str]) -> dict[str, Any]:
    exact_source(extension, fixture["extension"], AP_EXTENSION_SHA256, "AP finite extension")
    names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(extension))
    expected_names = ["shared_child_clone_checked", "shared_child_clone_registration",
                      "clone_surviving_child", "finite_inventory", "prove_inventory_push",
                      "empty_vec_terminal_drop"]
    require(names == expected_names, "AP extension executable/logic function surface changed")
    attrs = {name: AI.function_outer_attributes(extension, name) for name in expected_names}
    require(attrs["shared_child_clone_registration"] and
            AI.rust_tokens("#[trusted]") in attrs["shared_child_clone_registration"],
            "AP Shared clone registration lost its one explicit generic erasure trust boundary")
    for name in ("shared_child_clone_checked", "clone_surviving_child", "finite_inventory",
                 "prove_inventory_push", "empty_vec_terminal_drop"):
        require(AI.rust_tokens("#[trusted]") not in attrs[name], f"AP body-proved extension item gained trust: {name}")
    require("pointer_event::load_relaxed" in extension and "shallow_clone_arc_checked" in extension and
            "result.0==shared_table()" in extension and "registered3(result.0.clone" in extension,
            "AP clone callback no longer maps to original Shared relaxed-load/ARC callback")
    require("input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data)" in extension and
            "input.inner_logic().0.core.shared as *mut ()" in extension,
            "AP native Shared data field is not bound to the actual source child")
    require("(*scope.observation()).0.contains(id) ==" in extension and
            "owners[i].child_id()!=owners[j].child_id()" in extension and
            "(*scope.observation()).0.len()==owners.len()+1" in extension,
            "AP finite inventory no longer relates all actual Vec owners to the complete live map")
    # This helper consumes only an empty Vec. Its source must not mint a Bytes
    # completion, allocation receipt, or element-free fact other than its exact
    # empty-length requirement.
    helper_attrs = attrs["empty_vec_terminal_drop"]
    require(helper_attrs == [AI.rust_tokens("#[requires(value@.len()==0)]")],
            "empty Vec terminal helper requires/exports an altered source contract")
    helper_body = function_body(extension, "empty_vec_terminal_drop")
    helper_tokens = AI.rust_tokens(helper_body)
    for forbidden in ("Completion", "FreeReceipt", "deallocate", "free_recovered", "Bytes", "physical_projection"):
        require(forbidden not in helper_tokens, f"empty Vec terminal helper gained Bytes effect `{forbidden}`")
    lemma_body = function_body(extension, "prove_inventory_push")
    lemma_masked = AI.mask_noncode(lemma_body)
    lemma_tokens = AI.rust_tokens(lemma_body)
    other_code, proof_count = proof_assert_only_body(lemma_masked)
    require(proof_count > 0 and not other_code,
            "AP inventory induction lemma gained executable control flow or an owner effect")
    for forbidden in ("Ticket", "Perm", "State", "FreeReceipt", "Completion", "deallocate",
                      "free_recovered", "into_inner", "borrow_mut"):
        require(forbidden not in lemma_tokens, f"AP inventory induction lemma gained a resource/effect operation `{forbidden}`")
    return {"sha256": sha(extension.encode()), "function_inventory": names,
            "only_clone_registration_trusted": True,
            "inventory_push_lemma_body_proved_snapshot_only": True,
            "exact_live_map_domain_and_fractions": True,
            "empty_vec_adapter_requires_empty_and_mints_no_bytes_receipt": True,
            "shared_clone_uses_relaxed_pointer_load_and_existing_arc_callback": True}


def assert_client_source(client: str, fixture: dict[str, str]) -> dict[str, Any]:
    exact_source(client, fixture["client"], AP_CLIENT_SHA256, "AP generated proof client")
    names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", AI.mask_noncode(client))
    require(names == ["finite_shared_scope"], "AP generated client has an open executable surface")
    attrs = AI.function_outer_attributes(client, "finite_shared_scope")
    require(attrs == [AI.rust_tokens("#[requires(input@.len()>0)]"),
                      AI.rust_tokens("#[ensures(result@==input@)]")],
            "AP public client pre/postcondition changed or gained a false/trusted premise")
    calls = ["from_box_scoped(input)", "clone_root(&original,scope.borrow_mut())",
             "bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut())",
             "while made<count", "let old_owners=snapshot!(owners@)",
             "let old_scope=snapshot!(detached.inner_logic())",
             "let next=clone_surviving_child(&survivor,detached.borrow_mut())",
             "prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),",
             "owners.push(next)",
             "while let Some(peer)=owners.pop()", "bytes_detached_child_terminal_drop(peer,detached.borrow_mut(),peer_receipt.borrow_mut())",
             "read_surviving_child(&survivor,detached.borrow())", "empty_vec_terminal_drop(owners)",
             "bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut())"]
    pos = [client.find(token) for token in calls]
    require(all(i >= 0 for i in pos) and pos == sorted(pos),
            "AP source client no longer follows root retirement, runtime loops, empty Vec, and final survivor order")
    require("#[variant(count@-made@)]" in client and "#[variant(owners@.len())]" in client and
            "#[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]" in client,
            "AP loop lacks the selected arbitrary-count progress/inventory invariants")
    require("core::mem::forget" not in AI.mask_noncode(client) and "unsafe" not in AI.mask_noncode(client),
            "AP proof client contains a hidden escape or raw operation")
    return {"whole_client_token_stream_pinned": True, "macro_override_excluded": True,
            "finite_count_creation_and_drain_loops_retained": True,
            "all_popped_peers_are_retired_before_the_backedge": True,
            "empty_vec_drop_precedes_final_survivor_drop": True}


def assert_active_composition(prefix: str, extension: str, client: str, active: str) -> dict[str, Any]:
    expected = prefix + "\n" + extension + client
    require(active == expected, "AP active shadow is not the exact AO prefix plus checked extension/client")
    require(sha(active.encode()) == AP_ACTIVE_SHA256, "AP active shadow raw source identity changed")
    require(not re.search(r"\bimpl\s+Drop\s+for\s+Bytes\b", AI.mask_noncode(active)),
            "AP proof shadow introduced a synthetic Bytes Drop implementation")
    return {"active_composition_exact": True, "no_shadow_bytes_drop": True}


def parse_mir_blocks(mir: str) -> dict[tuple[str, bool], str]:
    result: dict[tuple[str, bool], str] = {}
    for match in re.finditer(r"\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}", mir, re.S):
        key = (match.group(1), bool(match.group(2)))
        require(key not in result, f"duplicate native MIR block {key}")
        result[key] = match.group(3)
    require(result, "native MIR block parser found no blocks")
    return result


def block_has(blocks: dict[tuple[str, bool], str], block: str, snippet: str,
              cleanup: bool = False) -> None:
    require((block, cleanup) in blocks and snippet in blocks[(block, cleanup)],
            f"native MIR {block}{' cleanup' if cleanup else ''} lacks expected operation: {snippet}")


def derive_client_mir(mir: str, capture: dict[str, Any]) -> dict[str, Any]:
    # Reconstruct the block-level relation independently of elaborate.py and
    # mapping.json. Then pin the actual per-block hashes, owner Drop edges and
    # the specific cyclic Some/None paths that the generated lowering maps.
    require(sha(mir.encode()) == AP_CLIENT_MIR_SHA256,
            "AP selected native client MIR input changed from its captured identity")
    raw_blocks = list(re.finditer(r"\b(bb\d+)(\s*\(cleanup\))?\s*:\s*\{(.*?)\n    \}", mir, re.S))
    blocks = parse_mir_blocks(mir)
    rows = [{"block": m.group(1), "cleanup": bool(m.group(2)), "body_sha256": sha(m.group(3).encode())}
            for m in raw_blocks]
    require(len(rows) == 51 and len({r["block"] for r in rows}) == 51,
            "AP native MIR must preserve the complete pinned 51-block cyclic/drop-flag graph")
    expected_block_names = {f"bb{i}" for i in range(51)}
    require({name for name, _ in blocks} == expected_block_names,
            "AP native MIR block IDs changed")
    cleanup_ids = {f"bb{i}" for i in list(range(24, 32)) + [33, 34, 37, 39, 41, 42, 45, 47, 48, 49, 50]}
    require({name for name, cleanup in blocks if cleanup} == cleanup_ids and
            {name for name, cleanup in blocks if not cleanup} == expected_block_names - cleanup_ids,
            "AP native MIR cleanup/drop-flag topology changed")
    places = dict(re.findall(r"(?m)^\s*debug\s+(\w+)\s*=>\s*(_\d+)\s*;", mir))
    expected_places = {"input": "_1", "count": "_2", "survivor": "_3", "original": "_4",
                       "owners": "_7", "made": "_8", "peer": "_26", "observed": "_30"}
    require(places == expected_places, "AP native owner/count locals changed")
    for fragment in (
        "_11 = Lt(move _12, move _13);\n        switchInt(move _11) -> [0: bb10, otherwise: bb6];",
        "_16 = <bytes::Bytes as Clone>::clone(move _17) -> [return: bb7, unwind: bb26];",
        "_14 = Vec::<bytes::Bytes>::push(move _15, move _16) -> [return: bb8, unwind: bb25];",
        "_18 = AddWithOverflow(copy _8, const 1_usize);",
        "_8 = move (_18.0: usize);\n        _10 = const ();\n        StorageDead(_11);\n        goto -> bb5;",
        "_23 = Vec::<bytes::Bytes>::pop(move _24) -> [return: bb12, unwind: bb26];",
        "_25 = discriminant(_23);\n        switchInt(move _25) -> [1: bb13, otherwise: bb14];",
        "_26 = move ((_23 as Some).0: bytes::Bytes);\n        _10 = const ();\n        drop(_26) -> [return: bb15, unwind: bb24];",
        "StorageDead(_26);\n        goto -> bb46;",
        "_37 = discriminant(_23);\n        switchInt(move _37) -> [1: bb43, otherwise: bb44];",
        "goto -> bb40;",
        "StorageDead(_23);\n        goto -> bb11;",
        "_22 = const ();\n        StorageDead(_28);\n        goto -> bb38;",
        "_35 = discriminant(_23);\n        switchInt(move _35) -> [1: bb35, otherwise: bb36];",
        "_0 = move _30;\n        goto -> bb20;",
        "drop(_7) -> [return: bb21, unwind: bb27];",
        "drop(_3) -> [return: bb22, unwind: bb30];",
        "StorageDead(_3);\n        goto -> bb23;",
        "return;",
    ):
        require(fragment in mir, "AP native cyclic CFG is missing a required exact operation/edge")
    # Explicitly bind source and order of all normal call/drop edges. The native
    # audit also checks every selected production body, including shared_clone.
    calls = [line.strip() for b in range(24) for line in blocks[(f"bb{b}", False)].splitlines()
             if "-> [return:" in line]
    expected_calls = [
        "_4 = <bytes::Bytes as From<Box<[u8]>>>::from(move _5) -> [return: bb1, unwind: bb29];",
        "_3 = <bytes::Bytes as Clone>::clone(move _6) -> [return: bb2, unwind: bb28];",
        "drop(_4) -> [return: bb3, unwind: bb30];",
        "_7 = Vec::<bytes::Bytes>::new() -> [return: bb4, unwind: bb27];",
        "_16 = <bytes::Bytes as Clone>::clone(move _17) -> [return: bb7, unwind: bb26];",
        "_14 = Vec::<bytes::Bytes>::push(move _15, move _16) -> [return: bb8, unwind: bb25];",
        "_23 = Vec::<bytes::Bytes>::pop(move _24) -> [return: bb12, unwind: bb26];",
        "drop(_26) -> [return: bb15, unwind: bb24];",
        "_32 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _33) -> [return: bb18, unwind: bb26];",
        "_30 = slice::<impl [u8]>::to_vec(move _31) -> [return: bb19, unwind: bb26];",
        "drop(_7) -> [return: bb21, unwind: bb27];",
        "drop(_3) -> [return: bb22, unwind: bb30];",
    ]
    require(calls == expected_calls, "AP native normal call/drop trace changed or a hidden call was inserted")
    drop_edges = []
    for (block, cleanup), body in blocks.items():
        if cleanup:
            continue
        for place, successor, unwind in re.findall(r"drop\((_\d+)\)\s*->\s*\[return:\s*(bb\d+),\s*unwind\s*:?\s*([^\]]+)\]", body):
            owner = next((name for name in ("original", "peer", "owners", "survivor") if places.get(name) == place), None)
            if owner:
                drop_edges.append({"block": block, "place": place, "owner": owner, "successor": successor,
                                   "unwind": unwind.strip(), "repeated": owner == "peer",
                                   "scope": "scope" if owner == "original" else
                                           ("empty_container" if owner == "owners" else "detached")})
    expected_edges = [
        {"block": "bb2", "place": "_4", "owner": "original", "successor": "bb3", "unwind": "bb30", "repeated": False, "scope": "scope"},
        {"block": "bb13", "place": "_26", "owner": "peer", "successor": "bb15", "unwind": "bb24", "repeated": True, "scope": "detached"},
        {"block": "bb20", "place": "_7", "owner": "owners", "successor": "bb21", "unwind": "bb27", "repeated": False, "scope": "empty_container"},
        {"block": "bb21", "place": "_3", "owner": "survivor", "successor": "bb22", "unwind": "bb30", "repeated": False, "scope": "detached"},
    ]
    require(drop_edges == expected_edges, "AP terminal normal Drop edge/place/order map changed")
    loop_regions = {
        "creation_header": "bb5", "creation_backedge": ["bb9", "bb5"],
        "drain_header": "bb11", "pop_branch": "bb12", "some_payload_place": "_26",
        "some_payload_drop": "bb13",
        "some_residual_path": ["bb15", "bb46", ["bb43", "bb44"], "bb40", "bb16", "bb11"],
        "none_residual_path": ["bb14", "bb38", ["bb35", "bb36"], "bb32", "bb17"],
        "interpretation": "cyclic CFG with actual pop-Some payload move; not a finite unrolled trace"}
    require(capture.get("native_source_sha256") == sha((ROOT / "native.rs").read_bytes()) and
            any(x.get("label") == "client" and x.get("path") == CLIENT_MIR_REL and
                x.get("sha256") == AP_CLIENT_MIR_SHA256 for x in capture.get("selected", [])),
            "AP native capture no longer binds the exact source and selected MIR body")
    require(sha(CLIENT_MIR.read_bytes()) == AP_CLIENT_MIR_SHA256,
            "AP captured native client MIR identity changed")
    return {"blocks": rows, "debug_places": places, "normal_edges": drop_edges,
            "normal_calls": calls, "loop_regions": loop_regions,
            "normal_completion": ["return Vec assigned at bb19", "empty owners Vec Drop bb20",
                                  "survivor Bytes Drop bb21", "return bb23"],
            "unwind_proved": False}


def assert_mapping(mapping: dict[str, Any], prefix: str, extension: str, client: str,
                   active: str, capture: dict[str, Any], mir: str) -> dict[str, Any]:
    expected_keys = {"feature", "status", "stage", "base_source", "base_source_sha256",
        "selected_prefix_sha256", "terminal_helpers_sha256", "extension_source", "extension_sha256",
        "client_sha256", "active", "active_sha256", "helpers", "callbacks", "inventory",
        "return_evaluation", "excluded", "tcb", "native_source", "native_source_sha256",
        "native_client_mir", "native_mir_ready", "debug_places", "normal_edges", "mir_blocks",
        "loop_regions", "mir"}
    require(set(mapping) == expected_keys, "AP mapping schema changed")
    require(mapping["feature"] == "" and mapping["status"] == "generated_unchecked" and
            mapping["stage"] == "after-ElaborateDrops", "AP mapping is not the default positive drop-elaborated source")
    require(mapping["base_source"] == "src/promotion.rs" and
            mapping["base_source_sha256"] == sha(prefix.encode()) == AO_POSITIVE_SHA256 and
            mapping["selected_prefix_sha256"] == AO_POSITIVE_SHA256 and
            mapping["terminal_helpers_sha256"] == sha(extension.encode()) and
            mapping["extension_source"] == "src/finite_extension.rs" and
            mapping["extension_sha256"] == sha(extension.encode()) and
            mapping["client_sha256"] == sha(client.encode()) and
            mapping["active"] == "generated/active.rs" and
            mapping["active_sha256"] == sha(active.encode()),
            "AP source-to-generated mapping hashes/routes do not match selected exact inputs")
    require(mapping["helpers"] == ["bytes_root_detaching_terminal_drop", "bytes_detached_child_terminal_drop",
                                   "empty_vec_terminal_drop"] and
            mapping["callbacks"] == ["shared_child_clone_checked", "detaching_root_drop_checked", "child_drop_checked"],
            "AP helper/callback inventory changed")
    require(mapping["inventory"] == "body-proved exact live-map domain and fractions over actual Vec elements plus survivor",
            "AP did not retain the exact actual-Vec live-map inventory claim")
    require(mapping["return_evaluation"] == {"shadow": "let saved_return=observed", "empty_vector_then_survivor": True},
            "AP return and empty-container ordering changed")
    require(mapping["excluded"] == ["unwind completion", "successful completion for all counts",
                                     "arbitrary concurrent closure", "whole crate"],
            "AP exclusions were broadened")
    require(mapping["tcb"] == ["native compiler/MIR and cyclic normal terminal-place elaboration",
        "address nonobservation and no independent Bytes field-drop glue",
        "generic empty Vec element-drop and storage-deallocation semantics",
        "inherited generic pointer/field/physical/erased-callback boundaries"],
        "AP generic compiler/empty-Vec TCB description changed")
    parsed = derive_client_mir(mir, capture)
    require(mapping["debug_places"] == parsed["debug_places"] and
            mapping["normal_edges"] == parsed["normal_edges"] and
            mapping["mir_blocks"] == parsed["blocks"] and
            mapping["loop_regions"] == parsed["loop_regions"],
            "AP generated mapping differs from independently rederived client MIR CFG/Drop effects")
    require(mapping["native_source"] == "native.rs" and
            mapping["native_source_sha256"] == capture.get("native_source_sha256") and
            mapping["native_client_mir"] == CLIENT_MIR_REL and mapping["native_mir_ready"] is True,
            "AP mapping native source/MIR route or capture readiness changed")
    rows = capture.get("selected")
    require(isinstance(rows, list) and len(rows) == 20, "AP native capture must bind its client and 19 production MIR bodies")
    expected_mir = sorted([{"path": row["path"], "sha256": row["sha256"]} for row in rows], key=lambda x: x["path"])
    require(mapping["mir"] == expected_mir, "AP mapping MIR inventory differs from the selected capture files")
    return {"native_cfg_rederived": True, "native_block_hashes": len(parsed["blocks"]),
            "normal_calls": parsed["normal_calls"], "normal_drop_edges": parsed["normal_edges"],
            "loop_regions": parsed["loop_regions"]}


def assert_native_audit(native_module: Any) -> tuple[dict[str, Any], dict[str, Any]]:
    bundle = native_module.load_bundle()
    native = native_module.audit_bundle(bundle)
    require(native.get("status") == "pass", "AP native source/MIR checker rejected its capture")
    client = native.get("client", {})
    expected_creation = {"header": "bb5", "condition": "made (_8) < count (_2)",
        "true_edge": "bb5 -> bb6", "false_edge": "bb5 -> bb10", "clone_source": "survivor (_3)",
        "push_owner": "Vec (_7)", "increment": "bb8/bb9; bb9 -> bb5", "runtime_count_not_unrolled": True}
    expected_drain = {"header": "bb11", "call": "Vec (_7).pop() -> bb12",
        "branch": "bb12 Some -> bb13; None -> bb14", "some_payload_move": "(_23 as Some).0 -> peer (_26)",
        "one_peer_drop": "bb13 drop(_26) -> bb15",
        "backedge": "bb15 -> bb46 -> bb43/bb44 -> bb40 -> bb16 -> bb11",
        "none_exit": "bb14 -> bb38 -> bb35/bb36 -> bb32 -> bb17"}
    require(client.get("native_client_source_closed") is True and client.get("normal_cfg_exact") is True and
            client.get("creation_loop") == expected_creation and client.get("drain_loop") == expected_drain,
            "AP native client audit did not certify the runtime-count loops and actual Some/None paths")
    require(client.get("normal_completion_order") == ["to_vec result moved into return place at bb19",
        "native Vec<Bytes> drop(_7) at bb20", "survivor Bytes drop(_3) at bb21", "return at bb23"],
        "AP native completion order changed")
    container = client.get("container_scope", {})
    require(container.get("native_vec_drop_retained") is True and
            container.get("vector_storage_drop_erased") is False and
            "no Bytes receipt" in container.get("Vec_storage_deallocation", "") and
            "generic Vec/pop element semantics remain Std/compiler TCB" in
                container.get("empty_element_drop_interpretation", ""),
            "AP empty Vec/storage TCB boundary is missing")
    samples = client.get("native_samples", {})
    require(samples.get("lengths") == [1, 2, 31, 256] and
            samples.get("counts") == [0, 1, 2, 7, 31] and
            samples.get("execution_is_corroboration_only") is True,
            "AP native harness samples changed or were promoted into a proof claim")
    mir = native.get("native_mir", {})
    require(mir.get("native_client_mir_order_exact") is True and
            mir.get("selected_mir_count_including_client") == 20 and
            mir.get("selected_production_mir_count") == 19 and
            mir.get("shared_clone_relaxed_load_and_existing_arc_helper_checked") is True,
            "AP native MIR audit failed the selected Shared Relaxed-load callback mapping")
    source = native.get("production_source", {})
    require(source.get("shared_vtable_binds_existing_arc_clone_and_drop_callbacks") is True and
            source.get("shared_fields_and_drop_guard_checked") is True and
            source.get("constructor_clone_cleanup_read_source_exact") is True,
            "AP native production source closure did not bind actual Shared constructor/Clone/Drop route")
    profile = native.get("terminal_field_profile", {})
    require(profile.get("compile_time_no_independent_field_drop_glue") is True,
            "AP native field profile permits independent Bytes field Drop")
    require(native.get("reviewed_production_inputs", {}).get("source_files") == 61 and
            native.get("reviewed_production_inputs", {}).get("manifest_and_lock_pinned") is True and
            native.get("reviewed_production_inputs", {}).get("global_import_macro_and_include_bindings_frozen") is True,
            "AP native audit omitted the reviewed production source/import closure")
    return bundle, native


def assert_probe_manifest(bundle: dict[str, Any], probe_manifest: bytes | None = None,
                          native_manifest_bytes: bytes | None = None) -> dict[str, Any]:
    probe_data = probe_manifest if probe_manifest is not None else (ROOT / "Cargo.toml").read_bytes()
    native_data = native_manifest_bytes if native_manifest_bytes is not None else (ROOT / "native-test/Cargo.toml").read_bytes()
    probe = tomllib.loads(probe_data.decode())
    expected = {"package": {"name": "bytes-original-shared-finite-owners", "version": "0.1.0",
                            "edition": "2021", "publish": False},
                "dependencies": {"creusot-std": "=0.13.0"}, "workspace": {},
                "features": {"negative_missing_acquire": [], "negative_missing_payload_free": [],
                              "negative_missing_control_free": []}}
    require(probe == expected, "AP Cargo manifest/default feature tables or package route changed")
    native_manifest = tomllib.loads(native_data.decode())
    require(native_manifest == {"package": {"name": "bytes-shared-finite-owners-native", "version": "0.0.0",
             "edition": "2021"}, "workspace": {}, "lib": {"name": "bytes_shared_finite_owners_native", "path": "../native.rs"},
             "dependencies": {"bytes": {"path": "../../../../"}}},
             "AP native crate manifest route changed")
    return {"probe_manifest_exact": True, "native_manifest_exact": True,
            "feature_free_default_selected": True}


def assert_compiled_records(bundle: dict[str, Any]) -> dict[str, Any]:
    """Reconstruct AP's production record and bind the real Cargo OUT_DIR files."""
    require(sha((AO_ROOT / "build.rs").read_bytes()) == sha((ROOT / "build.rs").read_bytes()) and
            sha((AO_ROOT / "extract_public.py").read_bytes()) == sha((ROOT / "extract_public.py").read_bytes()),
            "AP inherited build/extractor source differs from published AO")
    rerun = AO.AN.AL.assert_build_script_surface((ROOT / "build.rs").read_bytes())
    AO.AN.AL.assert_extractor_source_surface((ROOT / "extract_public.py").read_bytes())
    expected_paths = ["../../../src/bytes.rs", "../../../src/bytes/bytes_record.rs",
                      "../../../src/bytes/vtable_record.rs", "../../../src/bytes_mut.rs", "extract_public.py"]
    require(rerun == expected_paths, "AP build script dependency/rerun source closure changed")
    production_manifest = tomllib.loads((AO.AN.CRATE_ROOT / "Cargo.toml").read_text())
    require(production_manifest.get("package", {}).get("name") == "bytes" and
            production_manifest.get("package", {}).get("version") == "1.11.1" and
            production_manifest.get("lib", {}).get("path") == "src/lib.rs",
            "resolved production package/target is not bytes 1.11.1")
    inputs = {
        "../../../src/bytes.rs": AO.AN.CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": AO.AN.CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": AO.AN.CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": AO.AN.CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": ROOT / "extract_public.py",
    }
    source_hashes = {}
    for literal, path in inputs.items():
        resolved = (ROOT / literal).resolve()
        require(resolved == path.resolve() and resolved.is_file(), f"AP build input resolves elsewhere: {literal}")
        source_hashes[literal] = sha(resolved.read_bytes())
    records = (AO.AN.CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtables = (AO.AN.CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    mutable = (AO.AN.CRATE_ROOT / "src/bytes_mut.rs").read_text()
    AO.AN.AI.audit_generated_extractions(bundle["production_source"], ROOT / "generated", records, vtables, mutable)
    shared = AI.extract_struct_source(mutable, "struct Shared {", "BytesMut::Shared")
    bytesmut = AI.extract_struct_source(mutable, "pub struct BytesMut {", "BytesMut")
    expected = (records + "\n" + vtables + "\nmod mutable_record {\nuse alloc::vec::Vec;\n"
                "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" + shared + "\n" + bytesmut + "\n}\n"
                "use mutable_record::BytesMut;\n").encode()
    generated_path = ROOT / "generated/public_records.rs"
    source_map_path = ROOT / "generated/source-map.json"
    generated = generated_path.read_bytes()
    source_map = json.loads(source_map_path.read_text())
    require(generated == expected and source_map.get("generated/public_records.rs", {}).get("sha256") == sha(generated),
            "AP translated public record bytes do not reconstruct from reviewed production source")

    target_value = os.environ.get("CARGO_TARGET_DIR")
    target = pathlib.Path(target_value).resolve() if target_value else pathlib.Path("/workspace/bytes-proof-tools/targets/bytes")
    fingerprint_paths = list((target / "debug/.fingerprint").glob(
        "bytes-original-shared-finite-owners-*/run-build-script-build-script-build.json"))
    require(len(fingerprint_paths) == 1, "AP requires exactly one selected Cargo build fingerprint")
    fingerprint_path = fingerprint_paths[0]
    fingerprint_bytes = fingerprint_path.read_bytes()
    fingerprint = json.loads(fingerprint_bytes)
    rerun_rows = [x["RerunIfChanged"] for x in fingerprint.get("local", [])
                  if isinstance(x, dict) and "RerunIfChanged" in x]
    require(len(rerun_rows) == 1 and rerun_rows[0].get("paths") == expected_paths,
            "AP actual Cargo fingerprint does not record the reviewed build/extractor dependencies")
    output_rel = pathlib.Path(rerun_rows[0].get("output", ""))
    require(not output_rel.is_absolute() and output_rel.parts[:2] == ("debug", "build"),
            "AP Cargo build-output path escaped the selected target directory")
    output = (target / output_rel).resolve()
    out_dir = output.parent
    require(fingerprint_path.parent.name == out_dir.name and output.is_file(),
            "AP fingerprint does not bind its sibling Cargo build output")
    directives = (["cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
                   "cargo:rustc-cfg=bytes_original_shared_gate"] +
                  ["cargo:rerun-if-changed=" + p for p in expected_paths])
    output_bytes = output.read_bytes()
    require(output_bytes == ("\n".join(directives) + "\n").encode(),
            "AP Cargo build directives/configuration changed")
    root_output = out_dir / "root-output"
    compiled = out_dir / "out/public_records.rs"
    root_bytes = root_output.read_bytes()
    compiled_bytes = compiled.read_bytes()
    require(root_bytes == str(out_dir / "out").encode() and compiled_bytes == expected == generated,
            "AP actual Cargo OUT_DIR output differs from exact source reconstruction")
    capture_dir = ROOT / "generated/compiled-inputs"
    capture_dir.mkdir(parents=True, exist_ok=True)
    artifacts = {"public_records.rs": compiled_bytes,
        "cargo-run-build-fingerprint.json": fingerprint_bytes,
        "cargo-build-output.txt": output_bytes,
        "cargo-root-output.txt": root_bytes}
    for name, data in artifacts.items():
        (capture_dir / name).write_bytes(data)
    receipt = {"status": "pass", "probe_package": "bytes-original-shared-finite-owners",
        "build_script_sha256": sha((ROOT / "build.rs").read_bytes()),
        "extractor_sha256": sha((ROOT / "extract_public.py").read_bytes()),
        "cargo_target_dir": str(target), "cargo_build_fingerprint": str(fingerprint_path),
        "cargo_build_fingerprint_sha256": sha(fingerprint_bytes),
        "captured_cargo_build_fingerprint_path": "generated/compiled-inputs/cargo-run-build-fingerprint.json",
        "captured_cargo_build_fingerprint_sha256": sha(fingerprint_bytes),
        "build_output_path": str(output), "build_output_sha256": sha(output_bytes),
        "captured_build_output_path": "generated/compiled-inputs/cargo-build-output.txt",
        "captured_build_output_sha256": sha(output_bytes),
        "root_output_path": str(root_output), "root_output_sha256": sha(root_bytes),
        "captured_root_output_path": "generated/compiled-inputs/cargo-root-output.txt",
        "captured_root_output_sha256": sha(root_bytes),
        "actual_out_dir": str(compiled.parent), "compiled_input_path": str(compiled),
        "compiled_input_sha256": sha(compiled_bytes),
        "captured_input_path": "generated/compiled-inputs/public_records.rs",
        "captured_input_sha256": sha(compiled_bytes), "reconstructed_generated_sha256": sha(expected),
        "source_map_sha256": sha(source_map_path.read_bytes()), "production_rerun_input_sha256": source_hashes}
    (capture_dir / "public-records-build-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return {"actual_OUT_DIR_record_reconstructed": True, "captured_actual_build_artifact_count": 4,
            "receipt": receipt}


def audit(inputs: dict[str, Any] | None = None) -> dict[str, Any]:
    fixture = load_fixture()
    source = inputs if inputs is not None else {
        "prefix": (ROOT / "src/promotion.rs").read_text(),
        "extension": EXTENSION.read_text(),
        "client": CLIENT.read_text(),
        "helper": (ROOT / "generated/finite-extension.rs").read_text(),
        "terminal_helper": (ROOT / "generated/terminal-helper.rs").read_text(),
        "active": ACTIVE.read_text(), "lib": (ROOT / "src/lib.rs").read_text(),
        "native_source": (ROOT / "native.rs").read_text(),
        "mir": CLIENT_MIR.read_text(), "mapping": json.loads(MAPPING.read_text()),
        "capture": json.loads((ROOT / "native-mir/capture.json").read_text()),
    }
    inherited = assert_inherited_ap_sources()
    require(source["lib"] == fixture["module_route"], "AP live module route differs from frozen expected route")
    require(AI.rust_tokens(source["native_source"]) == AI.rust_tokens(fixture["native_client"]),
            "AP native outer client source differs from its full-source fixture")
    extension = assert_extension_source(source["extension"], fixture)
    client = assert_client_source(source["client"], fixture)
    require(source["helper"] == source["extension"] and source["terminal_helper"] == source["extension"],
            "AP generated extension/terminal helper differs from the exact source module")
    active = assert_active_composition(source["prefix"], source["extension"], source["client"], source["active"])
    native_module = import_pinned("ap_pinned_native_correspondence", ROOT / "check_native.py",
                                  AP_NATIVE_CHECKER_SHA256)
    bundle, native = assert_native_audit(native_module)
    mapping = assert_mapping(source["mapping"], source["prefix"], source["extension"], source["client"],
                             source["active"], source["capture"], source["mir"])
    require(native["client"].get("normal_call_and_drop_edges") == mapping["normal_calls"],
            "AP native checker and independent MIR parser disagree on exact normal call/drop events")
    require(source["mapping"] == json.loads((ROOT / "generated/mapping.json").read_text()),
            "AP mapping changed between selected live source and audit input")
    require(native["client"]["creation_loop"]["runtime_count_not_unrolled"] is True and
            native["client"]["drain_loop"]["some_payload_move"] == "(_23 as Some).0 -> peer (_26)",
            "AP proof/native mapping does not preserve runtime count and Some ownership transfer")
    manifest = assert_probe_manifest(bundle)
    compiled = assert_compiled_records(bundle)
    return {"status": "pass", "checker_scope": "AP actual native source/MIR and exact proof-shadow correspondence for one runtime-count finite Shared Vec client",
        "ancestor_pins": {"AO_checker_sha256": AO_CHECKER_SHA256,
                          "AP_native_checker_sha256": AP_NATIVE_CHECKER_SHA256,
                          "ao": inherited["published_ao_audit"]},
        "inherited_sources": inherited, "extension": extension, "proof_client": client,
        "active_composition": active, "module_route": {"selected": "../generated/active.rs", "lib_exact_to_AO": True},
        "native_audit": native, "mapping_reconstruction": mapping,
        "manifest": manifest, "compiled_inputs": compiled,
        "limits": ["No Creusot/Why3 proof claim.", "The generic cyclic MIR/Drop elaborator and Rust/Std Vec empty-element semantics remain TCB.",
                   "Vec storage allocation/deallocation is native Std/allocator behavior; Bytes receipts do not cover it.",
                   "Normal completion only; allocation failure, unwind, concurrency, arbitrary escaping owners, and full API/configuration admission are excluded."]}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", type=pathlib.Path, default=ACTIVE)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAPPING)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.shadow.resolve() == ACTIVE.resolve(), "shadow override must select the live generated/active.rs route")
        require(args.mapping.resolve() == MAPPING.resolve(), "mapping override must select the live generated/mapping.json route")
        result = audit()
    except Exception as exc:
        result = {"status": "reject", "checker_scope": "AP finite-owner source/native correspondence",
                  "reason": f"{type(exc).__name__}: {exc}"}
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0 if result.get("status") == "pass" else 1

if __name__ == "__main__":
    raise SystemExit(main())
