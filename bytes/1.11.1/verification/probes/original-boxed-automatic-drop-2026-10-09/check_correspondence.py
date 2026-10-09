#!/usr/bin/env python3
"""Fail-closed source/MIR correspondence audit for boxed Bytes automatic Drop.

This closed-client checker establishes the exact default-native client, its
post-ElaborateDrops normal terminal edge, the selected production callback
sources, and the proof-side consuming helper. It does not prove compiler/MIR
adequacy, exposed-provenance behavior, ghost erasure, or the generic physical
read/free contracts; those remain explicit TCB items.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
PROBES = ROOT.parent
AI_PROBE = PROBES / "original-shared-scoped-client-2026-10-09"
AI_CHECKER_PATH = AI_PROBE / "check_correspondence.py"
AI_CHECKER_SHA256 = "decb502490fdd1c534e02b5ac0798b070eeb8ed7be4d6aa1779e05cee5e08bb7"
AI_HELPER_FIXTURE_SHA256 = "c4903a139f0924fc26a62c8319617db1f34cfbb876e7e775abba0f70130d26e1"
GENERATED = ROOT / "generated"
MIR_DIR = ROOT / "native-mir"
HARNESS = GENERATED / "native-harness"
FIXTURES = ROOT / "fixtures"
EXPECTED = FIXTURES / "expected-inputs.json"
EXPECTED_MIR = FIXTURES / "expected-native-mir-tokens.json"
EXPECTED_SHADOW = FIXTURES / "expected-shadow-tokens.json"
EXPECTED_PRODUCTION = FIXTURES / "expected-production-tokens.json"
EXPECTED_GENERATED = FIXTURES / "expected-generated-tokens.json"
EXPECTED_PROVENANCE = FIXTURES / "expected-native-provenance.json"
POSITIVE_SHADOW = GENERATED / "positive.rs"
POSITIVE_MAPPING = GENERATED / "positive-mapping.json"

MIR_FILES = {
    "client": "bytes_boxed_drop_native.boxed_read_then_drop.2-2-004.ElaborateDrops.after.mir",
    "bytes_drop": "bytes.bytes-{impl#3}-drop.2-2-004.ElaborateDrops.after.mir",
    "static_drop": "bytes.bytes-static_drop.2-2-004.ElaborateDrops.after.mir",
    "even_drop": "bytes.bytes-promotable_even_drop.2-2-004.ElaborateDrops.after.mir",
    "even_outer_closure": "bytes.bytes-promotable_even_drop-{closure#0}.2-2-004.ElaborateDrops.after.mir",
    "even_tag_closure": "bytes.bytes-promotable_even_drop-{closure#0}-{closure#0}.2-2-004.ElaborateDrops.after.mir",
    "odd_drop": "bytes.bytes-promotable_odd_drop.2-2-004.ElaborateDrops.after.mir",
    "odd_closure": "bytes.bytes-promotable_odd_drop-{closure#0}.2-2-004.ElaborateDrops.after.mir",
    "free_boxed_slice": "bytes.bytes-free_boxed_slice.2-2-004.ElaborateDrops.after.mir",
    "ptr_map": "bytes.bytes-ptr_map.2-2-004.ElaborateDrops.after.mir",
    "atomic_with_mut": "bytes.loom-sync-atomic-{impl#0}-with_mut.2-2-004.ElaborateDrops.after.mir",
}


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_ai_checker():
    require(AI_CHECKER_PATH.is_file(), f"missing audited source checker dependency: {AI_CHECKER_PATH}")
    require(sha(AI_CHECKER_PATH.read_bytes()) == AI_CHECKER_SHA256,
            "reused source parser/checker differs from its audited frozen version")
    helper_fixture = AI_PROBE / "fixtures" / "expected-shadow-bodies.json"
    require(helper_fixture.is_file() and sha(helper_fixture.read_bytes()) == AI_HELPER_FIXTURE_SHA256,
            "reused parser helper-pattern fixture differs from its audited frozen version")
    spec = importlib.util.spec_from_file_location("boxed_automatic_drop_ai_checker", AI_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load audited parser/checker dependency")
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
    brace = (start + len(prefix_tokens) - 1 if prefix_tokens and prefix_tokens[-1] == "{" else
             next((i for i in range(start + len(prefix_tokens), len(tokens)) if tokens[i] == "{"), None))
    require(brace is not None, f"`{label}` has no body")
    depth = 0
    for i in range(brace, len(tokens)):
        if tokens[i] == "{":
            depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[start:i + 1]
    raise AuditError(f"unclosed source item `{label}`")


def expect_tokens(actual: str | list[str], expected: list[str], label: str) -> None:
    tokens = AI.rust_tokens(actual) if isinstance(actual, str) else actual
    require(tokens == expected, f"{label} differs from the reviewed token pattern")


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
    require(len(hits) == 1, f"MIR `{source_name}` must contain exactly one block `{label}`")
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


def expect_block(mir: str, label: str, expected: str, source_name: str) -> None:
    require(find_block(AI.rust_tokens(mir), label, source_name) == AI.rust_tokens(expected),
            f"MIR `{source_name}` block `{label}` changed")


def audit_native_client(native_source: str, expected: dict[str, Any]) -> dict[str, Any]:
    expect_tokens(native_source, expected["native_client_tokens"], "native client")
    tokens = AI.rust_tokens(native_source)
    require(tokens.count("from") == 1 and tokens.count("clone") == 0,
            "native client must have one Box constructor and no clone")
    require("drop" not in tokens and "forget" not in tokens and "ManuallyDrop" not in tokens,
            "native client must use ordinary normal-return Drop")
    return {
        "source_sha256": sha(native_source.encode()),
        "entrypoint": "boxed_read_then_drop",
        "client_protocol": ["Bytes::from(Box<[u8]>)", "AsRef<[u8]>", "to_vec", "normal return"],
        "explicit_drop_or_forget_calls": 0,
        "closed_source": True,
    }


def audit_module_wiring(lib_source: str, expected: dict[str, Any]) -> dict[str, Any]:
    expect_tokens(lib_source, expected["lib_source_tokens"], "proof crate root/module wiring")
    for name, path in expected["support_module_paths"].items():
        try:
            actual = AI.rust_path_for_module(lib_source, name)
        except Exception as exc:
            raise AuditError(f"cannot resolve selected module `{name}` path") from exc
        require(actual == path, f"support module `{name}` path changed")
    return {"lib_source_sha256": sha(lib_source.encode()), "support_module_paths": expected["support_module_paths"],
            "all_selected_module_routes_exact": True}


def audit_native_provenance_data(*, rustc_text: str, cargo_text: str, manifest_text: str,
                                 lock_text: str, harness_source: str, lib_source: str,
                                 native_run_log: str, expected: dict[str, Any]) -> dict[str, Any]:
    require(rustc_text == expected["rustc_version_text"], "recorded rustc version/host changed")
    require(cargo_text == expected["cargo_version_text"], "recorded cargo version changed")
    manifest = tomllib.loads(manifest_text)
    require(manifest == expected["native_manifest"],
            "native harness manifest changed crate target, dependency path, or feature surface")
    native_target = pathlib.Path(manifest["lib"]["path"])
    native_target = (HARNESS / native_target).resolve() if not native_target.is_absolute() else native_target.resolve()
    dependency_target = pathlib.Path(manifest["dependencies"]["bytes"]["path"])
    dependency_target = ((HARNESS / dependency_target).resolve() if not dependency_target.is_absolute()
                        else dependency_target.resolve())
    require(native_target == (ROOT / "native.rs").resolve() and dependency_target == CRATE_ROOT.resolve(),
            "relative native harness paths do not resolve to this probe's native.rs and bytes crate")
    lock = tomllib.loads(lock_text)
    packages = [{k: p[k] for k in ("name", "version", "source", "checksum", "dependencies") if k in p}
                for p in lock.get("package", [])]
    require(lock.get("version") == expected["lock_version"] and packages == expected["lock_packages"],
            "native harness lock package identities/checksums differ from reviewed versions")
    expect_tokens(harness_source, expected["native_harness_tokens"], "native execution harness")
    profile = expected["native_no_drop_profile_tokens"]
    tokens = AI.rust_tokens(harness_source)
    require(sum(tokens[i:i + len(profile)] == profile for i in range(len(tokens) - len(profile) + 1)) == 1,
            "native harness must compile-check the no-drop field-category profile")
    wiring = audit_module_wiring(lib_source, expected)
    run_line = expected["native_run_success_line"]
    require(native_run_log.rstrip().endswith(run_line) and "Finished `dev` profile" in native_run_log and
            "Running `" in native_run_log,
            "native harness log does not prove the recorded five-input execution")
    return {
        "rustc_version_sha256": sha(rustc_text.encode()), "cargo_version_sha256": sha(cargo_text.encode()),
        "native_manifest_sha256": sha(manifest_text.encode()), "native_lock_sha256": sha(lock_text.encode()),
        "native_harness_source_sha256": sha(harness_source.encode()), "native_run_log_sha256": sha(native_run_log.encode()),
        "proof_crate_root": wiring, "selected_native_library": str(native_target),
        "selected_production_dependency": str(dependency_target),
        "native_no_drop_field_profile": True, "native_harness_inputs": [0, 1, 7, 63, 1024],
        "toolchain_dependency_and_module_identity_pinned": True,
        "native_run_is_correspondence_corroboration_only": True,
    }


def audit_native_provenance(expected: dict[str, Any], lib_source: str) -> dict[str, Any]:
    values = getattr(AI, "_provenance_mutation", None) or {
        "rustc_text": (MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (MIR_DIR / "cargo-version.txt").read_text(),
        "manifest_text": (HARNESS / "Cargo.toml").read_text(),
        "lock_text": (HARNESS / "Cargo.lock").read_text(),
        "harness_source": (HARNESS / "src/main.rs").read_text(),
        "native_run_log": (GENERATED / "native-run.log").read_text(),
    }
    return audit_native_provenance_data(**values, lib_source=lib_source, expected=expected)


def audit_production_sources(*, bytes_source: str, bytes_record: str, vtable_record: str,
                             loom_source: str, expected: dict[str, Any]) -> dict[str, Any]:
    files = {"bytes.rs": bytes_source, "bytes_record.rs": bytes_record,
             "vtable_record.rs": vtable_record, "loom.rs": loom_source}
    for source_name, items in expected.items():
        tokens = AI.rust_tokens(files[source_name])
        for label, record in items.items():
            if isinstance(record, list):
                needle = record
                count = sum(tokens[i:i + len(needle)] == needle for i in range(len(tokens) - len(needle) + 1))
                require(count == 1, f"selected production snippet `{label}` in {source_name} changed or is ambiguous")
            elif "whole_file_tokens" in record:
                require(tokens == record["whole_file_tokens"], f"selected production module `{source_name}` changed")
            elif "snippet" in record:
                needle = record["snippet"]
                count = sum(tokens[i:i + len(needle)] == needle for i in range(len(tokens) - len(needle) + 1))
                require(count == 1, f"selected production snippet `{label}` in {source_name} changed or is ambiguous")
            else:
                actual = extract_item(tokens, record["prefix"], label)
                require(actual == record["tokens"], f"selected production item `{label}` in {source_name} changed")
    # Retain all three native table branches and their exact callable targets.
    bytes_tokens = AI.rust_tokens(bytes_source)
    for label in ("static_vtable", "promotable_even_vtable", "promotable_odd_vtable"):
        require(label in expected["bytes.rs"], f"expected source patterns omit `{label}`")
    return {
        "selected_drop_and_callback_items_exact": True,
        "native_tables_and_kind_branch_exact": True,
        "bytes_record_and_vtable_layout_exact": True,
        "default_native_atomicmut_exact": True,
        "production_bytes_source_sha256": sha(bytes_source.encode()),
    }


def raw_item_span(source: str, prefix: str, label: str) -> tuple[str, int]:
    require(source.count(prefix) == 1, f"expected one raw source prefix for `{label}`")
    start = source.index(prefix)
    brace = source.index("{", start)
    depth = 0
    for i in range(brace, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start:i + 1], source[:start].count("\n") + 1
    raise AuditError(f"unclosed raw source item `{label}`")


def marker_span(source: str, family: str, name: str) -> tuple[str, int]:
    begin = f"// {family}_BEGIN {name}\n"
    end = f"// {family}_END {name}"
    require(source.count(begin) == 1 and source.count(end) == 1,
            f"production source markers for `{name}` are missing or duplicated")
    start = source.index(begin)
    finish = source.index(end, start) + len(end)
    return source[start:finish], source[:start].count("\n") + 1


def audit_constructor_extractions(*, bytes_source: str, bytes_record_source: str,
                                  vtable_record_source: str, bytes_mut_source: str,
                                  generated_sources: dict[str, str], source_map: dict[str, Any],
                                  expected_generated: dict[str, Any]) -> dict[str, Any]:
    spans: dict[str, str] = {}
    span_rows: dict[str, tuple[str, int]] = {}
    for name, family in (
        ("bytes_from_vec_impl", "ORIGINAL_SHARED"),
        ("bytes_from_box_impl", "ORIGINAL_CONSTRUCTOR"),
        ("bytes_new", "ORIGINAL_CONSTRUCTOR"),
        ("bytes_from_static", "ORIGINAL_CONSTRUCTOR"),
        ("bytes_as_slice", "ORIGINAL_SHARED"),
        ("bytes_as_ref_impl", "ORIGINAL_SHARED"),
    ):
        span_rows[name] = marker_span(bytes_source, family, name)
        spans[name] = span_rows[name][0]
        row = source_map.get(name)
        require(row == {"sha256": sha(spans[name].encode()), "line": span_rows[name][1]},
                f"source map `{name}` hash/line differs from the actual marked production source span")

    require(source_map.get("from_refinements") == {
        "included_items": ["bytes_from_vec_impl", "bytes_from_box_impl", "bytes_new",
                           "bytes_from_static", "bytes_as_slice", "bytes_as_ref_impl"],
        "constructor_gate_preconditions": [],
        "ensures": ["From<Vec<u8>>: original_bytes_valid && original_bytes_content == vec@",
                    "From<Box<[u8]>>: original_bytes_valid && original_bytes_content == slice@"],
    }, "source map no longer describes the unrestricted selected constructor/read refinements")
    def add_static_repr_postcondition(source: str, signature: str) -> str:
        require(source.count(signature) > 0, f"expected instrumentation anchor `{signature}`")
        return source.replace(signature,
            "#[cfg_attr(creusot, ensures(result.static_repr()))]\n    " + signature)
    new_method = add_static_repr_postcondition(spans["bytes_new"], "pub fn new() -> Self")
    static_method = add_static_repr_postcondition(spans["bytes_from_static"],
        "pub fn from_static(bytes: &'static [u8]) -> Self")
    require(source_map.get("AK_shape_instrumentation") == {
        "postconditions": ["new/static result.static_repr"], "native_bodies_changed": False},
        "AK shape postcondition instrumentation changed")
    expected_traits = (spans["bytes_from_vec_impl"] + "\n" + spans["bytes_from_box_impl"] + "\n"
                       + "impl Bytes {\n" + new_method + "\n" + static_method + "\n"
                       + spans["bytes_as_slice"] + "\n}\n" + spans["bytes_as_ref_impl"] + "\n")
    require(AI.rust_tokens(generated_sources["public_traits.rs"]) == AI.rust_tokens(expected_traits),
            "generated public trait/constructor source differs from exact production marker spans")

    # Reconstruct the five native vtable declarations/getter from production.
    binding_specs = [
        ("static SHARED_VTABLE: Vtable = Vtable {", "native/shared_vtable"),
        ("fn original_shared_table_native() -> &'static Vtable", "native/shared_table_native"),
        ("const STATIC_VTABLE: Vtable = Vtable {", "native/static_vtable"),
        ("static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {", "native/promotable_even_vtable"),
        ("static PROMOTABLE_ODD_VTABLE: Vtable = Vtable {", "native/promotable_odd_vtable"),
    ]
    bindings: list[str] = []
    for prefix, label in binding_specs:
        span, line = raw_item_span(bytes_source, prefix, label)
        bindings.append(span)
        require(source_map.get(label) == {"sha256": sha(span.encode()), "line": line},
                f"source map `{label}` hash/line differs from native production declaration")
    expected_bindings = "\n\n".join(bindings) + "\n"
    require(AI.rust_tokens(generated_sources["native_constructor_bindings.rs"]) ==
            AI.rust_tokens(expected_bindings),
            "generated native vtable binding source differs from actual production declarations")

    # Reconstruct record extraction from real record files, including the two
    # small BytesMut record declarations included by the verification crate.
    shared_mut = AI.extract_struct_source(bytes_mut_source, "struct Shared {", "BytesMut::Shared")
    mutable_record = AI.extract_struct_source(bytes_mut_source, "pub struct BytesMut {", "BytesMut")
    expected_records = (bytes_record_source + "\n" + vtable_record_source + "\n" +
        "mod mutable_record {\nuse alloc::vec::Vec;\n"
        "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" + shared_mut + "\n" +
        mutable_record + "\n}\nuse mutable_record::BytesMut;\n")
    require(AI.rust_tokens(generated_sources["public_records.rs"]) == AI.rust_tokens(expected_records),
            "generated public record source differs from actual Bytes/Vtable/BytesMut declarations")

    for filename, source in generated_sources.items():
        key = f"generated/{filename}"
        if key in source_map:
            require(source_map[key].get("sha256") == sha(source.encode()),
                    f"source map generated-output row `{key}` has a stale hash")
    for filename, source in (("bytes_record.rs", bytes_record_source), ("vtable_record.rs", vtable_record_source)):
        require(source_map.get(filename, {}).get("sha256") == sha(source.encode()),
                f"source map `{filename}` hash differs from selected production record source")
    for name, source in (("bytes_mut::Shared", shared_mut), ("bytes_mut::BytesMut", mutable_record)):
        start = bytes_mut_source.index("struct Shared {") if name.endswith("Shared") else bytes_mut_source.index("pub struct BytesMut {")
        require(source_map.get(name) == {"sha256": sha(source.encode()),
                "line": bytes_mut_source[:start].count("\n") + 1},
                f"source map `{name}` row differs from actual record declaration")

    # The Shared constructor component comes from the already admitted scoped
    # constructor source. Pin its complete generated tokens and generator facts.
    shared_component = generated_sources["shared_constructor_component.rs"]
    require(AI.rust_tokens(shared_component) == expected_generated["shared_constructor_component_tokens"],
            "generated Shared constructor component differs from the reviewed source snapshot")
    row = source_map.get("generated/shared_constructor_component.rs", {})
    require(row == expected_generated["shared_constructor_component_source_map"],
            "Shared constructor component source-map provenance changed")

    # KIND_VEC is a source-level identity fact used by the tagged Box branch.
    match = re.search(r"const\s+KIND_VEC:\s*usize\s*=\s*(0b[01_]+|[0-9_]+)\s*;", bytes_source)
    require(match is not None and int(match.group(1).replace("_", ""), 0) == 1,
            "native KIND_VEC tag value is no longer one")
    kind_line = bytes_source[:match.start()].count("\n") + 1
    require(source_map.get("native/kind_vec") == {
        "sha256": sha(match.group(0).encode()), "line": kind_line, "value": 1},
        "source map KIND_VEC row differs from the actual native constant")
    bindings_file = generated_sources["native_constructor_bindings.rs"]
    require(source_map.get("closed_table_bindings") == {
        "sha256": sha(bindings_file.encode()),
        "status": "source-checked exact native table declarations; the restricted reification remains generic TCB",
        "tables": ["STATIC_VTABLE", "PROMOTABLE_EVEN_VTABLE", "PROMOTABLE_ODD_VTABLE", "SHARED_VTABLE"]},
        "closed-table binding source-map receipt differs from the reconstructed declarations")
    return {"actual_constructor_markers_rederived": True, "generated_traits_match_source": True,
            "generated_records_match_source": True, "native_tables_rederived": True,
            "source_map_selected_rows_recomputed": True}


def audit_mir_sources(mir_sources: dict[str, str], capture_data: dict[str, Any],
                      native_source: str, production_source: str,
                      expected: dict[str, Any]) -> dict[str, Any]:
    filenames = set(MIR_FILES.values())
    require(set(mir_sources) == filenames, "selected native MIR input set changed")
    expected_mir = json.loads(EXPECTED_MIR.read_text())["files"]
    require(set(expected_mir) == filenames, "native MIR fixture set changed")
    hashes: dict[str, str] = {}
    texts: dict[str, str] = {}
    for name, filename in MIR_FILES.items():
        text = mir_sources[filename]
        toks = AI.rust_tokens(text)
        require(toks == expected_mir[filename], f"native MIR `{filename}` differs from reviewed tokens")
        texts[name] = text
        hashes[filename] = sha(text.encode())

    client_name = MIR_FILES["client"]
    expect_block(texts["client"], "bb3", "StorageDead(_5); StorageDead(_8); StorageDead(_6); _0 = move _4; goto -> bb4;", client_name)
    expect_block(texts["client"], "bb4", "StorageDead(_4); drop(_2) -> [return: bb5, unwind: bb9];", client_name)
    expect_block(texts["client"], "bb5", "StorageDead(_2); goto -> bb6;", client_name)
    expect_block(texts["client"], "bb6", "return;", client_name)
    expect_block(texts["bytes_drop"], "bb0",
        "StorageLive(_2); _7 = deref_copy ((*_1).3: &bytes::Vtable); "
        "_2 = copy ((*_7).4: for<'a> unsafe fn(&'a mut core::sync::atomic::Atomic<*mut ()>, *const u8, usize)); "
        "StorageLive(_3); StorageLive(_4); _4 = &'_ mut ((*_1).2: core::sync::atomic::Atomic<*mut ()>); "
        "_3 = &'_ mut (*_4); StorageLive(_5); _5 = copy ((*_1).0: *const u8); "
        "StorageLive(_6); _6 = copy ((*_1).1: usize); "
        "_0 = move _2(move _3, move _5, move _6) -> [return: bb1, unwind continue];",
        MIR_FILES["bytes_drop"])
    expect_block(texts["static_drop"], "bb0", "_0 = const (); return;", MIR_FILES["static_drop"])
    discriminator = AI.rust_tokens("_7 = Eq(move _8, const bytes::KIND_ARC);")
    branch = AI.rust_tokens("switchInt(move _7) -> [0: bb4, otherwise: bb1];")
    release = AI.rust_tokens("_9 = bytes::release_shared(move _10) -> [return: bb3, unwind continue];")
    for name in ("even_outer_closure", "odd_closure"):
        toks = AI.rust_tokens(texts[name])
        require(any(toks[i:i + len(discriminator)] == discriminator for i in range(len(toks) - len(discriminator) + 1)) and
                any(toks[i:i + len(branch)] == branch for i in range(len(toks) - len(branch) + 1)) and
                any(toks[i:i + len(release)] == release for i in range(len(toks) - len(release) + 1)),
                f"native `{name}` MIR must retain ARC discriminator/release and raw branch")
    even_free = AI.rust_tokens("_37 = free_boxed_slice(move _38, move _39, move _40) -> [return: bb12, unwind continue];")
    odd_free = AI.rust_tokens("_33 = free_boxed_slice(move _34, move _36, move _37) -> [return: bb11, unwind continue];")
    for name, call in (("even_outer_closure", even_free), ("odd_closure", odd_free)):
        toks = AI.rust_tokens(texts[name])
        require(any(toks[i:i + len(call)] == call for i in range(len(toks) - len(call) + 1)),
                f"native `{name}` MIR must retain the boxed-slice free callback")
    require("PointerExposeProvenance" in AI.rust_tokens(texts["ptr_map"]),
            "selected native ptr_map MIR no longer uses the reviewed exposed-provenance route")

    capture = capture_data
    require(capture.get("stage") == "2-2-004.ElaborateDrops.after.mir" and
            capture.get("native_source") == "native.rs" and
            capture.get("native_source_sha256") == sha(native_source.encode()),
            "MIR capture report does not bind the exact native source/stage")
    selected = [{"path": f"native-mir/{f}", "sha256": hashes[f]} for f in MIR_FILES.values()]
    require(capture.get("selected") == selected, "MIR capture selected path/hash rows changed")
    require(capture.get("production_source") == "../../../src/bytes.rs" and
            capture.get("production_source_sha256") == sha(production_source.encode()),
            "MIR capture report does not bind current production bytes.rs")
    commands = capture.get("commands")
    require(isinstance(commands, list) and len(commands) == 2 and
            all("cargo rustc --locked --offline" in c and "-Zdump-mir=all" in c and
                "-Zmir-opt-level=0" in c and "-Zidentify-regions=yes" in c for c in commands),
            "MIR capture command record changed or omitted the dump settings")
    return {
        "mir_sha256": hashes,
        "capture_selected": selected,
        "capture_data": capture,
        "capture_sha256": sha(json.dumps(capture, sort_keys=True).encode()),
        "normal_terminal_drop": {"block": "bb4", "place": "_2", "owner": "bytes", "normal": "bb5", "unwind": "bb9"},
        "return_evaluated_before_drop": {"block": "bb3", "operation": "_0 = move _4"},
        "bytes_drop_dispatch": {"vtable_field_index": 4, "args": ["&mut data", "ptr", "len"]},
        "selected_branch_families": ["empty Static/no-op", "even raw/tag-clear/free", "odd raw/free"],
        "arc_branch_retained_in_native_mir": True,
        "normal_return_only": True,
    }


def audit_mir(mir_dir: pathlib.Path, native_source: str, production_source: str,
              expected: dict[str, Any], sources_override: dict[str, str] | None = None,
              capture_override: dict[str, Any] | None = None) -> dict[str, Any]:
    sources: dict[str, str] = {}
    for filename in MIR_FILES.values():
        if sources_override is not None:
            require(filename in sources_override, f"missing selected post-ElaborateDrops MIR `{filename}`")
            sources[filename] = sources_override[filename]
        else:
            path = mir_dir / filename
            require(path.is_file(), f"missing selected post-ElaborateDrops MIR `{filename}`")
            sources[filename] = path.read_text()
    capture = capture_override if capture_override is not None else json.loads((mir_dir / "capture.json").read_text())
    return audit_mir_sources(sources, capture, native_source, production_source, expected)


def audit_shadow_sources(*, public_constructor: str, boxed_drop: str, tag_specs: str,
                         physical_projection: str, erased_call: str, read_projection: str,
                         driver_template: str, shadow: str, expected: dict[str, Any]) -> dict[str, Any]:
    for name, source in (("public_constructor", public_constructor), ("boxed_drop", boxed_drop),
                         ("tag_specs", tag_specs), ("physical_projection", physical_projection),
                         ("erased_call", erased_call), ("read_projection", read_projection),
                         ("driver_template", driver_template)):
        expect_tokens(source, expected["items"][name], f"proof source `{name}`")
    expect_tokens(shadow, expected["items"]["driver_template"], "elaborated proof client")
    require(shadow == driver_template,
            "generated proof client must be the exact unmutated driver template in the canonical run")
    client, _, _ = AI.find_function(shadow, "boxed_read_then_drop")
    for forbidden in ("assume", "axiom", "extern_spec", "externspec", "checktrusted"):
        require(forbidden not in client, f"client shadow contains forbidden escape `{forbidden}`")
    require("trusted" not in client and "Drop" not in client,
            "terminal client shadow may not add trusted summaries or a Bytes Drop implementation")
    ops = [
        "let bytes = original_bytes_from_box(input)",
        "let observed = original_bytes_as_slice(&bytes).to_vec()",
        "let saved_return = observed",
        "bytes_terminal_drop(bytes, completion.borrow_mut())",
    ]
    positions = [AI.find_subsequence(client, AI.rust_tokens(op)) for op in ops]
    require(all(p is not None for p in positions) and positions == sorted(positions) and client[-1] == "saved_return",
            "proof client must read and save return before the single consuming terminal helper")
    attrs = AI.function_outer_attributes(shadow, "boxed_read_then_drop")
    require(attrs == [AI.rust_tokens("#[ensures(result@ == input@)]")],
            "proof client admission/result attributes must remain unrestricted input and content preservation")
    return {
        "shadow_sha256": sha(shadow.encode()),
        "proof_source_hashes": {"public_constructor": sha(public_constructor.encode()),
                                "boxed_drop": sha(boxed_drop.encode()), "tag_specs": sha(tag_specs.encode()),
                                "physical_projection": sha(physical_projection.encode()),
                                "erased_call": sha(erased_call.encode()),
                                "read_projection": sha(read_projection.encode()),
                                "driver_template": sha(driver_template.encode())},
        "terminal_helper": "bytes_terminal_drop",
        "normal_terminal_owner_order": ["bytes"],
        "return_saved_before_terminal_effect": True,
        "unrestricted_client_precondition": True,
        "shadow_adds_trust_or_drop_impl": False,
    }


def audit_mapping(mapping: dict[str, Any], *, native_source: str, mir: dict[str, Any],
                  public_constructor: str, boxed_drop: str, tag_specs: str,
                  driver_template: str, shadow: str) -> dict[str, Any]:
    require(mapping.get("stage") == "before Creusot borrow/liveness; external consuming terminal-place",
            "mapping terminal stage changed")
    require(mapping.get("terminal_feature") == "",
            "canonical correspondence must select the positive body with no negative terminal feature")
    require(mapping.get("native_source") == "native.rs" and mapping.get("native_source_sha256") == sha(native_source.encode()),
            "mapping native client path/hash does not match current source")
    require(mapping.get("normal_edges") == [dict(block="bb4", place="_2", owner="bytes", successor="bb5",
            unwind="bb9", helper="bytes_terminal_drop", output="completion")],
            "mapping normal terminal edge differs from independently parsed MIR")
    require(mapping.get("return_evaluation") == {"block": "bb3", "operation": "_0 = move _4"},
            "mapping return evaluation differs from client MIR")
    expected_inputs = [("src/public_constructor.rs", public_constructor), ("src/boxed_drop.rs", boxed_drop),
                       ("src/tag_specs.rs", tag_specs), ("driver-template.rs", driver_template)]
    inputs = mapping.get("source_inputs")
    require(isinstance(inputs, list) and inputs == [
        {"path": path, "sha256": sha(source.encode())} for path, source in expected_inputs],
        "mapping source inputs/path hashes differ from independently inspected proof sources")
    require(mapping.get("shadow") == {"path": "generated/boxed_client.rs", "sha256": sha(shadow.encode())},
            "mapping proof-client source/hash differs from selected checker input")
    require(mapping.get("native_capture") == mir["capture_data"],
            "mapping embedded native capture differs from the independently read and checked capture report")
    require(mapping.get("representational_routes") == [
        "empty Static -> static_drop no allocation",
        "nonempty even PromotableRaw -> clear low bit -> free_boxed_slice",
        "nonempty odd PromotableRaw -> raw pointer -> free_boxed_slice"],
        "mapping branch claims changed")
    require(mapping.get("unreachable_native_branch") ==
            "KIND_ARC -> release_shared retained in native source/MIR; readonly binding proves kind == KIND_VEC in each checked callback",
            "mapping must retain the ARC branch and identify proof-side unreachability")
    require(mapping.get("generic_tcb") == [
        "terminal-place address non-observation and no field glue",
        "native source/MIR/shadow mapping and erasure",
        "assumed exposed-provenance tag roundtrip",
        "equal pointer distance",
        "existing physical read/free and readonly atomic binding"],
        "mapping TCB list changed or overstates tag provenance")
    require(mapping.get("exclusions") == ["unwind", "Clone/promotion", "arbitrary move/drop glue", "non-default cfg", "full-crate admission"],
            "mapping exclusions changed")
    return {"mapping_sha256": sha(json.dumps(mapping, sort_keys=True).encode()),
            "normal_edge_rederived": True, "return_evaluation_rederived": True,
            "source_mapping_rederived": True, "mapping_hashes_not_used_as_sole_evidence": True}


def audit_all_data(*, native_source: str, mir_dir: pathlib.Path, lib_source: str,
                   public_constructor: str, boxed_drop: str, tag_specs: str,
                   physical_projection: str, erased_call: str, read_projection: str,
                   driver_template: str, shadow: str, mapping: dict[str, Any],
                   expected: dict[str, Any], expected_mir: dict[str, Any],
                   expected_shadow: dict[str, Any], expected_production: dict[str, Any],
                   expected_provenance: dict[str, Any], expected_generated: dict[str, Any],
                   generated_sources: dict[str, str], source_map: dict[str, Any],
                   production_sources: dict[str, str],
                   mir_sources: dict[str, str] | None = None,
                   capture_data: dict[str, Any] | None = None,
                   provenance_data: dict[str, str] | None = None) -> dict[str, Any]:
    global AI
    AI = load_ai_checker()
    native = audit_native_client(native_source, expected)
    production = audit_production_sources(
        bytes_source=production_sources["bytes.rs"],
        bytes_record=production_sources["bytes_record.rs"],
        vtable_record=production_sources["vtable_record.rs"],
        loom_source=production_sources["loom.rs"], expected=expected_production)
    extraction = audit_constructor_extractions(
        bytes_source=production_sources["bytes.rs"],
        bytes_record_source=production_sources["bytes_record.rs"],
        vtable_record_source=production_sources["vtable_record.rs"],
        bytes_mut_source=production_sources["bytes_mut.rs"],
        generated_sources=generated_sources, source_map=source_map,
        expected_generated=expected_generated)
    if provenance_data is not None:
        AI._provenance_mutation = provenance_data
    else:
        AI._provenance_mutation = None
    mir = audit_mir(mir_dir, native_source, production_sources["bytes.rs"], expected_mir,
                    sources_override=mir_sources, capture_override=capture_data)
    provenance = audit_native_provenance(expected_provenance, lib_source)
    shadow_facts = audit_shadow_sources(public_constructor=public_constructor,
        boxed_drop=boxed_drop, tag_specs=tag_specs, physical_projection=physical_projection,
        erased_call=erased_call, read_projection=read_projection, driver_template=driver_template,
        shadow=shadow, expected=expected_shadow)
    mapping_facts = audit_mapping(mapping, native_source=native_source, mir=mir,
        public_constructor=public_constructor, boxed_drop=boxed_drop, tag_specs=tag_specs,
        driver_template=driver_template, shadow=shadow)
    return {
        "status": "correspondence_pass", "native_client": native,
        "production_drop_sources": production, "constructor_source_extraction": extraction,
        "native_mir": mir,
        "native_toolchain_harness_and_module_route": provenance,
        "proof_shadow": shadow_facts, "generated_mapping": mapping_facts,
        "claim_boundary": [
            "selected default-native Box normal-return path only; unwind correctness excluded",
            "native runtime coverage is corroboration and does not force even/odd pointer parity",
            "Clone/promotion, arbitrary Bytes move/drop glue, other cfgs, and whole-crate admission excluded",
            "selected MIR/source/ghost-erasure terminal-place relation remains a generic TCB premise",
            "the tag roundtrip assumes exposed provenance selection; it is not strict-provenance preservation",
            "physical read/free and read-only atomic cursor boundaries remain generic TCB items",
            "source checker is structural correspondence, not a proof of compiler correctness",
        ],
    }


def collect_inputs(shadow_path: pathlib.Path, mapping_path: pathlib.Path) -> dict[str, Any]:
    return dict(
        native_source=(ROOT / "native.rs").read_text(), mir_dir=MIR_DIR,
        lib_source=(ROOT / "src/lib.rs").read_text(),
        public_constructor=(ROOT / "src/public_constructor.rs").read_text(),
        boxed_drop=(ROOT / "src/boxed_drop.rs").read_text(),
        tag_specs=(ROOT / "src/tag_specs.rs").read_text(),
        physical_projection=(ROOT / "src/physical_projection.rs").read_text(),
        erased_call=(ROOT / "src/erased_call.rs").read_text(),
        read_projection=(ROOT / "src/read_projection.rs").read_text(),
        driver_template=(ROOT / "driver-template.rs").read_text(), shadow=shadow_path.read_text(),
        mapping=json.loads(mapping_path.read_text()), expected=json.loads(EXPECTED.read_text()),
        expected_mir=json.loads(EXPECTED_MIR.read_text()),
        expected_shadow=json.loads(EXPECTED_SHADOW.read_text()),
        expected_production=json.loads(EXPECTED_PRODUCTION.read_text()),
        expected_provenance=json.loads(EXPECTED_PROVENANCE.read_text()),
        expected_generated=json.loads(EXPECTED_GENERATED.read_text()),
        generated_sources={name: (GENERATED / name).read_text() for name in (
            "public_traits.rs", "public_records.rs", "shared_constructor_component.rs",
            "native_constructor_bindings.rs")},
        source_map=json.loads((GENERATED / "source-map.json").read_text()),
        mir_sources={filename:(MIR_DIR/filename).read_text() for filename in MIR_FILES.values()},
        capture_data=json.loads((MIR_DIR/"capture.json").read_text()),
        provenance_data={
            "rustc_text":(MIR_DIR/"rustc-version.txt").read_text(),
            "cargo_text":(MIR_DIR/"cargo-version.txt").read_text(),
            "manifest_text":(HARNESS/"Cargo.toml").read_text(),
            "lock_text":(HARNESS/"Cargo.lock").read_text(),
            "harness_source":(HARNESS/"src/main.rs").read_text(),
            "native_run_log":(GENERATED/"native-run.log").read_text(),
        },
        production_sources={
            "bytes.rs": (CRATE_ROOT / "src/bytes.rs").read_text(),
            "bytes_record.rs": (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
            "vtable_record.rs": (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text(),
            "bytes_mut.rs": (CRATE_ROOT / "src/bytes_mut.rs").read_text(),
            "loom.rs": (CRATE_ROOT / "src/loom.rs").read_text(),
        })


def load_inputs(shadow_path: pathlib.Path, mapping_path: pathlib.Path) -> dict[str, Any]:
    return audit_all_data(**collect_inputs(shadow_path, mapping_path))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shadow", type=pathlib.Path, default=POSITIVE_SHADOW)
    parser.add_argument("--mapping", type=pathlib.Path, default=POSITIVE_MAPPING)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        result = load_inputs(args.shadow, args.mapping)
    except Exception as exc:
        print(json.dumps({"status": "correspondence_failed", "error": str(exc)}, indent=2))
        return 2
    output = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.write_text(output)
    print(output, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
