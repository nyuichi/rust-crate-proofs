#!/usr/bin/env python3
"""Fail-closed correspondence audit for the actual normal-edge Bytes Drop witness.

The audit checks a closed native client, pinned post-ElaborateDrops MIR, the
selected production Drop/vtable/AtomicMut source, and an AI-shadow prefix plus
terminal-place proof helper. It does not prove generic MIR/codegen adequacy,
unwind equivalence, or the trusted generic cursor/receipt TCB.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import pathlib
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
AI_PROBE = ROOT.parent / "original-shared-scoped-client-2026-10-09"
AI_CHECKER_PATH = AI_PROBE / "check_correspondence.py"
AI_CHECKER_SHA256 = "decb502490fdd1c534e02b5ac0798b070eeb8ed7be4d6aa1779e05cee5e08bb7"
AI_HELPER_FIXTURE_SHA256 = "c4903a139f0924fc26a62c8319617db1f34cfbb876e7e775abba0f70130d26e1"
NATIVE_SOURCE = ROOT / "native.rs"
MIR_DIR = ROOT / "native-mir"
GENERATED = ROOT / "generated"
# `active.rs` and `mapping.json` are mutable generator outputs used by the
# proof-control runner. The positive checker default is the immutable copy
# captured before semantic mutation controls are applied.
ACTIVE_SHADOW = GENERATED / "positive.rs"
MAP_DEFAULT = GENERATED / "positive-mapping.json"
EXPECTED_MIR = ROOT / "fixtures" / "expected-native-mir-tokens.json"
EXPECTED_SOURCE = ROOT / "fixtures" / "expected-native-drop-source-tokens.json"
EXPECTED_TERMINAL_SHADOW = ROOT / "fixtures" / "expected-terminal-shadow-tokens.json"
EXPECTED_NATIVE_PROVENANCE = ROOT / "fixtures" / "expected-native-provenance.json"
NATIVE_HARNESS = GENERATED / "native-harness"

MIR_FILENAMES = {
    "scope": "bytes_automatic_drop_native.shared_automatic_scope.2-2-004.ElaborateDrops.after.mir",
    "bytes_drop": "bytes.bytes-{impl#3}-drop.2-2-004.ElaborateDrops.after.mir",
    "shared_drop": "bytes.bytes-shared_drop.2-2-004.ElaborateDrops.after.mir",
    "shared_drop_closure": "bytes.bytes-shared_drop-{closure#0}.2-2-004.ElaborateDrops.after.mir",
    "release_shared": "bytes.bytes-release_shared.2-2-004.ElaborateDrops.after.mir",
    "free_shared": "bytes.bytes-free_shared.2-2-004.ElaborateDrops.after.mir",
    "atomicmut_with_mut": "bytes.loom-sync-atomic-{impl#0}-with_mut.2-2-004.ElaborateDrops.after.mir",
    "shared_record_drop": "bytes.bytes-{impl#49}-drop.2-2-004.ElaborateDrops.after.mir",
}

EXPECTED_NATIVE_SOURCE = r'''use bytes::Bytes;
/// Native normal-return scope: no explicit cleanup or mem::drop call.
#[cfg_attr(creusot, requires(input@.len() < creusot_std::std::vec::capacity_model(input)))]
#[cfg_attr(creusot, ensures(result@ == input@))]
pub fn shared_automatic_scope(input: Vec<u8>) -> Vec<u8> {
    let first = Bytes::from(input);
    let second = first.clone();
    let observed = AsRef::<[u8]>::as_ref(&second).to_vec();
    observed
}'''


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_ai_checker():
    require(AI_CHECKER_PATH.is_file(), f"missing audited AI checker dependency: {AI_CHECKER_PATH}")
    checker_hash = sha(AI_CHECKER_PATH.read_bytes())
    require(checker_hash == AI_CHECKER_SHA256,
            "the reused AI source checker differs from its audited frozen version")
    helper_fixture = AI_PROBE / "fixtures" / "expected-shadow-bodies.json"
    require(helper_fixture.is_file() and sha(helper_fixture.read_bytes()) == AI_HELPER_FIXTURE_SHA256,
            "the reused AI checker helper-pattern fixture differs from its audited version")
    spec = importlib.util.spec_from_file_location("ai_shared_client_checker", AI_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load audited AI checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def extract_item(tokens: list[str], prefix: str, label: str) -> list[str]:
    prefix_tokens = AI.rust_tokens(prefix)
    hits = [i for i in range(len(tokens) - len(prefix_tokens) + 1)
            if tokens[i:i + len(prefix_tokens)] == prefix_tokens]
    require(len(hits) == 1, f"expected one `{label}` source item, found {len(hits)}")
    start = hits[0]
    brace = (start + len(prefix_tokens) - 1
             if prefix_tokens and prefix_tokens[-1] == "{" else
             next((i for i in range(start + len(prefix_tokens), len(tokens))
                   if tokens[i] == "{"), None))
    require(brace is not None, f"`{label}` source item has no body")
    depth = 0
    for i in range(brace, len(tokens)):
        if tokens[i] == "{":
            depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[start:i + 1]
    raise AuditError(f"unclosed `{label}` source item")


def find_block(tokens: list[str], label: str, source_name: str) -> list[str]:
    hits: list[int] = []
    for i, token in enumerate(tokens):
        if token != label:
            continue
        cursor = i + 1
        if tokens[cursor:cursor + 3] == ["(", "cleanup", ")"]:
            cursor += 3
        if tokens[cursor:cursor + 2] == [":", "{"]:
            hits.append(cursor + 1)
    require(len(hits) == 1, f"MIR `{source_name}` must contain exactly one basic block `{label}`")
    brace = hits[0]
    depth = 0
    for i in range(brace, len(tokens)):
        if tokens[i] == "{":
            depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[brace + 1:i]
    raise AuditError(f"MIR block `{label}` in `{source_name}` is unclosed")


def expect_block(mir: str, label: str, exact: str, source_name: str) -> None:
    actual = find_block(AI.rust_tokens(mir), label, source_name)
    expected = AI.rust_tokens(exact)
    require(actual == expected, f"MIR `{source_name}` block `{label}` changed")


def contains_tokens_once(tokens: list[str], snippet: str, label: str) -> None:
    needle = AI.rust_tokens(snippet)
    count = sum(tokens[i:i + len(needle)] == needle
                for i in range(len(tokens) - len(needle) + 1))
    require(count == 1, f"expected one `{label}` MIR pattern, found {count}")


def audit_native_source(source: str) -> dict[str, Any]:
    require(AI.rust_tokens(source) == AI.rust_tokens(EXPECTED_NATIVE_SOURCE),
            "native client is outside the fixed two-owner automatic-Drop witness")
    tokens = AI.rust_tokens(source)
    require("cleanup" not in tokens and "drop" not in tokens,
            "native client must leave both Bytes handles for ordinary scope Drop")
    require(tokens.count("clone") == 1 and tokens.count("from") == 1,
            "native client must contain exactly one Clone and one From construction")
    return {
        "source_sha256": sha(source.encode()),
        "entrypoint": "shared_automatic_scope",
        "client_protocol": ["From(first)", "Clone(second)", "AsRef(second)", "to_vec", "normal return"],
        "explicit_cleanup_or_drop_calls": 0,
        "closed_source": True,
    }


def audit_native_provenance_data(*, rustc_text: str, cargo_text: str,
                                 manifest_text: str, lock_text: str,
                                 harness_source: str, module_source: str,
                                 native_run_log: str, ai: Any) -> dict[str, Any]:
    expected = json.loads(EXPECTED_NATIVE_PROVENANCE.read_text())
    require(rustc_text == expected["rustc_version_text"],
            "recorded rustc version/host differs from the reviewed compiler used for MIR capture")
    require(cargo_text == expected["cargo_version_text"],
            "recorded cargo version differs from the reviewed native harness/MIR toolchain")

    manifest = tomllib.loads(manifest_text)
    expected_manifest = {
        "package": {"name": "bytes-automatic-drop-native", "version": "0.0.0", "edition": "2021"},
        "workspace": {},
        "lib": {"name": "bytes_automatic_drop_native", "path": str(NATIVE_SOURCE.resolve())},
        "dependencies": {"bytes": {"path": str(CRATE_ROOT.resolve())}},
    }
    require(manifest == expected_manifest,
            "native harness manifest changed package identity, selected native.rs, dependency path, or feature surface")

    lock = tomllib.loads(lock_text)
    expected_lock = expected["native_harness_lock"]
    normalized_packages = [
        {key: package[key] for key in ("name", "version", "source", "checksum", "dependencies")
         if key in package}
        for package in lock.get("package", [])
    ]
    require(lock.get("version") == expected_lock["version"] and
            normalized_packages == expected_lock["packages"],
            "native harness Cargo.lock package identities, pinned versions, dependencies, or registry checksums changed")
    require(any(package["name"] == "bytes" and package["version"] == "1.11.1"
                for package in normalized_packages) and
            any(package["name"] == "creusot-std" and package["version"] == "0.13.0"
                for package in normalized_packages),
            "native harness lock does not pin bytes 1.11.1 and creusot-std 0.13.0")

    harness_tokens = ai.rust_tokens(harness_source)
    require(harness_tokens == expected["native_harness_source_tokens"].split(),
            "native harness source changed input coverage, call target, profile witness, or execution shape")
    no_drop_profile = (
        "const _: [(); 0] = [(); core::mem::needs_drop::<("
        "*const u8, usize, core::sync::atomic::AtomicPtr<()>, &'static ()"
        ")>() as usize];"
    )
    contains_tokens_once(harness_tokens, no_drop_profile,
                         "compiled Bytes field-category no-drop witness")
    module_tokens = ai.rust_tokens(module_source)
    require(module_tokens == expected["native_harness_module_route_tokens"].split(),
            "proof crate root changed the exact module/import surface used by Creusot")
    for module_name, expected_path in expected["support_module_paths"].items():
        try:
            actual_path = ai.rust_path_for_module(module_source, module_name)
        except Exception as exc:
            raise AuditError(f"cannot resolve selected support module `{module_name}` path") from exc
        require(actual_path == expected_path,
                f"proof crate support module `{module_name}` changed its selected source path")
    route = expected["active_module_route_line"]
    require(module_source.splitlines().count(route) == 1 and
            module_tokens.count("public_shared") == 1 and
            ai.rust_tokens("#[cfg(creusot)] #[path=\"../generated/active.rs\"] mod public_shared;") ==
            ai.rust_tokens(route),
            "src/lib.rs must select exactly generated/active.rs for the sole Creusot public_shared module")

    run_line = expected["native_run_success_line"]
    require(native_run_log.rstrip().endswith(run_line) and "Finished `dev` profile" in native_run_log and
            "Running `" in native_run_log,
            "native harness log does not record the checked-in five-input execution")
    return {
        "rustc_version_sha256": sha(rustc_text.encode()),
        "cargo_version_sha256": sha(cargo_text.encode()),
        "native_manifest_sha256": sha(manifest_text.encode()),
        "native_lock_sha256": sha(lock_text.encode()),
        "native_harness_source_sha256": sha(harness_source.encode()),
        "proof_crate_root_sha256": sha(module_source.encode()),
        "native_run_log_sha256": sha(native_run_log.encode()),
        "selected_native_library": str(NATIVE_SOURCE.resolve()),
        "selected_production_dependency": str(CRATE_ROOT.resolve()),
        "native_field_profile_no_drop_witness": True,
        "native_harness_inputs": [0, 1, 7, 63, 1024],
        "creusot_module_route": "src/lib.rs -> generated/active.rs",
        "toolchain_and_dependency_identity_pinned": True,
        "native_run_is_correspondence_corroboration_only": True,
    }


def audit_native_provenance(ai: Any) -> dict[str, Any]:
    return audit_native_provenance_data(
        rustc_text=(MIR_DIR / "rustc-version.txt").read_text(),
        cargo_text=(MIR_DIR / "cargo-version.txt").read_text(),
        manifest_text=(NATIVE_HARNESS / "Cargo.toml").read_text(),
        lock_text=(NATIVE_HARNESS / "Cargo.lock").read_text(),
        harness_source=(NATIVE_HARNESS / "src/main.rs").read_text(),
        module_source=(ROOT / "src/lib.rs").read_text(),
        native_run_log=(GENERATED / "native-run.log").read_text(),
        ai=ai,
    )


def audit_production_drop_sources(ai: Any, bytes_source: str, refcount_source: str,
                                  shared_record_source: str, bytes_record_source: str,
                                  vtable_record_source: str, loom_source: str) -> dict[str, Any]:
    chain = ai.audit_production_chain(bytes_source, refcount_source, shared_record_source)
    expected = json.loads(EXPECTED_SOURCE.read_text())["items"]
    bytes_tokens = ai.rust_tokens(bytes_source)
    bytes_impl = extract_item(bytes_tokens, "impl Drop for Bytes {", "Bytes Drop")
    require(bytes_impl == expected["impl_drop_bytes"].split(),
            "actual production `impl Drop for Bytes` differs from the reviewed vtable-drop body")
    record_tokens = ai.rust_tokens(bytes_record_source)
    bytes_record = extract_item(record_tokens, "pub struct Bytes {", "Bytes record")
    require(bytes_record == expected["bytes_record"].split(),
            "Bytes native field profile changed; independent field drop glue is not admitted")
    vtable_tokens = ai.rust_tokens(vtable_record_source)
    vtable_record = extract_item(vtable_tokens, "pub(crate) struct Vtable {", "Vtable record")
    require(vtable_record == expected["vtable_record"].split(),
            "production Vtable field order/signatures changed; MIR drop-slot mapping is not admitted")
    require(ai.rust_tokens(loom_source) == expected["loom_native_atomicmut"].split(),
            "default native loom AtomicMut alias/body changed from `f(self.get_mut())`")
    atomic_mut = ai.rust_tokens(loom_source)
    require("f" in atomic_mut and ai.rust_tokens("self . get_mut ( )") ==
            ["self", ".", "get_mut", "(", ")"] and
            atomic_mut.count("get_mut") == 1,
            "selected AtomicMut implementation must expose only the mutable pointer value")
    return {
        "ai_native_chain": chain,
        "bytes_drop_body_exact": True,
        "bytes_native_fields_exact": True,
        "loom_native_atomicmut_exact": True,
        "drop_body_operation": "(self.vtable.drop)(&mut self.data, self.ptr, self.len)",
        "selected_with_mut_operation": "f(self.get_mut())",
        "selected_shared_drop_closure_source": "checked through the exact SHARED_VTABLE -> shared_drop -> release_shared source chain",
    }


def audit_mir_sources(mir_sources: dict[str, str]) -> dict[str, Any]:
    expected = json.loads(EXPECTED_MIR.read_text())["files"]
    require(set(expected) == set(MIR_FILENAMES.values()),
            "pinned MIR fixture file set changed")
    texts: dict[str, str] = {}
    hashes: dict[str, str] = {}
    tokens: dict[str, list[str]] = {}
    for key, filename in MIR_FILENAMES.items():
        require(filename in mir_sources,
                f"missing post-ElaborateDrops MIR input `{filename}`")
        source = mir_sources[filename]
        tok = AI.rust_tokens(source)
        require(tok == expected[filename].split(),
                f"post-ElaborateDrops MIR `{filename}` differs from reviewed token snapshot")
        texts[key], tokens[key], hashes[filename] = source, tok, sha(source.encode())

    scope_file = MIR_FILENAMES["scope"]
    scope = tokens["scope"]
    for snippet, label in (
        ("debug first => _2;", "first local mapping"),
        ("debug second => _4;", "second local mapping"),
        ("debug observed => _6;", "owned output local mapping"),
    ):
        contains_tokens_once(scope, snippet, f"scope {label}")
    expect_block(texts["scope"], "bb4",
                 "StorageDead(_7); StorageDead(_10); StorageDead(_8); "
                 "_0 = move _6; goto -> bb5;", scope_file)
    expect_block(texts["scope"], "bb5",
                 "StorageDead(_6); drop(_4) -> [return: bb6, unwind: bb10];", scope_file)
    expect_block(texts["scope"], "bb6",
                 "StorageDead(_4); drop(_2) -> [return: bb7, unwind: bb12];", scope_file)
    expect_block(texts["scope"], "bb7", "StorageDead(_2); goto -> bb8;", scope_file)
    expect_block(texts["scope"], "bb8", "return;", scope_file)
    expect_block(texts["scope"], "bb9",
                 "drop(_4) -> [return: bb10, unwind terminate(cleanup)];", scope_file)
    expect_block(texts["scope"], "bb10",
                 "drop(_2) -> [return: bb12, unwind terminate(cleanup)];", scope_file)
    expect_block(texts["scope"], "bb11", "goto -> bb12;", scope_file)
    expect_block(texts["scope"], "bb12", "goto -> bb13;", scope_file)
    expect_block(texts["scope"], "bb13", "resume;", scope_file)

    bytes_drop_file = MIR_FILENAMES["bytes_drop"]
    contains_tokens_once(tokens["bytes_drop"],
        "_2 = copy ((*_7).4: for<'a> unsafe fn(&'a mut core::sync::atomic::Atomic<*mut ()>, *const u8, usize));",
        "actual Bytes Drop vtable slot load")
    contains_tokens_once(tokens["bytes_drop"],
        "_0 = move _2(move _3, move _5, move _6) -> [return: bb1, unwind continue];",
        "actual Bytes Drop callback arguments")
    expect_block(texts["bytes_drop"], "bb0",
                 "StorageLive(_2); _7 = deref_copy ((*_1).3: &bytes::Vtable); "
                 "_2 = copy ((*_7).4: for<'a> unsafe fn(&'a mut core::sync::atomic::Atomic<*mut ()>, *const u8, usize)); "
                 "StorageLive(_3); StorageLive(_4); _4 = &'_ mut ((*_1).2: core::sync::atomic::Atomic<*mut ()>); "
                 "_3 = &'_ mut (*_4); StorageLive(_5); _5 = copy ((*_1).0: *const u8); "
                 "StorageLive(_6); _6 = copy ((*_1).1: usize); "
                 "_0 = move _2(move _3, move _5, move _6) -> [return: bb1, unwind continue];",
                 bytes_drop_file)

    shared_drop_file = MIR_FILENAMES["shared_drop"]
    closure_file = MIR_FILENAMES["shared_drop_closure"]
    expect_block(texts["shared_drop"], "bb0",
                 "StorageLive(_4); StorageLive(_5); _5 = &'_ mut (*_1); StorageLive(_6); "
                 "_6 = {closure@src/bytes.rs:1604:19: 1604:27}; "
                 "_4 = <Atomic<*mut ()> as AtomicMut<()>>::with_mut::<{closure@src/bytes.rs:1604:19: 1604:27}, ()>(move _5, move _6) -> [return: bb1, unwind continue];",
                 shared_drop_file)
    expect_block(texts["shared_drop_closure"], "bb0",
                 "StorageLive(_3); StorageLive(_4); StorageLive(_5); _5 = copy (*_2); "
                 "_4 = core::ptr::mut_ptr::<impl *mut ()>::cast::<bytes::Shared>(move _5) -> [return: bb1, unwind continue];",
                 closure_file)
    expect_block(texts["shared_drop_closure"], "bb1",
                 "StorageDead(_5); _3 = bytes::release_shared(move _4) -> [return: bb2, unwind continue];",
                 closure_file)
    release_file = MIR_FILENAMES["release_shared"]
    expect_block(texts["release_shared"], "bb0",
                 "StorageLive(_2); StorageLive(_3); StorageLive(_4); StorageLive(_5); "
                 "_5 = &'_ ((*_1).2: core::sync::atomic::Atomic<usize>); StorageLive(_6); "
                 "_6 = core::sync::atomic::Ordering::Release; "
                 "_4 = Atomic::<usize>::fetch_sub(move _5, const 1_usize, move _6) -> [return: bb1, unwind continue];",
                 release_file)
    expect_block(texts["release_shared"], "bb1",
                 "StorageDead(_6); StorageDead(_5); _3 = Ne(move _4, const 1_usize); "
                 "switchInt(move _3) -> [0: bb3, otherwise: bb2];", release_file)
    expect_block(texts["release_shared"], "bb3",
                 "StorageDead(_4); _2 = const (); StorageDead(_3); StorageDead(_2); "
                 "StorageLive(_8); StorageLive(_9); _9 = &'_ ((*_1).2: core::sync::atomic::Atomic<usize>); "
                 "StorageLive(_10); _10 = core::sync::atomic::Ordering::Acquire; "
                 "_8 = Atomic::<usize>::load(move _9, move _10) -> [return: bb4, unwind continue];",
                 release_file)
    expect_block(texts["release_shared"], "bb4",
                 "StorageDead(_10); StorageDead(_9); StorageDead(_8); StorageLive(_11); StorageLive(_12); "
                 "_12 = copy _1; _11 = free_shared(move _12) -> [return: bb5, unwind continue];",
                 release_file)
    free_file = MIR_FILENAMES["free_shared"]
    expect_block(texts["free_shared"], "bb0",
                 "StorageLive(_2); _2 = copy ((*_1).0: *mut u8); StorageLive(_3); "
                 "_3 = copy ((*_1).1: usize); StorageLive(_4); StorageLive(_5); _5 = copy _2; "
                 "StorageLive(_6); StorageLive(_7); StorageLive(_8); _8 = copy _3; "
                 "_7 = Layout::from_size_align(move _8, const 1_usize) -> [return: bb1, unwind continue];",
                 free_file)
    expect_block(texts["free_shared"], "bb2",
                 "StorageDead(_7); _4 = std::alloc::dealloc(move _5, move _6) -> [return: bb3, unwind continue];",
                 free_file)
    expect_block(texts["free_shared"], "bb3",
                 "StorageDead(_6); StorageDead(_5); StorageDead(_4); StorageLive(_9); StorageLive(_10); "
                 "StorageLive(_11); _11 = copy _1; "
                 "_10 = core::ptr::mut_ptr::<impl *mut bytes::Shared>::cast::<u8>(move _11) -> [return: bb4, unwind continue];",
                 free_file)
    expect_block(texts["free_shared"], "bb4",
                 "StorageDead(_11); StorageLive(_12); _12 = Layout::new::<bytes::Shared>() -> [return: bb5, unwind continue];",
                 free_file)
    expect_block(texts["free_shared"], "bb5",
                 "_9 = std::alloc::dealloc(move _10, move _12) -> [return: bb6, unwind continue];",
                 free_file)
    mut_file = MIR_FILENAMES["atomicmut_with_mut"]
    expect_block(texts["atomicmut_with_mut"], "bb0",
                 "_8 = const false; StorageLive(_3); _8 = const true; _3 = move _2; "
                 "StorageLive(_4); StorageLive(_5); StorageLive(_6); StorageLive(_7); "
                 "_7 = &'_ mut (*_1); _6 = Atomic::<*mut T>::get_mut(move _7) -> [return: bb1, unwind: bb4];",
                 mut_file)
    expect_block(texts["atomicmut_with_mut"], "bb1",
                 "_5 = &'_ mut (*_6); StorageDead(_7); _4 = (move _5,); _8 = const false; "
                 "_0 = <F as FnOnce<(&mut *mut T,)>>::call_once(move _3, move _4) -> [return: bb2, unwind: bb4];",
                 mut_file)
    expect_block(texts["shared_record_drop"], "bb2",
                 "StorageDead(_4); _0 = std::alloc::dealloc(move _2, move _3) -> [return: bb3, unwind continue];",
                 MIR_FILENAMES["shared_record_drop"])

    return {
        "mir_sha256": hashes,
        "scope_entry": "shared_automatic_scope",
        "output_saved_before_terminal_drops": {"block": "bb4", "place": "_0", "source": "_6"},
        "normal_terminal_drops": [
            {"place": "_4", "debug_name": "second", "block": "bb5",
             "normal_successor": "bb6", "unwind_successor": "bb10"},
            {"place": "_2", "debug_name": "first", "block": "bb6",
             "normal_successor": "bb7", "unwind_successor": "bb12"},
        ],
        "unwind_cleanup_edges_excluded": [
            {"block": "bb9", "drop": "_4", "normal_successor": "bb10", "unwind": "terminate(cleanup)"},
            {"block": "bb10", "drop": "_2", "normal_successor": "bb12", "unwind": "terminate(cleanup)"},
            {"block": "bb11", "normal_successor": "bb12"},
            {"block": "bb12", "normal_successor": "bb13"},
            {"block": "bb13", "terminator": "resume"},
        ],
        "actual_bytes_drop": {
            "vtable_field_index": 4,
            "arguments": ["&mut self.data", "self.ptr", "self.len"],
        },
        "shared_drop_callback": "AtomicMut::with_mut -> closure copies contained Shared pointer -> release_shared",
        "release_shared_mir_chain": ["Release fetch_sub", "if final owner: Acquire load", "free_shared"],
        "free_shared_mir_effects": ["payload dealloc", "Shared control dealloc"],
        "atomicmut_with_mut_mir": "AtomicPtr::get_mut -> FnOnce(&mut *mut T)",
        "normal_return_only": True,
    }


def audit_mir(mir_dir: pathlib.Path) -> dict[str, Any]:
    sources = {}
    for filename in MIR_FILENAMES.values():
        path = mir_dir / filename
        require(path.is_file(), f"missing post-ElaborateDrops MIR input `{filename}`")
        sources[filename] = path.read_text()
    return audit_mir_sources(sources)


def audit_capture(capture_path: pathlib.Path, mir_facts: dict[str, Any],
                  native_source: str, production_source: str) -> dict[str, Any]:
    capture_bytes = capture_path.read_bytes()
    capture = json.loads(capture_bytes)
    require(capture.get("stage") == "2-2-004.ElaborateDrops.after.mir",
            "native MIR capture report records an unexpected compiler stage")
    require(capture.get("native_source") == "native.rs" and
            capture.get("native_source_sha256") == sha(native_source.encode()),
            "native MIR capture report does not bind the checked client source")
    expected_selected = [
        {"path": f"native-mir/{filename}", "sha256": mir_facts["mir_sha256"][filename]}
        for filename in (
            MIR_FILENAMES["scope"], MIR_FILENAMES["bytes_drop"], MIR_FILENAMES["shared_record_drop"],
            MIR_FILENAMES["shared_drop"], MIR_FILENAMES["shared_drop_closure"],
            MIR_FILENAMES["release_shared"], MIR_FILENAMES["free_shared"], MIR_FILENAMES["atomicmut_with_mut"],
        )
    ]
    selected = capture.get("selected")
    require(isinstance(selected, list) and selected == expected_selected,
            "MIR capture report selected paths/hashes differ from the independently parsed eight inputs")
    require(capture.get("production_source") == "../../../src/bytes.rs" and
            capture.get("production_source_sha256") == sha(production_source.encode()),
            "native MIR capture report does not bind the actual Bytes production source")
    commands = capture.get("commands")
    require(isinstance(commands, list) and len(commands) == 2 and
            all("cargo rustc --locked --offline" in command and "-Zdump-mir=all" in command and
                "-Zmir-opt-level=0" in command and "-Zidentify-regions=yes" in command
                for command in commands),
            "MIR capture command record is incomplete or not the fixed fresh dump procedure")
    return {
        "capture_sha256": sha(capture_bytes),
        "selected_mir_count": len(selected),
        "source_and_mir_hashes_rederived": True,
        "command_recorded": True,
    }


def audit_shadow(active: str, ai: Any) -> dict[str, Any]:
    old_shadow = (AI_PROBE / "src" / "public_shared.rs").read_text()
    require(active.startswith(old_shadow),
            "generated active source must retain the byte-exact audited AI helper prefix")
    old_tokens = ai.rust_tokens(old_shadow)
    actual_tokens = ai.rust_tokens(active)
    require(actual_tokens[:len(old_tokens)] == old_tokens,
            "generated active proof source does not preserve the exact audited AI source prefix")
    tail = actual_tokens[len(old_tokens):]
    expected_shadow_tokens = json.loads(EXPECTED_TERMINAL_SHADOW.read_text())["items"]
    helper_tokens = expected_shadow_tokens["terminal_helper"].split()
    client_tokens = expected_shadow_tokens["elaborated_client"].split()
    require(tail == helper_tokens + client_tokens,
            "active proof source differs from the complete reviewed terminal helper and caller token patterns")
    require(not any(tail[i:i + 4] == ["impl", "Drop", "for", "Bytes"]
                    for i in range(len(tail) - 3)),
            "proof shadow must not introduce a Bytes Drop implementation")
    for forbidden in ("trusted", "assume", "axiom", "extern_spec", "externspec", "checktrusted"):
        require(forbidden not in tail,
                f"terminal shadow adds forbidden trusted/proof escape token `{forbidden}`")
    helper_signature = "fn bytes_terminal_drop(value:Bytes,cursor:Ghost<&mut Cursor>,output:Ghost<&mut Option<Completion>>)"
    ai.check_function_surface(active, "bytes_terminal_drop", helper_signature,
                              "{ original_shared_cleanup(value,cursor,output) }")
    base_cleanup_attributes = ai.function_outer_attributes(old_shadow, "original_shared_cleanup")
    terminal_attributes = ai.function_outer_attributes(active, "bytes_terminal_drop")
    require(terminal_attributes == base_cleanup_attributes,
            "terminal helper must reuse the exact body-proved cleanup contract and may not add trust")
    caller_signature = "fn shared_automatic_scope(input:Vec<u8>)->Vec<u8>"
    ai.check_function_surface(active, "shared_automatic_scope", caller_signature)
    require(ai.function_outer_attributes(active, "shared_automatic_scope") == [
        ai.rust_tokens("#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]"),
        ai.rust_tokens("#[ensures(result@ == input@)]"),
    ], "automatic-Drop shadow caller has unsupported admission/result attributes")
    driver, _, _ = ai.find_function(active, "shared_automatic_scope")
    expected_operations = [
        "original_shared_from_vec ( input )",
        "original_shared_clone ( & first , cursor . borrow_mut ( ) )",
        "original_shared_as_slice ( & second )",
        "borrowed . to_vec ( )",
        "let saved_return = observed ;",
        "bytes_terminal_drop ( second , cursor . borrow_mut ( ) , second_receipt . borrow_mut ( ) )",
        "bytes_terminal_drop ( first , cursor . borrow_mut ( ) , first_receipt . borrow_mut ( ) )",
    ]
    positions = [ai.find_subsequence(driver, ai.rust_tokens(expr)) for expr in expected_operations]
    require(all(pos is not None for pos in positions) and positions == sorted(positions) and
            driver[-1] == "saved_return",
            "shadow must save the observed Vec before consuming terminal owners in native MIR order")
    ai.contains_once(driver, ai.rust_tokens("!(*cursor.inner_logic().observation()).0.contains(*second_id)"),
                     "second-ticket removal assertion")
    ai.contains_once(driver, ai.rust_tokens("(*cursor.inner_logic().observation()).0.contains(*first_id)"),
                     "first-ticket survives second terminal drop")
    ai.contains_once(driver, ai.rust_tokens("(*cursor.inner_logic().observation()).0.len() == 0"),
                     "empty terminal issuance map assertion")
    return {
        "active_source_sha256": sha(active.encode()),
        "audited_ai_prefix_sha256": sha(old_shadow.encode()),
        "ai_prefix_token_count": len(old_tokens),
        "terminal_helper_token_count": len(helper_tokens),
        "elaborated_client_token_count": len(client_tokens),
        "helper_name": "bytes_terminal_drop",
        "helper_contract_reused_from": "original_shared_cleanup",
        "terminal_owner_order": ["second", "first"],
        "terminal_output_bindings": [
            {"owner": "second", "cursor": "cursor", "output": "second_receipt"},
            {"owner": "first", "cursor": "cursor", "output": "first_receipt"},
        ],
        "return_saved_before_terminal_effects": True,
        "second_kept_alive_then_first_reclaimed": True,
        "final_ticket_map_empty": True,
        "terminal_shadow_extra_trust": False,
        "bytes_drop_impl_in_shadow": False,
    }


def audit_mapping_data(mapping: dict[str, Any], mapping_bytes: bytes, mapping_label: str,
                  mir_facts: dict[str, Any], shadow_facts: dict[str, Any], *, native_source: str,
                  base_source: str, active_source: str, ai: Any, mir_dir: pathlib.Path) -> dict[str, Any]:
    require(mapping.get("feature") == "", "canonical mapping must select the ordinary positive feature set")
    require(mapping.get("stage") == "2-2-004.ElaborateDrops.after.mir",
            "mapping uses an unexpected MIR stage")
    require(mapping.get("native_source") == "native.rs" and
            mapping.get("native_source_sha256") == sha(native_source.encode()),
            "mapping native source path/hash differs from checked native client")
    require(mapping.get("base_source") == "../original-shared-scoped-client-2026-10-09/src/public_shared.rs" and
            mapping.get("base_source_sha256") == sha(base_source.encode()),
            "mapping base proof source differs from audited AI source")
    require(mapping.get("active") == "generated/active.rs" and
            mapping.get("active_sha256") == sha(active_source.encode()),
            "mapping generated active source path/hash differs from audited shadow")
    require(mapping.get("helper") == "bytes_terminal_drop" and
            mapping.get("helper_body") == "ordinary forward call to body-proved original_shared_cleanup; expanded native projected dispatch checked independently",
            "mapping terminal helper identity/body claim changed")
    expected_edges = [
        {"block": "bb5", "place": "_4", "owner": "second", "successor": "bb6", "unwind": "bb10", "cursor": "cursor", "output": "second_receipt"},
        {"block": "bb6", "place": "_2", "owner": "first", "successor": "bb7", "unwind": "bb12", "cursor": "cursor", "output": "first_receipt"},
    ]
    derived_mir_edges = [
        {"block": row["block"], "place": row["place"], "owner": row["debug_name"],
         "successor": row["normal_successor"], "unwind": row["unwind_successor"]}
        for row in mir_facts["normal_terminal_drops"]
    ]
    derived_shadow_edges = [
        {"cursor": row["cursor"], "output": row["output"]}
        for row in shadow_facts["terminal_output_bindings"]
    ]
    combined_edges = [dict(**native_edge, **shadow_edge)
                      for native_edge, shadow_edge in zip(derived_mir_edges, derived_shadow_edges)]
    require(combined_edges == expected_edges and mapping.get("normal_edges") == expected_edges,
            "mapping terminal edges differ from independently reconstructed MIR places/order and shadow receipt channels")
    require(mapping.get("return_evaluation") == {
        "native_block": "bb4", "native": "_0 = move _6", "shadow": "let saved_return=observed",
        "effects_after_evaluation": True,
    }, "mapping return-evaluation point differs from native MIR and shadow source")
    mir_rows = mapping.get("mir")
    require(isinstance(mir_rows, list), "mapping must list the selected native MIR inputs")
    expected_mir_rows = [
        {"path": f"native-mir/{filename}", "sha256": mir_facts["mir_sha256"][filename]}
        for filename in sorted(MIR_FILENAMES.values())
    ]
    require(mir_rows == expected_mir_rows,
            "mapping MIR path/hash list differs from independently parsed pinned inputs")
    require(mapping.get("excluded") == ["unwind cleanup correctness", "other representations",
            "arbitrary Drop/move equivalence", "arbitrary concurrent closure", "whole crate"],
            "mapping exclusions changed or no longer disclose the actual scope")
    require(mapping.get("tcb") == ["native MIR/compiler interpretation",
            "generic terminal-place/non-address-observing correspondence",
            "ghost erasure and native callback reification", "AI scoped field/physical primitives"],
            "mapping TCB list changed or omits the terminal-place premise")
    return {
        "mapping_sha256": sha(mapping_bytes),
        "mapping_path": mapping_label,
        "facts_match_independent_reconstruction": True,
        "normal_edges_rederived": expected_edges,
    }


def audit_mapping(mapping_path: pathlib.Path, mir_facts: dict[str, Any], shadow_facts: dict[str, Any], *, native_source: str,
                  base_source: str, active_source: str, ai: Any, mir_dir: pathlib.Path) -> dict[str, Any]:
    return audit_mapping_data(json.loads(mapping_path.read_text()), mapping_path.read_bytes(),
                              str(mapping_path), mir_facts, shadow_facts,
                              native_source=native_source, base_source=base_source,
                              active_source=active_source, ai=ai, mir_dir=mir_dir)


def audit_all(*, native_source: str, mir_dir: pathlib.Path, active_source: str,
              mapping_path: pathlib.Path) -> dict[str, Any]:
    global AI
    AI = load_ai_checker()
    native = audit_native_source(native_source)
    production = audit_production_drop_sources(
        AI, (CRATE_ROOT / "src/bytes.rs").read_text(),
        (CRATE_ROOT / "src/ref_count_ops.rs").read_text(),
        (CRATE_ROOT / "src/bytes/shared_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text(),
        (CRATE_ROOT / "src/loom.rs").read_text())
    mir = audit_mir(mir_dir)
    capture = audit_capture(mir_dir / "capture.json", mir, native_source,
                            (CRATE_ROOT / "src/bytes.rs").read_text())
    provenance = audit_native_provenance(AI)
    shadow = audit_shadow(active_source, AI)
    mapping = audit_mapping(mapping_path, mir, shadow, native_source=native_source,
                            base_source=(AI_PROBE / "src/public_shared.rs").read_text(),
                            active_source=active_source, ai=AI, mir_dir=mir_dir)
    return {
        "status": "correspondence_pass",
        "native_client": native,
        "production_drop_sources": production,
        "native_mir": mir,
        "mir_capture": capture,
        "native_toolchain_and_harness": provenance,
        "proof_shadow": shadow,
        "generated_mapping": mapping,
        "claim_boundary": [
            "selected default-std normal-return path only; unwind edges are explicitly excluded",
            "no arbitrary Bytes representation, concurrency, panic, allocator, or other cfg claim",
            "native Bytes field-category no-drop witness uses the selected default core AtomicPtr; generic borrowed-reference no-drop behavior remains compiler/library TCB",
            "compiler, Cargo lock, native harness, and proof module route are pinned provenance, not a proof of compiler correctness",
            "normal Drop-to-consuming-terminal-helper equivalence is a generic MIR/codegen TCB premise",
            "trusted event/cursor/free/invoke3 adequacy and helper contracts remain separately scoped TCB",
            "native run is corroboration; this audit is structural source/MIR correspondence, not proof",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=pathlib.Path, default=NATIVE_SOURCE)
    parser.add_argument("--mir-dir", type=pathlib.Path, default=MIR_DIR)
    parser.add_argument("--shadow", "--active", dest="shadow", type=pathlib.Path, default=ACTIVE_SHADOW)
    parser.add_argument("--mapping", type=pathlib.Path, default=MAP_DEFAULT)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        result = audit_all(native_source=args.native.read_text(), mir_dir=args.mir_dir,
                           active_source=args.shadow.read_text(),
                           mapping_path=args.mapping)
    except Exception as exc:
        print(json.dumps({"status": "correspondence_failed", "error": str(exc)}, indent=2))
        return 2
    text = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.write_text(text)
    print(text, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
