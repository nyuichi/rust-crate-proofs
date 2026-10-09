#!/usr/bin/env python3
"""Independent source correspondence gate for the AL first-promotion proof.

The gate binds the selected shadow to the bounded native Box/clone/read/drop
witness, checks the affine proof state and its source-level event wiring, and
then runs the frozen native source/MIR checker. It does not prove compiler MIR
adequacy or discharge the generic AtomicPtr, allocator, pointer provenance,
callback-erasure, or physical-resource TCB.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import shutil
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
AI_PATH = ROOT.parent / "original-shared-scoped-client-2026-10-09" / "check_correspondence.py"
NATIVE_PATH = ROOT / "check_native.py"


class CheckError(RuntimeError):
    pass


EXPECTED_MODULE_TOKEN_HASHES = {
    "promotion": "43c7d5d4e779b42f2d16343f1703b245ed9ac853fdbcbbb49c9101c14247f73d",
    "lifecycle": "f0e1c6033c68d3098d1999169a98acba08f6bf9dcece98c8d910b1b5c075c71c",
    "owned_pointer": "b2433feb8cc60bf80a781e86f042d1893ce4f8005b00fef818791a8b20f3a632",
    "erased_call": "ed51249e9b5178b81b5c262c1ebe59f566418146322cff324c40bd88c718230a",
    "event": "4c58c1ef28e68068e4e4ddda687123ed74d68e7277b679d4407906222966549f",
    "provenance_specs": "87f748a25b88254dc5d912f5c49b14a99234a422e19626a41c4d812e34c9ab76",
    "field_event": "20c3f88d7ced2d93b4721c9cf9bb82bb94a55b364af03f4d1bd452bef76989d1",
    "physical_projection": "63876916c9a3851372fcfdeb2d6b35f91dcc7b2457b1b03c707e137809c7c8e8",
    "free_effect": "77b90c3e95c37afc08e31312f0f1d4460035241bebcf2886694d11dc376e1aa0",
    "pointer_event": "58af472bbc35f9d76bb6a30cd927a045e4e4856d9d9fd52dece41a8a394e445d",
    "raw_vec": "18ab32b1054e99e71686829562a2ab3dde4db0594eff9af66a73014cc6976fd5",
    "boxed_alignment": "bd66e45a49af701ca1ae707ffadc0bd272b3be84e43cdc20e42048df431a2dd1",
    "promotion_tags": "316a445f4995f4417c476d6cc92764e5ea6b5f3a1b21bfcfab9c879f8c2c0ee8",
}

EXPECTED_EFFECT_BODY_HASHES = {
    ("promotion", "shallow_clone_vec_checked"): "ffe31fc3f432dde32ef1890f1bc53980c04f07a4d369968265d8ba4caa15220c",
    ("promotion", "even_clone_checked"): "6ccd534a71ebe0a8b942808a7c6c315a8347749fdf6e25c77ca352711354882b",
    ("promotion", "odd_clone_checked"): "053f8633b2dbf9ece5a6c1b68cdabd65b4a9cba12548fb6be3e8b2ed5f4b9311",
    ("promotion", "release_core"): "f0916a97da610192b8f90295dfa187c72a016f6862467f6fa069d5387b1700c7",
    ("promotion", "free_recovered"): "3e71614b5db85b9d05b5f25c89192ff1904f01f2787a50c961dc36961b64acdc",
    ("lifecycle", "initialize_pair"): "0c2edfb776501c23813c9c2ab6f35e406d6c51f5b1d830860d301d1cabbbb1eb",
    ("owned_pointer", "exchange_known"): "7f20c7d38c077c0eb16815b02f6412e6996196a561a0b773f6670b8175b60861",
    ("raw_vec", "detach_boxed_slice"): "444c5d3c84bb783f76f2de019f977b7754fb16381a047e880b8efda6ae672361",
}

EXPECTED_BUILD_INPUT_SHA256 = {
    "build.rs": "9738750d6d526fc2605cee4528e9757296cd3181ee259f23bda5a5c457806925",
    "extract_public.py": "8bb04cfb359affd3a0eb2e7cdf03c01c0364a951d58f74bb2d1f37f45f4cd079",
}

EXPECTED_BUILD_LITERAL_LEXEMES = [
    '"cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)"',
    '"cargo:rustc-cfg=bytes_original_shared_gate"',
    '"python3"', '"extract_public.py"',
    '"cargo:rerun-if-changed=../../../src/bytes.rs"',
    '"cargo:rerun-if-changed=../../../src/bytes/bytes_record.rs"',
    '"cargo:rerun-if-changed=../../../src/bytes/vtable_record.rs"',
    '"cargo:rerun-if-changed=../../../src/bytes_mut.rs"',
    '"cargo:rerun-if-changed=extract_public.py"',
]
EXPECTED_BUILD_RERUN_PATHS = [
    "../../../src/bytes.rs",
    "../../../src/bytes/bytes_record.rs",
    "../../../src/bytes/vtable_record.rs",
    "../../../src/bytes_mut.rs",
    "extract_public.py",
]


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CheckError(message)


def load_module(name: str, path: pathlib.Path):
    require(path.is_file(), f"missing checker dependency: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


AI = load_module("al_promotion_source_parser", AI_PATH)
NATIVE = load_module("al_promotion_native_audit", NATIVE_PATH)


def tokens(text: str) -> list[str]:
    return AI.rust_tokens(text)


def token_count(haystack: list[str], needle: list[str]) -> int:
    if not needle:
        return 0
    return sum(haystack[i:i + len(needle)] == needle
               for i in range(len(haystack) - len(needle) + 1))


def has(haystack: list[str], snippet: str, label: str, count: int | None = None) -> None:
    wanted = tokens(snippet)
    actual = token_count(haystack, wanted)
    if count is None:
        require(actual > 0, f"{label}: missing `{snippet}`")
    else:
        require(actual == count, f"{label}: expected {count} occurrence(s) of `{snippet}`, found {actual}")


def ordered(haystack: list[str], snippets: list[str], label: str) -> None:
    cursor = 0
    for snippet in snippets:
        wanted = tokens(snippet)
        found = next((i for i in range(cursor, len(haystack) - len(wanted) + 1)
                      if haystack[i:i + len(wanted)] == wanted), None)
        require(found is not None, f"{label}: required operation order breaks at `{snippet}`")
        cursor = found + len(wanted)


def body(source: str, name: str) -> list[str]:
    try:
        return AI.find_function(source, name)[0]
    except Exception as exc:
        raise CheckError(f"cannot uniquely extract function `{name}`: {exc}") from exc


def attrs(source: str, name: str) -> list[list[str]]:
    try:
        result = AI.function_outer_attributes(source, name)
        if result:
            return result
        # The frozen parser intentionally recognizes only plain visibility
        # decorations before `fn`. Normalize one code-position `unsafe` token
        # so the same contiguous #[...] parser can inspect unsafe adapters.
        code = AI.mask_noncode(source)
        matches = list(re.finditer(r"\bunsafe\s+fn\s+" + re.escape(name) + r"\b", code))
        if len(matches) == 1:
            start = matches[0].start()
            normalized = source[:start] + " " * len("unsafe ") + source[start + len("unsafe "):]
            result = AI.function_outer_attributes(normalized, name)
        return result
    except Exception as exc:
        raise CheckError(f"cannot extract attributes for `{name}`: {exc}") from exc


def exact_attrs(source: str, name: str, expected: list[str]) -> None:
    got = attrs(source, name)
    want = [tokens(x) for x in expected]
    require(got == want, f"`{name}` attributes/contracts differ from the reviewed registration surface")


def attrs_have(source: str, name: str, snippet: str, label: str | None = None) -> None:
    flattened = [tok for attr in attrs(source, name) for tok in attr]
    require(token_count(flattened, tokens(snippet)) > 0,
            f"{label or name}: attribute/contracts lost `{snippet}`")


def exact_body(source: str, name: str, expected: str, label: str | None = None) -> None:
    require(body(source, name) == tokens(expected),
            f"{label or name}: executable body differs from the reviewed bounded adapter")


def assert_attr_token_hash(source: str, name: str, expected: str, label: str) -> None:
    canonical = " ".join(" ".join(attr) for attr in attrs(source, name))
    actual = hashlib.sha256(canonical.encode()).hexdigest()
    require(actual == expected, f"{label} complete attribute/contract surface changed ({actual})")


def assert_body_token_hash(source: str, name: str, expected: str, label: str) -> None:
    actual = hashlib.sha256(" ".join(body(source, name)).encode()).hexdigest()
    require(actual == expected, f"{label} complete executable token body changed ({actual})")


def assert_module_token_surface(name: str, source: str) -> None:
    expected = EXPECTED_MODULE_TOKEN_HASHES.get(name)
    require(expected is not None, f"no reviewed token surface registered for `{name}`")
    actual = hashlib.sha256(" ".join(tokens(source)).encode()).hexdigest()
    require(actual == expected,
            f"complete reviewed `{name}` module token surface changed ({actual})")


def type_item(source: str, prefix: str, label: str) -> list[str]:
    try:
        return NATIVE.extract_item_tokens(source, prefix, label)
    except Exception as exc:
        raise CheckError(f"cannot extract type `{label}`: {exc}") from exc


def comma_parts(items: list[str]) -> list[list[str]]:
    """Split token sequence at commas outside (), [], {}, and generic angles."""
    out: list[list[str]] = []
    start = 0
    round_depth = square_depth = brace_depth = angle_depth = 0
    for i, tok in enumerate(items):
        if tok == "(": round_depth += 1
        elif tok == ")": round_depth -= 1
        elif tok == "[": square_depth += 1
        elif tok == "]": square_depth -= 1
        elif tok == "{": brace_depth += 1
        elif tok == "}": brace_depth -= 1
        elif tok == "<": angle_depth += 1
        elif tok == ">" and angle_depth: angle_depth -= 1
        elif tok == "," and (round_depth, square_depth, brace_depth, angle_depth) == (0, 0, 0, 0):
            if start < i:
                out.append(items[start:i])
            start = i + 1
    if start < len(items):
        out.append(items[start:])
    return out


def struct_fields(source: str, name: str) -> list[str]:
    prefix = f"struct {name} {{"
    item = type_item(source, prefix, name)
    opening = item.index("{")
    closing = len(item) - 1
    fields = comma_parts(item[opening + 1:closing])
    names = []
    for field in fields:
        require(len(field) > 1 and field[1] == ":", f"`{name}` contains a non-field or malformed field")
        names.append(field[0])
    return names


def struct_field_types(source: str, name: str) -> dict[str, list[str]]:
    prefix = f"struct {name} {{"
    item = type_item(source, prefix, name)
    fields = comma_parts(item[item.index("{") + 1:-1])
    result: dict[str, list[str]] = {}
    for field in fields:
        require(len(field) > 2 and field[1] == ":", f"`{name}` contains a malformed typed field")
        require(field[0] not in result, f"`{name}` has duplicate field `{field[0]}`")
        result[field[0]] = field[2:]
    return result


def external_struct_profile(source: str, prefix: str, label: str) -> list[dict[str, Any]]:
    """Extract fields plus field attributes from a selected production record."""
    try:
        item = NATIVE.extract_item_tokens(source, prefix, label)
    except Exception as exc:
        raise CheckError(f"cannot extract production record `{label}`: {exc}") from exc
    opening = item.index("{")
    require(item[-1] == "}", f"production record `{label}` is not a closed braced item")
    result: list[dict[str, Any]] = []
    for part in comma_parts(item[opening + 1:-1]):
        attrs_found: list[list[str]] = []
        cursor = 0
        while part[cursor:cursor + 2] == ["#", "["]:
            depth = 0
            end = None
            for i in range(cursor + 1, len(part)):
                if part[i] == "[": depth += 1
                elif part[i] == "]":
                    depth -= 1
                    if depth == 0:
                        end = i + 1
                        break
            require(end is not None, f"malformed field attribute in production `{label}`")
            attrs_found.append(part[cursor:end])
            cursor = end
        remainder = part[cursor:]
        try:
            colon = remainder.index(":")
        except ValueError as exc:
            raise CheckError(f"malformed field in production `{label}`") from exc
        result.append({"prefix": remainder[:colon], "type": remainder[colon + 1:], "attrs": attrs_found})
    return result


def rust_literal_lexemes(source: str) -> list[str]:
    """Return code-position Rust literal spellings, preserving string contents."""
    literals: list[str] = []
    i, n = 0, len(source)
    while i < n:
        if source[i].isspace():
            i += 1
            continue
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = n if end < 0 else end + 1
            continue
        if source.startswith("/*", i):
            depth = 1
            i += 2
            while i < n and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            require(depth == 0, "unterminated block comment while extracting Rust literal routes")
            continue
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", source[i:])
        if raw:
            hashes = raw.group(1)
            start = i
            i += len(raw.group(0))
            end = source.find('"' + hashes, i)
            require(end >= 0, "unterminated raw Rust literal in selected source")
            i = end + len(hashes) + 1
            literals.append(source[start:i])
            continue
        prefix_len = 0
        if source.startswith(("b\"", "c\""), i):
            prefix_len = 1
        if source[i + prefix_len] == '"':
            start = i
            i += prefix_len + 1
            escaped = False
            while i < n:
                char = source[i]
                i += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    break
            else:
                raise CheckError("unterminated Rust string literal in selected source")
            literals.append(source[start:i])
            continue
        if source[i] == "'":
            start = i
            j = i + 1
            if j < n and source[j] == "\\":
                j += 2
                while j < n and source[j].isalnum():
                    j += 1
            else:
                j += 1
            if j < n and source[j] == "'":
                literals.append(source[start:j + 1])
                i = j + 1
                continue
        i += 1
    return literals


def literal_aware_tokens(source: str) -> list[str]:
    """Use the shared lexical scanner while restoring exact literal spellings."""
    result = tokens(source)
    literals = iter(rust_literal_lexemes(source))
    out = []
    for tok in result:
        if tok == "<literal>":
            try:
                out.append("<literal:" + next(literals) + ">")
            except StopIteration as exc:
                raise CheckError("Rust token/literal scan disagreement") from exc
        else:
            out.append(tok)
    try:
        next(literals)
    except StopIteration:
        return out
    raise CheckError("Rust token/literal scan left unmatched literal spellings")


def macro_argument_tokens(source: str, macro_name: str) -> list[list[str]]:
    """Extract macro argument token streams from code positions, not comments."""
    code = AI.mask_noncode(source)
    pattern = re.compile(r"\b" + re.escape(macro_name) + r"\s*!\s*\(")
    found: list[list[str]] = []
    for match in pattern.finditer(code):
        opening = code.find("(", match.start(), match.end())
        depth = 0
        closing = None
        for i in range(opening, len(code)):
            if code[i] == "(":
                depth += 1
            elif code[i] == ")":
                depth -= 1
                if depth == 0:
                    closing = i
                    break
        require(closing is not None, f"unclosed `{macro_name}!` macro invocation")
        found.append(literal_aware_tokens(source[opening + 1:closing]))
    return found


EXPECTED_INCLUDE_EXPRESSIONS = [
    ['<literal:"../../../../src/bytes/shared_record.rs">'],
    ["concat", "!", "(", "env", "!", "(", '<literal:"OUT_DIR">', ")", ",",
     '<literal:"/public_records.rs">', ")"],
]


def assert_selected_include_routes(source: str, source_path: pathlib.Path, label: str) -> dict[str, Any]:
    actual = macro_argument_tokens(source, "include")
    require(actual == EXPECTED_INCLUDE_EXPRESSIONS,
            f"{label} literal include! routes changed: {actual}")
    shared_literal = "../../../../src/bytes/shared_record.rs"
    resolved_shared = (source_path.parent / shared_literal).resolve()
    expected_shared = (CRATE_ROOT / "src/bytes/shared_record.rs").resolve()
    require(resolved_shared == expected_shared and resolved_shared.is_file(),
            f"{label} shared record include resolves to a different production source: {resolved_shared}")
    return {
        "shared_record_literal": shared_literal,
        "shared_record_resolves_to_production_source": True,
        "public_records_expression": 'concat!(env!("OUT_DIR"),"/public_records.rs")',
    }


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def assert_cargo_manifest_routes(probe_manifest_text: str, production_manifest_text: str) -> dict[str, str]:
    try:
        probe_manifest = tomllib.loads(probe_manifest_text)
        production_manifest = tomllib.loads(production_manifest_text)
    except tomllib.TOMLDecodeError as exc:
        raise CheckError(f"Cargo manifest route cannot be parsed: {exc}") from exc
    probe_package = probe_manifest.get("package", {})
    require(probe_package.get("name") == "bytes-original-promotable-first-clone" and
            probe_package.get("version") == "0.1.0",
            "probe Cargo package identity changed")
    require("build" not in probe_package and "lib" not in probe_manifest,
            "probe Cargo must use its default build.rs and src/lib.rs routes")
    require((ROOT / "build.rs").is_file() and (ROOT / "src/lib.rs").is_file(),
            "probe default build.rs or src/lib.rs target is missing")
    prod_package = production_manifest.get("package", {})
    prod_lib = production_manifest.get("lib", {})
    require(prod_package.get("name") == "bytes" and prod_package.get("version") == "1.11.1" and
            prod_package.get("build") is False,
            "production crate identity/build-script selection changed")
    require(prod_lib.get("path") == "src/lib.rs",
            "production package no longer selects its actual src/lib.rs entrypoint")
    require((CRATE_ROOT / "src/lib.rs").is_file(), "production src/lib.rs is missing")
    return {"probe_package": probe_package["name"], "probe_lib_entry": "Cargo default src/lib.rs",
            "probe_build_entry": "Cargo default build.rs", "production_package": prod_package["name"],
            "production_lib_entry": "src/lib.rs", "production_build_script": False}


def assert_build_script_surface(source_bytes: bytes) -> list[str]:
    actual_sha = sha256_bytes(source_bytes)
    require(actual_sha == EXPECTED_BUILD_INPUT_SHA256["build.rs"],
            f"build.rs actual source changed ({actual_sha})")
    source = source_bytes.decode()
    require(rust_literal_lexemes(source) == EXPECTED_BUILD_LITERAL_LEXEMES,
            "build.rs code-position string/rerun routes changed")
    return list(EXPECTED_BUILD_RERUN_PATHS)


def assert_extractor_source_surface(source_bytes: bytes) -> None:
    actual_sha = sha256_bytes(source_bytes)
    require(actual_sha == EXPECTED_BUILD_INPUT_SHA256["extract_public.py"],
            f"extract_public.py actual source changed ({actual_sha})")


def assert_public_records_build_pipeline() -> dict[str, Any]:
    """Bind Cargo routing, the extractor, and its live OUT_DIR record bytes."""
    probe_manifest_path = ROOT / "Cargo.toml"
    production_manifest_path = CRATE_ROOT / "Cargo.toml"
    cargo_routes = assert_cargo_manifest_routes(probe_manifest_path.read_text(),
                                                production_manifest_path.read_text())

    build_path = ROOT / "build.rs"
    extractor_path = ROOT / "extract_public.py"
    build_bytes = build_path.read_bytes()
    extractor_bytes = extractor_path.read_bytes()
    build_sha = sha256_bytes(build_bytes)
    extractor_sha = sha256_bytes(extractor_bytes)
    rerun_values = assert_build_script_surface(build_bytes)
    assert_extractor_source_surface(extractor_bytes)
    expected_inputs = {
        "../../../src/bytes.rs": CRATE_ROOT / "src/bytes.rs",
        "../../../src/bytes/bytes_record.rs": CRATE_ROOT / "src/bytes/bytes_record.rs",
        "../../../src/bytes/vtable_record.rs": CRATE_ROOT / "src/bytes/vtable_record.rs",
        "../../../src/bytes_mut.rs": CRATE_ROOT / "src/bytes_mut.rs",
        "extract_public.py": extractor_path,
    }
    require(rerun_values == list(expected_inputs), "build.rs rerun source order/set changed")
    input_hashes: dict[str, str] = {}
    for relative, expected_path in expected_inputs.items():
        resolved = (ROOT / relative).resolve()
        require(resolved == expected_path.resolve() and resolved.is_file(),
                f"build rerun input {relative!r} resolves to the wrong source: {resolved}")
        input_hashes[relative] = sha256_bytes(resolved.read_bytes())

    # Rebuild the generated record bytes directly from the production source
    # fragments, including the runtime Shared/vtable/Bytes fields and the
    # BytesMut namespace. This is byte equality, in addition to the existing
    # independent token comparison performed by audit_generated_extractions.
    bytes_record_source = (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text()
    vtable_record_source = (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text()
    shared_record_source = (CRATE_ROOT / "src/bytes/shared_record.rs").read_text()
    bytes_mut_source = (CRATE_ROOT / "src/bytes_mut.rs").read_text()
    mutable_shared = AI.extract_struct_source(bytes_mut_source, "struct Shared {", "BytesMut::Shared")
    mutable_record = AI.extract_struct_source(bytes_mut_source, "pub struct BytesMut {", "BytesMut")
    expected_records = (
        bytes_record_source + "\n" + vtable_record_source + "\n" +
        "mod mutable_record {\nuse alloc::vec::Vec;\n"
        "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        mutable_shared + "\n" + mutable_record + "\n}\nuse mutable_record::BytesMut;\n"
    ).encode()
    expected_profiles = {
        "Shared": (shared_record_source, "pub(crate) struct Shared", [
            ("buf", "*mut u8", []), ("cap", "usize", []), ("pub(crate) ref_cnt", "AtomicUsize", []),
        ]),
        "Bytes": (bytes_record_source, "pub struct Bytes {", [
            ("ptr", "*const u8", []), ("len", "usize", []),
            ("data", "AtomicPtr<()> ", []), ("vtable", "&'static Vtable", []),
            ("original_frozen", "Option<OriginalFrozenProof>", ["#[cfg(all(creusot,bytes_original_freeze_gate))]"]),
            ("original_bytes", "creusot_std::prelude::Ghost<OriginalBytesProof>", ["#[cfg(all(creusot,bytes_original_constructor_gate))]"]),
            ("original_shared", "creusot_std::prelude::Ghost<OriginalSharedProof>", ["#[cfg(all(creusot,bytes_original_shared_gate))]"]),
        ]),
        "Vtable": (vtable_record_source, "pub(crate) struct Vtable", [
            ("pub clone", "unsafe fn(&AtomicPtr<()>,*const u8,usize)->Bytes", []),
            ("pub into_vec", "unsafe fn(&AtomicPtr<()>,*const u8,usize)->Vec<u8>", []),
            ("pub into_mut", "unsafe fn(&AtomicPtr<()>,*const u8,usize)->BytesMut", []),
            ("pub is_unique", "unsafe fn(&AtomicPtr<()>)->bool", []),
            ("pub drop", "unsafe fn(&mut AtomicPtr<()>,*const u8,usize)", []),
        ]),
    }
    actual_profiles: dict[str, list[dict[str, Any]]] = {}
    for name, (profile_source, prefix, expected) in expected_profiles.items():
        actual_profile = external_struct_profile(profile_source, prefix, name)
        normalized_expected = [
            {"prefix": tokens(field_prefix), "type": tokens(field_type), "attrs": [tokens(a) for a in field_attrs]}
            for field_prefix, field_type, field_attrs in expected
        ]
        require(actual_profile == normalized_expected,
                f"actual production `{name}` runtime/Ghost field or callback profile changed")
        actual_profiles[name] = actual_profile
    generated_path = ROOT / "generated/public_records.rs"
    source_map_path = ROOT / "generated/source-map.json"
    require(generated_path.is_file() and source_map_path.is_file(),
            "extractor source copy or source-map receipt is missing")
    generated_bytes = generated_path.read_bytes()
    require(generated_bytes == expected_records,
            "generated/public_records.rs bytes differ from direct production-source reconstruction")
    generated_sha = sha256_bytes(generated_bytes)
    try:
        source_map = json.loads(source_map_path.read_text())
    except (json.JSONDecodeError, OSError) as exc:
        raise CheckError(f"extractor source-map receipt cannot be read: {exc}") from exc
    require(source_map.get("generated/public_records.rs", {}).get("sha256") == generated_sha,
            "extractor source-map record hash differs from reconstructed public_records.rs")

    target_env = os.environ.get("CARGO_TARGET_DIR")
    target_dir = pathlib.Path(target_env).resolve() if target_env else (ROOT.parents[5] / "bytes-proof-tools/targets/bytes").resolve()
    require(target_dir.is_dir(), f"active Cargo target directory is unavailable: {target_dir}")
    run_fingerprints = list((target_dir / "debug/.fingerprint").glob(
        "bytes-original-promotable-first-clone-*/run-build-script-build-script-build.json"))
    require(len(run_fingerprints) == 1,
            f"expected one live Cargo build-script receipt under {target_dir}, found {len(run_fingerprints)}")
    fingerprint_path = run_fingerprints[0]
    fingerprint_bytes = fingerprint_path.read_bytes()
    try:
        fingerprint = json.loads(fingerprint_bytes)
    except (json.JSONDecodeError, OSError) as exc:
        raise CheckError(f"Cargo build-script fingerprint cannot be read: {exc}") from exc
    rerun_records = [entry["RerunIfChanged"] for entry in fingerprint.get("local", [])
                     if isinstance(entry, dict) and "RerunIfChanged" in entry]
    require(len(rerun_records) == 1, "Cargo fingerprint lacks a unique live RerunIfChanged record")
    rerun_record = rerun_records[0]
    require(rerun_record.get("paths") == rerun_values,
            "Cargo build-script fingerprint input set differs from the selected build.rs routes")
    output_rel = pathlib.Path(rerun_record.get("output", ""))
    require(not output_rel.is_absolute() and output_rel.parts[:2] == ("debug", "build"),
            "Cargo build-script output receipt is not under the selected debug/build directory")
    build_output_path = (target_dir / output_rel).resolve()
    build_output_dir = build_output_path.parent
    require(fingerprint_path.parent.name == build_output_dir.name,
            "Cargo run-build fingerprint does not identify the selected build output directory")
    require(build_output_path.is_file(), f"Cargo build-script output record is missing: {build_output_path}")
    build_output_bytes = build_output_path.read_bytes()
    expected_output_lines = [
        "cargo:rustc-check-cfg=cfg(bytes_original_shared_gate)",
        "cargo:rustc-cfg=bytes_original_shared_gate",
        "cargo:rerun-if-changed=../../../src/bytes.rs",
        "cargo:rerun-if-changed=../../../src/bytes/bytes_record.rs",
        "cargo:rerun-if-changed=../../../src/bytes/vtable_record.rs",
        "cargo:rerun-if-changed=../../../src/bytes_mut.rs",
        "cargo:rerun-if-changed=extract_public.py",
    ]
    require(build_output_bytes == ("\n".join(expected_output_lines) + "\n").encode(),
            "actual Cargo build-script output directives differ from the reviewed source routes")
    root_output_path = build_output_dir / "root-output"
    require(root_output_path.is_file(), f"Cargo root-output receipt is missing: {root_output_path}")
    root_output_bytes = root_output_path.read_bytes()
    require(root_output_bytes == str(build_output_dir / "out").encode(),
            "Cargo root-output receipt does not identify this build script OUT_DIR")
    compiled_path = build_output_dir / "out/public_records.rs"
    require(compiled_path.is_file(), f"actual compiled OUT_DIR public_records.rs is missing: {compiled_path}")
    compiled_bytes = compiled_path.read_bytes()
    require(compiled_bytes == expected_records and compiled_bytes == generated_bytes,
            "actual compiled OUT_DIR record bytes differ from reconstructed/generated production records")
    compiled_sha = sha256_bytes(compiled_bytes)

    # Keep the exact bytes consumed by include!(concat!(env!("OUT_DIR"), ...))
    # in the evidence tree so a later archive need not rely on Cargo's cache.
    capture_dir = ROOT / "generated/compiled-inputs"
    capture_dir.mkdir(parents=True, exist_ok=True)
    capture_path = capture_dir / "public_records.rs"
    capture_path.write_bytes(compiled_bytes)
    fingerprint_capture_path = capture_dir / "cargo-run-build-fingerprint.json"
    fingerprint_capture_path.write_bytes(fingerprint_bytes)
    output_capture_path = capture_dir / "cargo-build-output.txt"
    output_capture_path.write_bytes(build_output_bytes)
    root_output_capture_path = capture_dir / "cargo-root-output.txt"
    root_output_capture_path.write_bytes(root_output_bytes)
    receipt_path = capture_dir / "public-records-build-receipt.json"
    receipt = {
        "status": "pass",
        "cargo_package": cargo_routes["probe_package"],
        "probe_manifest": str(probe_manifest_path),
        "crate_entry": "Cargo default src/lib.rs",
        "build_script": str(build_path),
        "build_script_sha256": build_sha,
        "extractor": str(extractor_path),
        "extractor_sha256": extractor_sha,
        "production_manifest": str(production_manifest_path),
        "production_lib_entry": "src/lib.rs",
        "cargo_target_dir": str(target_dir),
        "cargo_build_fingerprint": str(fingerprint_path),
        "cargo_build_fingerprint_sha256": sha256_bytes(fingerprint_bytes),
        "captured_cargo_build_fingerprint_path": str(fingerprint_capture_path.relative_to(ROOT)),
        "captured_cargo_build_fingerprint_sha256": sha256_bytes(fingerprint_capture_path.read_bytes()),
        "build_output_path": str(build_output_path),
        "build_output_sha256": sha256_bytes(build_output_bytes),
        "captured_build_output_path": str(output_capture_path.relative_to(ROOT)),
        "captured_build_output_sha256": sha256_bytes(output_capture_path.read_bytes()),
        "root_output_path": str(root_output_path),
        "root_output_sha256": sha256_bytes(root_output_bytes),
        "captured_root_output_path": str(root_output_capture_path.relative_to(ROOT)),
        "captured_root_output_sha256": sha256_bytes(root_output_capture_path.read_bytes()),
        "actual_out_dir": str(compiled_path.parent),
        "compiled_input_path": str(compiled_path),
        "compiled_input_sha256": compiled_sha,
        "captured_input_path": str(capture_path),
        "captured_input_sha256": sha256_bytes(capture_path.read_bytes()),
        "generated_source_copy_sha256": generated_sha,
        "extractor_source_map_sha256": sha256_bytes(source_map_path.read_bytes()),
        "production_rerun_input_sha256": input_hashes,
        "actual_record_profiles": actual_profiles,
        "captured_compiled_bytes_equal_reconstructed_and_generated": True,
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    return {
        "cargo_routes": cargo_routes,
        "probe_Cargo_default_routes_pinned": True,
        "production_src_lib_route_pinned": True,
        "build_and_extractor_source_hashes": {"build.rs": build_sha, "extract_public.py": extractor_sha},
        "build_inputs_resolve_to_production_sources": True,
        "generated_public_records_byte_reconstruction": True,
        "active_Cargo_OUT_DIR_record_match": True,
        "active_Cargo_receipt": receipt,
        "captured_compiled_input": str(capture_path),
        "record_structs_and_vtable_callbacks": {"Shared": ["buf", "cap", "ref_cnt"],
                                                  "Bytes": ["ptr", "len", "data", "vtable",
                                                           "original_frozen [freeze gate]",
                                                           "original_bytes [constructor gate]",
                                                           "original_shared [selected shared gate]"],
                                                  "Vtable": ["clone", "into_vec", "into_mut", "is_unique", "drop"]},
        "full_record_field_profiles": actual_profiles,
    }


def assert_routes(lib: str, shadow: str) -> dict[str, Any]:
    """Pin the actual module source paths and selected promotion route."""
    expected_paths = {
        "relaxed": "../../original-public-shared-gate-2026-10-08/src/relaxed.rs",
        "ref_count_limit": "../../../../src/ref_count_limit.rs",
        "fraction_map": "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs",
        "release": "../../shared-physical-lifecycle-2026-10-08/src/release.rs",
        "owned_region": "../../../../src/ownership_proof/owned_region.rs",
        "raw_vec": "../../../../src/ownership_proof/raw_vec.rs",
        "boxed_alignment": "../../../../src/ownership_proof/boxed_alignment.rs",
        "pointer_event": "../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs",
        "promotion_tags": "../../original-boxed-automatic-drop-2026-10-09/src/tag_specs.rs",
    }
    for module, expected in expected_paths.items():
        actual = AI.rust_path_for_module(lib, module)
        require(actual == expected, f"`{module}` path changed: {actual!r}")
        resolved = (ROOT / "src" / actual).resolve()
        require(resolved.is_file(), f"`{module}` route resolves to no source: {resolved}")

    # rust_path_for_module binds the path string, but by itself it does not
    # reject a second cfg/path decoration on that same declaration. Pin the
    # complete outer-attribute list for every selected module declaration.
    # This also catches a new #[path] on default modules such as owned_pointer
    # or provenance_specs.
    expected_module_attrs = {
        "event": [],
        "relaxed": ['#[path="../../original-public-shared-gate-2026-10-08/src/relaxed.rs"]'],
        "ref_count_limit": ['#[path="../../../../src/ref_count_limit.rs"]'],
        "fraction_map": ['#[path="../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs"]'],
        "release": ['#[path="../../shared-physical-lifecycle-2026-10-08/src/release.rs"]'],
        "lifecycle": [],
        "owned_region": ['#[path="../../../../src/ownership_proof/owned_region.rs"]'],
        "raw_vec": ['#[path="../../../../src/ownership_proof/raw_vec.rs"]'],
        "boxed_alignment": ['#[path="../../../../src/ownership_proof/boxed_alignment.rs"]'],
        "provenance_specs": [],
        "pointer_event": ['#[path="../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs"]'],
        "field_event": [],
        "physical_projection": [],
        "free_effect": [],
        "erased_call": [],
        "public_shared": ["#[cfg(creusot)]"],
        "owned_pointer": ["#[cfg(creusot)]"],
        "promotion": ["#[cfg(creusot)]"],
        "promotion_tags": ['#[path="../../original-boxed-automatic-drop-2026-10-09/src/tag_specs.rs"]'],
    }
    module_tokens = tokens(lib)
    for module, expected in expected_module_attrs.items():
        declarations = []
        for i in range(len(module_tokens) - 2):
            if module_tokens[i:i + 3] != ["mod", module, ";"]:
                continue
            found_attrs: list[list[str]] = []
            cursor = i
            while cursor >= 2 and module_tokens[cursor - 1] == "]":
                depth = 0
                opening = None
                for j in range(cursor - 1, -1, -1):
                    if module_tokens[j] == "]":
                        depth += 1
                    elif module_tokens[j] == "[":
                        depth -= 1
                        if depth == 0:
                            opening = j
                            break
                require(opening is not None and opening > 0 and module_tokens[opening - 1] == "#",
                        f"cannot parse outer attributes on module `{module}`")
                found_attrs.insert(0, module_tokens[opening - 1:cursor])
                cursor = opening - 1
            declarations.append(found_attrs)
        wanted = [tokens(attr) for attr in expected]
        require(declarations == [wanted],
                f"module `{module}` declaration/attributes changed: {declarations}")

    lib_tokens = tokens(lib)
    masked = AI.mask_noncode(lib)
    for declaration in (
        "mod event;", "mod lifecycle;", "mod field_event;", "mod physical_projection;",
        "mod free_effect;", "mod erased_call;", "#[cfg(creusot)] mod owned_pointer;",
        "#[cfg(creusot)] mod promotion;", "#[cfg(creusot)] mod public_shared;",
    ):
        require(token_count(lib_tokens, tokens(declaration)) == 1,
                f"probe module wiring must contain exactly one `{declaration}`")
    for module in ("event", "lifecycle", "provenance_specs", "field_event", "physical_projection", "free_effect", "erased_call"):
        require(len(re.findall(r"#\s*\[\s*path\s*=.*?\]\s*mod\s+" + re.escape(module) + r"\b", masked, re.S)) == 0,
                f"default module `{module}` gained an alternate #[path] source")
        require((ROOT / "src" / f"{module}.rs").is_file(), f"default module `{module}` source is missing")
    require(len(re.findall(r"\bmod\s+promotion\b", masked)) == 1,
            "promotion module has an alternate or duplicate source declaration")
    require(len(re.findall(r"\bmod\s+owned_pointer\b", masked)) == 1,
            "owned_pointer module has an alternate or duplicate source declaration")
    require(len(re.findall(r"#\s*\[\s*path\s*=\s*\"[^\"]*\"\s*\]\s*mod\s+promotion\b", masked)) == 0,
            "promotion cannot use an alternate #[path] route")

    include_routes = {
        "promotion": assert_selected_include_routes(
            shadow, (ROOT / "src/promotion.rs"), "promotion.rs"),
        "public_shared": assert_selected_include_routes(
            (ROOT / "src/public_shared.rs").read_text(), (ROOT / "src/public_shared.rs"), "public_shared.rs"),
    }

    shadow_tokens = tokens(shadow)
    for item in (
        'include!("../../../../src/bytes/shared_record.rs");',
        'include!(concat!(env!("OUT_DIR"),"/public_records.rs"));',
    ):
        require(token_count(shadow_tokens, tokens(item)) == 1,
                f"selected proof shadow must include exact `{item}`")
    return {
        "selected_shadow": "src/promotion.rs",
        "module_cfg": "#[cfg(creusot)] mod promotion;",
        "promotion_alternate_path": False,
        "all_selected_module_outer_attributes_exact": True,
        "literal_include_routes": include_routes,
        "support_paths": expected_paths,
        "production_shared_record_included": True,
        "generated_production_record_included": True,
    }


PROMOTION_FN_NAMES = [
    "atomic_field", "field_model", "metadata", "wellformed", "reclaimed", "valid", "valid_for",
    "word", "table", "valid", "matches", "valid", "valid", "is_raw", "is_shared", "valid",
    "root_valid", "observation", "cursor_model", "cursor_public", "root_id", "root_fraction",
    "root_metadata", "same_root", "same_pointer_owner", "child_valid", "is_child", "child_id",
    "child_fraction", "child_content", "child_public", "child_accepts", "even_table", "odd_table",
    "shared_table", "even_table_reification", "odd_table_reification", "shared_table_reification",
    "tag_pointer", "from_box_scoped", "shallow_clone_vec_checked", "valid", "content", "public",
    "accepts", "clone_result", "even_clone_checked", "even_clone_registration", "odd_clone_checked",
    "odd_clone_registration", "clone_root", "release_core", "free_recovered", "child_drop_checked",
    "child_drop_registration", "cleanup_child", "root_drop_checked", "even_root_drop_registration",
    "odd_root_drop_registration", "cleanup_root", "read_root", "first_promotion_client",
]


def assert_closed_function_surface(source: str) -> None:
    code = AI.mask_noncode(source)
    names = re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", code)
    require(sorted(names) == sorted(PROMOTION_FN_NAMES),
            "promotion.rs function surface changed; unreviewed/removed logic or executable helper")


def rust_attributes(source: str) -> list[list[str]]:
    """Collect bracketed Rust attrs from the comment/string-masked token stream."""
    ts = tokens(source)
    out: list[list[str]] = []
    i = 0
    while i + 1 < len(ts):
        if ts[i:i + 2] != ["#", "["]:
            i += 1
            continue
        depth = 1
        j = i + 2
        while j < len(ts) and depth:
            if ts[j] == "[": depth += 1
            elif ts[j] == "]": depth -= 1
            j += 1
        require(depth == 0, "unclosed Rust attribute in selected module")
        out.append(ts[i:j])
        i = j
    return out


def assert_no_unreviewed_proof_attributes(source: str, label: str,
                                          allowed_trusted: set[str] | list[str] | None) -> None:
    """Reject trust aliases and vacuous contracts even inside cfg_attr."""
    forbidden = {"trusted", "assume", "axiom", "extern_spec", "externspec", "checktrusted"}
    attrs_all = rust_attributes(source)
    trust_attrs = []
    for attr in attrs_all:
        present = forbidden.intersection(attr)
        if present:
            require(present == {"trusted"} and attr == tokens("#[trusted]"),
                    f"{label} has a qualified/cfg_attr/axiom/assume trust alias: {sorted(present)}")
            trust_attrs.append(attr)
        if ("requires" in attr or "ensures" in attr):
            require(not any(attr[i:i + 3] in (["requires", "(", "false"], ["ensures", "(", "false"])
                            for i in range(len(attr) - 2)),
                    f"{label} contains a false requires/ensures clause")
    masked = AI.mask_noncode(source)
    require(not re.search(r"\b(?:assume|axiom|extern_spec|externspec|checktrusted)\b", masked),
            f"{label} contains an assume/axiom/extern-spec source escape")
    if allowed_trusted is not None:
        names: list[str] = []
        matches = list(re.finditer(r"#\s*\[\s*trusted\s*\]", masked))
        for hit in matches:
            item = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", masked[hit.end():])
            require(item is not None, f"{label} has an unattached trusted attribute")
            names.append(item.group(1))
        expected_names = sorted(allowed_trusted)
        require(sorted(names) == expected_names,
                f"{label} trusted function allowlist changed: {sorted(names)}")


def assert_reviewed_token_surfaces() -> dict[str, Any]:
    """Lock complete contracts/bodies as a supplement to semantic checks.

    These digests are over the parser's comment/string-masked Rust token
    streams. The independent route, field, call-order, contract-anchor, and
    native MIR checks below remain required; a digest by itself is not a pass.
    """
    sources = {
        "promotion": (ROOT / "src/promotion.rs").read_text(),
        "lifecycle": (ROOT / "src/lifecycle.rs").read_text(),
        "owned_pointer": (ROOT / "src/owned_pointer.rs").read_text(),
        "erased_call": (ROOT / "src/erased_call.rs").read_text(),
    }
    # Pin the complete token surface of every selected proof and generic
    # support module. This closes changes to pure projection signatures,
    # resource getters, trusted adapter contracts, and helper attributes that
    # are intentionally outside the bounded effect-body snippets below. The
    # structured source/body checks remain the correspondence argument; these
    # module pins make the reviewed proof boundary closed against additions.
    lib = (ROOT / "src/lib.rs").read_text()
    selected_module_paths = {
        "promotion": ROOT / "src/promotion.rs",
        "lifecycle": ROOT / "src/lifecycle.rs",
        "owned_pointer": ROOT / "src/owned_pointer.rs",
        "erased_call": ROOT / "src/erased_call.rs",
        "event": ROOT / "src/event.rs",
        "provenance_specs": ROOT / "src/provenance_specs.rs",
        "field_event": ROOT / "src/field_event.rs",
        "physical_projection": ROOT / "src/physical_projection.rs",
        "free_effect": ROOT / "src/free_effect.rs",
    }
    for module in ("pointer_event", "raw_vec", "boxed_alignment", "promotion_tags"):
        selected_module_paths[module] = (ROOT / "src" / AI.rust_path_for_module(lib, module)).resolve()
    for name, expected in EXPECTED_MODULE_TOKEN_HASHES.items():
        source_path = selected_module_paths[name]
        source_text = source_path.read_text()
        assert_module_token_surface(name, source_text)

    event_source = selected_module_paths["event"].read_text()
    require(token_count(tokens(event_source), tokens("fn abort_overflow() { std::process::abort() }")) == 1,
            "event.rs must retain its sole exact fail-stop overflow escape")
    provenance_source = selected_module_paths["provenance_specs"].read_text()
    require(token_count(tokens(provenance_source), tokens("extern_spec! { impl<T> *mut T {")) == 1 and
            token_count(tokens(provenance_source), tokens("fn wrapping_add(self, offset: usize) -> *mut T;")) == 1,
            "provenance_specs.rs must retain only the reviewed address-only wrapping_add extern spec")
    expected_attrs = {
        ("promotion", "from_box_scoped"): "7b12f5c607d8ae0784f8decfb301b14894284b7852983c661fa5d3e0fa94761e",
        ("promotion", "shallow_clone_vec_checked"): "d27dcea2c128679a799a6efba14124c14e07c969ce0b7c68228f029ff3289946",
        ("promotion", "even_clone_checked"): "c91c5f583ed89884c8753de1a44e434a16aeb99f2ebc2f927614c1482e3c0f14",
        ("promotion", "odd_clone_checked"): "b7af163ba17e11c1095c603afe5b908c8c04a78f59724dc4eea7e3b5b76ccadf",
        ("promotion", "clone_root"): "8fdfb85b0d05135c6d519e0a4f623a3e6324fa0a9b1b3125de999edec77ddf3e",
        ("promotion", "release_core"): "462eb1bd465b2118b27446c32f1eec644d93ba02d0d1a3b97b348f40ab6071ce",
        ("promotion", "free_recovered"): "68a6c24e00c13dfd52da0d3af55264f5a58d3f2990254073c9a272856a5b1218",
        ("promotion", "child_drop_checked"): "b7144a2fce1a063e5611e43c22b400acfd4317d1ae9df973dae6d1a47d884ea8",
        ("promotion", "cleanup_child"): "039dfb66d82463ddc5c7b2a9b28ddcf9f275184cf5a768aed2170d125d29ee06",
        ("promotion", "root_drop_checked"): "fcb19c53329e76ae1db0a80c24dd23a51ce4eb3ae60cbde95c3c7bf7c35c06a1",
        ("promotion", "cleanup_root"): "a6575095c3a4f92abb34cfd51c2c9af1dda6e3c402fb335b2d3578297d124cc0",
        ("promotion", "read_root"): "773e8d388dc6dc8d3f9166fedac70b0589feb8408dd3dba7485147e5ccd0371b",
        ("promotion", "first_promotion_client"): "ae7529abf9d19abcb04c4281c374e57abc19ae365be4826334a542586c1f9dd5",
        ("lifecycle", "initialize_pair"): "28a45dd80812dc073a9f8d2ceacae2264fba2b988281587a114ed853c0418531",
        ("owned_pointer", "load_acquire"): "efded89e1409103b1f4d959539cda5a87f5ee86d68bc505c3abc702420f71fb1",
        ("owned_pointer", "compare_exchange"): "d82bb40a51d290535a0d4b4d3572916bc8755f9f384ea602b49f2d12bc6ad9d4",
        ("owned_pointer", "exchange_known"): "76a3ef26d704dff02f0372834a94c6e719fd3503c9bb5efe7f6fb51c414e4455",
        ("owned_pointer", "exchange_singleton"): "180fc318b7a72cc86cda2cee928b4b10d4065c7d7d2190c5553bb7aaf1ada5f2",
        ("owned_pointer", "get_mut_finish"): "d1f605f5e39f26b95b4858f69b5e1bfedebd95324ed77b0f09ec0c22581d560e",
        ("erased_call", "registered3"): "fc732698980c1d8ebd1f2dd48189b195b6ac7e72449fce68b4cb265cf88dd8f4",
        ("erased_call", "invoke3"): "55b721cbc9a35b346f690ede4ca047d20675b227a92a7778797e9b026d832cbe",
    }
    for (source_name, function), expected in expected_attrs.items():
        assert_attr_token_hash(sources[source_name], function, expected,
                               f"{source_name}::{function}")

    sources["raw_vec"] = (CRATE_ROOT / "src/ownership_proof/raw_vec.rs").read_text()
    for (source_name, function), expected in EXPECTED_EFFECT_BODY_HASHES.items():
        assert_body_token_hash(sources[source_name], function, expected,
                               f"{source_name}::{function}")
    return {"complete_attribute_surfaces_pinned": len(expected_attrs),
            "effect_bearing_body_token_streams_pinned": len(EXPECTED_EFFECT_BODY_HASHES),
            "complete_proof_and_support_module_token_surfaces_pinned": len(EXPECTED_MODULE_TOKEN_HASHES),
            "event_abort_and_existing_address_only_extern_spec_explicitly_bound": True,
            "semantic_operation_checks_also_required": True}


def assert_proof_state_types(source: str) -> dict[str, Any]:
    expected_fields = {
        "Payload": ["recovery", "physical_end", "control_end", "physical", "control", "base", "capacity", "len", "expected"],
        "SharedCore": ["shared", "bound", "capacity", "control", "physical", "invariant", "ticket"],
        "RootDescriptor": ["base", "capacity", "expected", "model"],
        "ChildProof": ["core", "binding"],
        "RawPhase": ["recovery", "physical", "own", "current"],
        "SharedPhase": ["root", "cursor", "own", "current"],
        "PromotionScope": ["descriptor", "phase"],
    }
    for name, wanted in expected_fields.items():
        got = struct_fields(source, name)
        require(got == wanted, f"`{name}` fields changed: {got}")
    expected_field_types = {
        "Payload": {
            "recovery": "raw_vec::Recovery", "physical_end": "EndBorrow<raw_vec::PhysicalRegion>",
            "control_end": "EndBorrow<field_event::OwnedControl<Shared>>",
            "physical": "Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>",
            "control": "Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>",
            "base": "raw_vec::BoundPtr", "capacity": "usize", "len": "usize", "expected": "Snapshot<Seq<u8>>",
        },
        "SharedCore": {
            "shared": "*mut Shared", "bound": "raw_vec::BoundPtr", "capacity": "usize",
            "control": "Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>",
            "physical": "Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>",
            "invariant": "Ghost<GhostShared<field_event::ScopedFieldInvariant<lifecycle::State<Payload>>>>",
            "ticket": "Ghost<lifecycle::Ticket<Payload>>",
        },
        "RootDescriptor": {
            "base": "raw_vec::BoundPtr", "capacity": "usize", "expected": "Snapshot<Seq<u8>>",
            "model": "Snapshot<ModelAtomicPtr<()>>",
        },
        "ChildProof": {"core": "SharedCore", "binding": "Ghost<pointer_event::ReadOnlyPointer>"},
        "RawPhase": {
            "recovery": "raw_vec::Recovery", "physical": "raw_vec::PhysicalRegion",
            "own": "Perm<ModelAtomicPtr<()>>", "current": "SyncView",
        },
        "SharedPhase": {
            "root": "SharedCore", "cursor": "Cursor", "own": "Perm<ModelAtomicPtr<()>>",
            "current": "SyncView",
        },
        "PromotionScope": {"descriptor": "RootDescriptor", "phase": "Option<Phase>"},
    }
    for name, expected in expected_field_types.items():
        got = struct_field_types(source, name)
        normalized_expected = {field: tokens(ty) for field, ty in expected.items()}
        require(got == normalized_expected, f"`{name}` field type/binding surface changed")
    descriptor = type_item(source, "struct RootDescriptor {", "RootDescriptor")
    for forbidden in ("Perm", "Cursor", "Recovery", "Ticket", "LifetimeToken", "PhysicalRegion"):
        require(forbidden not in descriptor, f"RootDescriptor gained affine resource `{forbidden}`")
    require(token_count(tokens(source), tokens("type Cursor=crate::event::ScopeCursor<lifecycle::State<Payload>>;")) == 1,
            "there must be one exact erased lifecycle cursor alias")

    for alias in (
        "type CloneInput<'a>=(&'a RootDescriptor,&'a mut PromotionScope);",
        "type CloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<CloneInput<'a>>)->Bytes;",
        "type ChildDropInput<'a>=(ChildProof,&'a mut Cursor,&'a mut Option<Completion>);",
        "type ChildDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<ChildDropInput<'a>>);",
        "type RootDropInput<'a>=(RootDescriptor,&'a mut PromotionScope,&'a mut Option<Completion>);",
        "type RootDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<RootDropInput<'a>>);",
    ):
        require(token_count(tokens(source), tokens(alias)) == 1, f"proof callback ABI alias changed: `{alias}`")
    for enum_name, variants in {
        "Completion": ["KeptAlive", "Reclaimed", "physical_projection::FreeReceipt", "free_effect::TypedFreeReceipt<Shared>"],
        "OriginalSharedProof": ["Root", "RootDescriptor", "Child", "ChildProof"],
        "Phase": ["Raw", "RawPhase", "Shared", "SharedPhase"],
    }.items():
        item = type_item(source, f"enum {enum_name} {{", enum_name)
        for variant in variants:
            require(token_count(item, tokens(variant)) > 0, f"`{enum_name}` lost `{variant}`")
    require(token_count(type_item(source, "struct RawPhase {", "RawPhase"), tokens("Perm<ModelAtomicPtr<()>>")) > 0,
            "raw phase no longer owns the pointer history permission")
    require(token_count(type_item(source, "struct SharedPhase {", "SharedPhase"), tokens("Perm<ModelAtomicPtr<()>>")) > 0,
            "shared phase no longer retains the pointer history permission")
    require(struct_fields(source, "SharedPhase").count("cursor") == 1,
            "shared phase lost its unique lifecycle cursor field")
    require(token_count(type_item(source, "struct ChildProof {", "ChildProof"), tokens("pointer_event::ReadOnlyPointer")) > 0,
            "child proof no longer carries the affine read-only pointer binding")
    require(token_count(type_item(source, "struct ChildProof {", "ChildProof"), tokens("Ghost<Perm<ModelAtomicPtr<()>>>")) == 0,
            "child proof illegally owns/rebinds the root pointer permission")
    # Pin the return types of pure state projections as well as their backing
    # field profile. In particular a resource-returning getter cannot be
    # introduced under an existing method name.
    for name, signature in {
        "observation": "fn observation(self)->Snapshot<(crate::fraction_map::LiveFractions,Int)>",
        "cursor_model": "fn cursor_model(self)->ModelAtomic",
        "cursor_public": "fn cursor_public(self)-><lifecycle::State<Payload> as Protocol>::Public",
        "root_id": "fn root_id(self)->Int",
        "root_fraction": "fn root_fraction(self)->PositiveReal",
        "root_metadata": "fn root_metadata(self)-><Payload as lifecycle::RecoveryPayload>::Metadata",
    }.items():
        require(AI.function_signature(source, name) == tokens(signature),
                f"PromotionScope projection `{name}` signature changed")
    return {"affine_scope_external_to_Bytes": True,
            "root_descriptor_copyable_metadata_only": True,
            "child_pointer_binding_read_only": True,
            "proof_state_field_types_and_projection_signatures_exact": True,
            "single_scope_cursor_type": "event::ScopeCursor<State<Payload>>",
            "completion_contains_both_free_receipts": True}


def assert_support_adapters(lib: str, shadow: str, native_bundle: dict[str, Any]) -> dict[str, Any]:
    """Check the exact generic adapters used by the selected proof path."""
    owned = (ROOT / "src/owned_pointer.rs").read_text()
    pointer_path = (ROOT / "src" / AI.rust_path_for_module(lib, "pointer_event")).resolve()
    pointer = pointer_path.read_text()
    field = (ROOT / "src/field_event.rs").read_text()
    projection = (ROOT / "src/physical_projection.rs").read_text()
    free = (ROOT / "src/free_effect.rs").read_text()
    erased = (ROOT / "src/erased_call.rs").read_text()
    rawvec_path = (ROOT / "src" / AI.rust_path_for_module(lib, "raw_vec")).resolve()
    rawvec = rawvec_path.read_text()
    align_path = (ROOT / "src" / AI.rust_path_for_module(lib, "boxed_alignment")).resolve()
    alignment = align_path.read_text()
    tags_path = (ROOT / "src" / AI.rust_path_for_module(lib, "promotion_tags")).resolve()
    tags = tags_path.read_text()

    assert_no_unreviewed_proof_attributes(owned, "owned_pointer.rs",
                                          {"load_acquire", "compare_exchange", "get_mut_finish"})
    assert_no_unreviewed_proof_attributes((ROOT / "src/lifecycle.rs").read_text(),
                                          "lifecycle.rs", set())
    assert_no_unreviewed_proof_attributes(erased, "erased_call.rs", {"registered3", "invoke3"})
    support_trust = (
        (pointer_path, "pointer_event.rs", ["bind_read_only", "get_mut", "load_relaxed", "new_pointer"]),
        (ROOT / "src/field_event.rs", "field_event.rs",
         ["new", "model", "model", "bind", "increment_owned", "decrement_owned", "acquire_owned"]),
        (ROOT / "src/physical_projection.rs", "physical_projection.rs", ["borrow", "borrow_empty", "deallocate"]),
        (ROOT / "src/free_effect.rs", "free_effect.rs", ["deallocate_typed_box"]),
        (rawvec_path, "raw_vec.rs", ["reallocate_bound", "offset_from_bound_base", "detach_vec",
                                       "detach_boxed_slice", "resume_vec", "deallocate_vec",
                                       "deallocate_bound_vec", "borrow_mut", "borrow_bound_mut",
                                       "borrow_bound_uninit_mut", "borrow_bound", "borrow_empty_bound",
                                       "borrow_empty_bound_mut", "borrow_empty_bound_uninit_mut"]),
        (align_path, "boxed_alignment.rs", ["into_raw_aligned"]),
        (tags_path, "tag_specs.rs", ["clear_low_bit", "equal_pointer_distance"]),
    )
    for path, label, trusted_names in support_trust:
        assert_no_unreviewed_proof_attributes(path.read_text(), label, trusted_names)

    # The model adapters are checked as actual executable wrappers, not as a
    # claim that native std atomics or their Rust memory-model interpretation
    # have been derived by Creusot.
    exact_body(owned, "load_acquire", "field.load(Ordering::Acquire)", "owned_pointer::load_acquire")
    exact_body(owned, "compare_exchange",
               "field.compare_exchange(expected,new,Ordering::AcqRel,Ordering::Acquire)",
               "owned_pointer::compare_exchange")
    exact_body(owned, "exchange_known", r"""compare_exchange(field,expected,new,ghost! {
        |event:Result<&mut Committer<ModelAtomicPtr<()>,*mut (),Acquire,Release>,&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>,>| {
            match event {
                Ok(c)=>{c.shoot_load(&**own,&mut **current);c.shoot_store(&mut **own,&mut **current);}
                Err(c)=>{c.shoot_load(&**own,&mut **current);}
            }
        }
    })""", "owned_pointer::exchange_known")
    exact_body(owned, "exchange_singleton",
               "exchange_known(field,expected,new,own,current)", "owned_pointer::exchange_singleton")
    exact_body(owned, "get_mut_finish",
               "#[cfg(creusot)]{unreachable!(\"generic terminal owned-history interpretation\")}#[cfg(not(creusot))]{(*field.get_mut(),snapshot!(0))}",
               "owned_pointer::get_mut_finish")
    attrs_have(owned, "compare_exchange", "Committer<ModelAtomicPtr<()>,*mut (),Acquire,Release>")
    attrs_have(owned, "compare_exchange", "Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>")
    attrs_have(owned, "compare_exchange", "old==c.val_load()")
    attrs_have(owned, "get_mut_finish", "forall<t:Int>")
    attrs_have(owned, "get_mut_finish", "t<=*result.1")
    attrs_have(owned, "get_mut_finish", "^field==*field")
    require("finish_latest" not in body(shadow, "root_drop_checked"),
            "root terminal route must use the consuming get_mut_finish operation directly")

    exact_body(pointer, "new_pointer", "(CoreAtomicPtr::new(value),Ghost::conjure())", "pointer_event::new_pointer")
    exact_body(pointer, "bind_read_only", "Ghost::conjure()", "pointer_event::bind_read_only")
    exact_body(pointer, "get_mut", "*field.get_mut()", "pointer_event::get_mut")
    attrs_have(pointer, "new_pointer", "pointer_model(&result.0)")
    attrs_have(pointer, "bind_read_only", "permission.inner_logic().val().get(t).unwrap_logic().0==value")
    attrs_have(pointer, "bind_read_only", "result.value()==value")
    attrs_have(pointer, "get_mut", "result==binding.inner_logic().value()")
    attrs_have(pointer, "get_mut", "^field==*field")

    exact_body(field, "new", "(CoreAtomicUsize::new(value),Ghost::conjure())", "field_event::new")
    exact_body(field, "bind", "Ghost::conjure()", "ScopedFieldInvariant::bind")
    exact_body(field, "decrement_owned",
               "unsafe{&*pointer}.atomic_field().fetch_sub(1,Ordering::Release)",
               "field_event::decrement_owned")
    exact_body(field, "acquire_owned",
               "unsafe{&*pointer}.atomic_field().load(Ordering::Acquire)",
               "field_event::acquire_owned")
    attrs_have(field, "new", "atomic_model(&result.0)")
    attrs_have(field, "bind", "state.atomic()==atomic_model(atomic)")
    attrs_have(field, "bind", "result.inner_logic().1.observation()==state.observe()")
    attrs_have(field, "decrement_owned", "cursor.inner_logic().model()==invariant.inner_logic().model()")
    exact_body(shadow, "atomic_field", "&self.ref_cnt", "Shared::atomic_field")
    exact_body(shadow, "field_model", "field_event::atomic_model(&self.ref_cnt)", "Shared::field_model")

    exact_body(projection, "borrow", "unsafe{core::slice::from_raw_parts(pointer,len)}",
               "physical_projection::borrow")
    exact_body(projection, "deallocate",
               "unsafe{alloc::alloc::dealloc(pointer,alloc::alloc::Layout::from_size_align(capacity,1).unwrap())}Ghost::conjure()",
               "physical_projection::deallocate")
    attrs_have(projection, "borrow", "pointer==bound.inner_logic().raw_pointer() as *const u8")
    attrs_have(projection, "borrow", "bound.inner_logic()@.unwrap_logic().2+len@<=region.inner_logic().hi()")
    attrs_have(projection, "deallocate", "result.inner_logic().pointer()==pointer")
    attrs_have(projection, "deallocate", "result.inner_logic().size()==capacity@")
    attrs_have(projection, "deallocate", "result.inner_logic().align()==1")

    exact_body(free, "deallocate_typed_box",
               "if core::mem::size_of::<T>()!=0{unsafe{dealloc(pointer.cast(),Layout::new::<T>())}}Ghost::conjure()",
               "free_effect::deallocate_typed_box")
    attrs_have(free, "deallocate_typed_box", "*owner.inner_logic().ward()==pointer as *const T")
    attrs_have(free, "deallocate_typed_box", "result.inner_logic().consumed(owner.inner_logic())")
    attrs_have(free, "deallocate_typed_box", "result.inner_logic().allocated()")

    exact_body(erased, "registered3", "dead", "erased_call::registered3")
    exact_attrs(erased, "registered3", ["#[trusted]", "#[logic(opaque)]"])
    exact_body(erased, "invoke3",
               "#[cfg(not(creusot))]{unsafe{native(args.0,args.1,args.2)}}#[cfg(creusot)]{unreachable!(\"generic registered erased call\")}",
               "erased_call::invoke3")
    attrs_have(erased, "invoke3", "registered3(native,spec.inner_logic())")
    attrs_have(erased, "invoke3", "spec.inner_logic().precondition((args.0,args.1,args.2,input))")
    attrs_have(erased, "invoke3", "spec.inner_logic().postcondition((args.0,args.1,args.2,input),result)")

    detach = body(rawvec, "detach_boxed_slice")
    has(detach, "Box::into_raw(input)", "raw_vec::detach_boxed_slice", count=1)
    has(detach, "Resource::alloc(allocation_value)", "raw_vec::detach_boxed_slice")
    has(detach, "Recovery{resource:recovery_resource", "raw_vec::detach_boxed_slice")
    has(detach, "PhysicalRegion{ledger", "raw_vec::detach_boxed_slice")
    attrs_have(rawvec, "detach_boxed_slice", "result.0.capacity()==input@.len()")
    attrs_have(rawvec, "detach_boxed_slice", "result.2.inner_logic().1.slot(index)")

    exact_body(alignment, "into_raw_aligned", "Perm::from_box(value)", "boxed_alignment::into_raw_aligned")
    attrs_have(alignment, "into_raw_aligned", "*result.1.ward()==result.0")
    attrs_have(alignment, "into_raw_aligned", "*result.1.val()==*value")
    attrs_have(alignment, "into_raw_aligned", "result.0.is_aligned_logic()")
    exact_body(tags, "equal_pointer_distance", "unsafe{pointer.offset_from(base)}", "tag_specs::equal_pointer_distance")
    attrs_have(tags, "equal_pointer_distance", "pointer==base as *const u8")
    attrs_have(tags, "equal_pointer_distance", "result==0isize")

    # Native selected vtable records bind the actual source callback names
    # consumed by the native MIR audit. Table identity remains part of the
    # explicit trusted reification/registration TCB in promotion.rs.
    production = native_bundle["production_source"]
    expected_vtables = {
        "PROMOTABLE_EVEN_VTABLE": ("promotable_even_clone", "promotable_even_drop"),
        "PROMOTABLE_ODD_VTABLE": ("promotable_odd_clone", "promotable_odd_drop"),
        "SHARED_VTABLE": ("shared_clone", "shared_drop"),
    }
    for table, (clone_fn, drop_fn) in expected_vtables.items():
        item = NATIVE.extract_item_tokens(production, f"static {table}: Vtable", table)
        require(token_count(item, tokens(f"clone:{clone_fn}")) == 1 and
                token_count(item, tokens(f"drop:{drop_fn}")) == 1,
                f"native {table} no longer points to the selected clone/drop callbacks")

    return {
        "owned_pointer_acquire_strong_cas_and_terminal_body_checked": True,
        "pointer_event_new_immutable_binding_and_get_mut_checked": True,
        "field_event_real_atomic_projection_release_acquire_checked": True,
        "physical_read_buffer_free_and_typed_control_free_checked": True,
        "rawvec_box_detach_and_box_alignment_adapter_source_checked": True,
        "tag_distance_and_native_vtable_targets_checked": True,
        "erased_call_abi_body_checked": True,
        "native_pointer_model_and_erasure_interpretation": "generic TCB, contracts/source-bound; not proved from Rust std"
    }


def assert_trust_boundary(source: str) -> dict[str, Any]:
    allowed = {
        "even_table_reification", "odd_table_reification", "shared_table_reification",
        "tag_pointer", "even_clone_registration", "odd_clone_registration",
        "child_drop_registration", "even_root_drop_registration", "odd_root_drop_registration",
    }
    assert_no_unreviewed_proof_attributes(source, "promotion.rs", allowed)
    masked = AI.mask_noncode(source)
    matches = list(re.finditer(r"#\s*\[\s*trusted\s*\]", masked))
    names = []
    for hit in matches:
        tail = masked[hit.end():]
        fn = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b", tail)
        require(fn is not None, "unattached or non-function trusted attribute in promotion module")
        names.append(fn.group(1))
    require(set(names) == allowed and len(names) == len(allowed),
            f"promotion trusted allowlist changed: {sorted(names)}")

    for name in ("from_box_scoped", "shallow_clone_vec_checked", "even_clone_checked", "odd_clone_checked",
                 "clone_root", "release_core", "free_recovered", "child_drop_checked", "cleanup_child",
                 "root_drop_checked", "cleanup_root", "read_root", "first_promotion_client"):
        a = attrs(source, name)
        require(not any("trusted" in x or "assume" in x or "axiom" in x or "extern_spec" in x for x in a),
                f"proof-bearing operation `{name}` acquired a trust/vacuity attribute")
    require(len(attrs(source, "first_promotion_client")) == 2,
            "client requires/ensures surface changed; re-review for vacuity")
    first_attrs = attrs(source, "first_promotion_client")
    require(first_attrs == [tokens("#[requires(input@.len()>0)]"), tokens("#[ensures(result@==input@)]")],
            "client nonempty/content contract changed")

    registration_contracts = {
        "even_clone_registration": [
            "#[trusted]", "#[ensures(result.0==even_table())]",
            "#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]",
            "#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>> result.1.inner_logic().precondition((data,ptr,len,input))==even_clone_checked.precondition((data,ptr,len,input)))]",
            "#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes> result.1.inner_logic().postcondition((data,ptr,len,input),output)==even_clone_checked.postcondition((data,ptr,len,input),output))]",
        ],
        "odd_clone_registration": [
            "#[trusted]", "#[ensures(result.0==odd_table())]",
            "#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]",
            "#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>> result.1.inner_logic().precondition((data,ptr,len,input))==odd_clone_checked.precondition((data,ptr,len,input)))]",
            "#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes> result.1.inner_logic().postcondition((data,ptr,len,input),output)==odd_clone_checked.postcondition((data,ptr,len,input),output))]",
        ],
        "child_drop_registration": [
            "#[trusted]", "#[ensures(result.0==shared_table())]",
            "#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ChildDropInput>> result.1.inner_logic().precondition((data,ptr,len,input))==child_drop_checked.precondition((data,ptr,len,input)))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ChildDropInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==child_drop_checked.postcondition((data,ptr,len,input),()))]",
        ],
        "even_root_drop_registration": [
            "#[trusted]", "#[ensures(result.0==even_table())]",
            "#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>> result.1.inner_logic().precondition((data,ptr,len,input))==root_drop_checked.precondition((data,ptr,len,input)))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==root_drop_checked.postcondition((data,ptr,len,input),()))]",
        ],
        "odd_root_drop_registration": [
            "#[trusted]", "#[ensures(result.0==odd_table())]",
            "#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>> result.1.inner_logic().precondition((data,ptr,len,input))==root_drop_checked.precondition((data,ptr,len,input)))]",
            "#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>> result.1.inner_logic().postcondition((data,ptr,len,input),())==root_drop_checked.postcondition((data,ptr,len,input),()))]",
        ],
    }
    for name, wanted in registration_contracts.items():
        exact_attrs(source, name, wanted)
        require(body(source, name) == tokens("unreachable!(\"checked closed table/clone erasure\")" if name in ("even_clone_registration", "odd_clone_registration") else
                                              "unreachable!(\"closed native shared-child drop erasure\")" if name == "child_drop_registration" else
                                              "unreachable!(\"closed native promotable ARC-drop erasure\")"),
                f"`{name}` body changed; checked callback shim must remain source-only and closed")

    exact_attrs(source, "even_table_reification", ["#[trusted]", "#[ensures(result==even_table())]"])
    exact_attrs(source, "odd_table_reification", ["#[trusted]", "#[ensures(result==odd_table())]"])
    exact_attrs(source, "shared_table_reification", ["#[trusted]", "#[ensures(result==shared_table())]"])
    exact_body(source, "even_table_reification", "unreachable!(\"closed native even-table reification\")")
    exact_body(source, "odd_table_reification", "unreachable!(\"closed native odd-table reification\")")
    exact_body(source, "shared_table_reification", "unreachable!(\"closed native Shared-table reification\")")
    exact_attrs(source, "tag_pointer", [
        "#[trusted]", "#[requires(ptr.addr_logic() & 1usize==0usize)]",
        "#[ensures(result==tag_specs::tagged_data(ptr))]",
        "#[ensures(result.addr_logic()==(ptr.addr_logic() | 1usize))]",
    ])
    tag_body = body(source, "tag_pointer")
    exact_body(source, "tag_pointer", r"""#[cfg(creusot)]{unreachable!("generic native exposed-provenance forward tag")}
        #[cfg(not(creusot))]{((ptr as usize)|1) as *mut ()}""", "tag_pointer")
    ordered(tag_body, ["#[cfg(creusot)]", "unreachable!(\"generic native exposed-provenance forward tag\")",
                       "#[cfg(not(creusot))]", "((ptr as usize)|1) as *mut ()"], "tag_pointer")
    return {"trusted_functions": sorted(allowed),
            "registration_contracts_exact": True,
            "native_callback_erasure": "only trusted registrations; all other proof operations body-checked"}


def assert_operation_mapping(source: str, native_result: dict[str, Any], native_bundle: dict[str, Any]) -> dict[str, Any]:
    # Box constructor: the proof scope starts from a real Box detach adapter,
    # records the same base/length, classifies the actual pointer parity, and
    # constructs exactly one pointer field plus external RawPhase authority.
    ctor = body(source, "from_box_scoped")
    exact_body(source, "from_box_scoped", r"""if input.len()==0{proof_assert!(false);unreachable!("nonempty selected client excludes static branch");}
        let expected=snapshot!(input@);
        let (raw,len,capabilities)=raw_vec::detach_boxed_slice(input);
        let (base,capacity)=raw.into_bound_ptr_at_zero();
        let ptr=base.as_ptr();
        let address=crate::provenance_specs::pointer_addr(ptr);
        ghost!{tag_specs::classify_low_bit(address);tag_specs::tagged_low_bit(address);};
        let (word,vtable)=if address&1usize==0usize{(tag_pointer(ptr),even_table_reification())}
            else{(ptr.cast::<()>(),odd_table_reification())};
        let mut current=ghost!{SyncView::new().into_inner()};
        let (data,own)=pointer_event::new_pointer(word,current.borrow_mut());
        let descriptor=ghost!{RootDescriptor{base,capacity,expected,model:snapshot!(pointer_event::pointer_model(&data))}};
        let scope=ghost!{let (recovery,physical)=capabilities.into_inner();
            PromotionScope{descriptor:*descriptor,phase:Some(Phase::Raw(RawPhase{
                recovery,physical,own:own.into_inner(),current:current.into_inner(),
            }))}};
        (Bytes{ptr,len,data,vtable,original_shared:ghost!{OriginalSharedProof::Root(*descriptor)}},scope)""",
               "from_box_scoped")
    ordered(ctor, ["raw_vec::detach_boxed_slice(input)", "raw.into_bound_ptr_at_zero()",
                   "provenance_specs::pointer_addr(ptr)", "if address & 1usize==0usize",
                   "pointer_event::new_pointer(word,current.borrow_mut())", "RootDescriptor",
                   "PromotionScope", "RawPhase", "OriginalSharedProof::Root"], "from_box_scoped")
    for snippet in ("if input.len()==0", "tag_pointer(ptr)", "even_table_reification()",
                    "odd_table_reification()", "snapshot!(input@)"):
        has(ctor, snippet, "from_box_scoped")
    require(token_count(ctor, tokens("raw_vec::detach_boxed_slice(input)")) == 1,
            "Box ownership must be detached once")
    raw_vec = (CRATE_ROOT / "src/ownership_proof/raw_vec.rs").read_text()
    detach = body(raw_vec, "detach_boxed_slice")
    has(detach, "Box::into_raw(input)", "RawVec Box adapter")
    has(detach, "Recovery", "RawVec Box adapter")
    has(detach, "PhysicalRegion", "RawVec Box adapter")

    # The exact native MIR/source audit has already pinned From<Box> and its
    # parity vtables. This shadow checks the selected branch is nonempty-only
    # and reaches the checked proof-facing constructor.
    native_audit = native_result["native_mir"]
    require(native_audit.get("selected_definition_headers_unique") is True and
            native_audit.get("selected_production_mir_count") == 18,
            "native checker did not admit the unique selected constructor/callback MIR set")

    # Clone route: saved vtable callback, parity-specific exact registration,
    # three unchanged runtime arguments, one Ghost bundle, and checked shim.
    clone_root = body(source, "clone_root")
    exact_body(source, "clone_root", r"""let descriptor=ghost!{match &*source.original_shared{
            OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
        let native=source.vtable.clone;
        let address=crate::provenance_specs::pointer_addr(source.ptr);
        if address&1usize==0usize{
            let (table,spec)=even_clone_registration();
            proof_assert!(source.vtable==table);
            erased_call::invoke3(native,(&source.data,source.ptr,source.len),
                ghost!{(*descriptor,&mut **scope)},spec)
        }else{
            let (table,spec)=odd_clone_registration();
            proof_assert!(source.vtable==table);
            erased_call::invoke3(native,(&source.data,source.ptr,source.len),
                ghost!{(*descriptor,&mut **scope)},spec)
        }""", "clone_root")
    ordered(clone_root, ["source.vtable.clone", "if address & 1usize==0usize",
                         "even_clone_registration()", "erased_call::invoke3",
                         "odd_clone_registration()", "erased_call::invoke3"], "clone_root")
    has(clone_root, "(&source.data,source.ptr,source.len)", "clone_root runtime ABI")
    has(clone_root, "ghost!{(*descriptor,&mut **scope)}", "clone_root affine proof argument")

    # Both parity helpers perform the actual Acquire adapter, expose only the
    # existing tagged pointer kind, and route to the single strong-CAS helper.
    for name in ("even_clone_checked", "odd_clone_checked"):
        helper = body(source, name)
        ordered(helper, ["owned_pointer::load_acquire", "Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>",
                         "provenance_specs::pointer_addr(stored)&1usize",
                         "unreachable!(\"first clone raw phase excludes existing ARC branch\")",
                         "shallow_clone_vec_checked"], name)
        has(helper, "c.shoot_load(&raw.own,&mut raw.current)", name)
        require(token_count(helper, tokens("shallow_clone_vec_checked(")) == 1,
                f"{name} must perform one checked vector promotion")
    shallow = body(source, "shallow_clone_vec_checked")
    ordered(shallow, ["field_event::new(2,control_view.borrow_mut())", "Shared{buf,cap,ref_cnt}",
                      "boxed_alignment::into_raw_aligned(boxed)",
                      "owned_pointer::exchange_singleton(data,expected,shared.cast(),own_borrow,current_borrow)",
                      "lifecycle::State::initialize_pair(count_permission,count_view,payload,lifetime)",
                      "field_event::ScopedFieldInvariant::bind(&shared_ref.ref_cnt,state)",
                      "pointer_event::new_pointer(shared.cast()",
                      "pointer_event::bind_read_only(&child_data,shared.cast(),child_permission)",
                      "Phase::Shared(SharedPhase",
                      "OriginalSharedProof::Child"], "shallow_clone_vec_checked")
    has(shallow, "Err(_actual)", "singleton strong-CAS failure exclusion")
    has(shallow, "unreachable!", "singleton strong-CAS failure exclusion")
    require(token_count(shallow, tokens("owned_pointer::exchange_singleton(")) == 1,
            "promotion must use one checked strong CAS")
    require(token_count(shallow, tokens("field_event::new(2")) == 1,
            "Shared initial count must be two exactly once")
    require(token_count(shallow, tokens("State::initialize_pair(")) == 1,
            "promotion must issue the real initial root/child ticket pair once")
    require(token_count(shallow, tokens("ScopedFieldInvariant::bind(")) == 1,
            "the single control field must receive one scoped cursor binding")
    for snippet in (
        "(^input.inner_logic().1).is_shared()",
        "result.child_valid()",
        "result.child_accepts(^input.inner_logic().1)",
        "result.ptr==offset&&result.len==len",
        "FMap::singleton(",
        ".insert(result.child_id()",
        "(^input.inner_logic().1).root_id()!=result.child_id()",
    ):
        attrs_have(source, "shallow_clone_vec_checked", snippet, "shallow_clone_vec_checked contract")

    # Count-two initialization produces the root's external affine authority
    # and child's read-only pointer view. Audit actual source body/contracts.
    lifecycle = (ROOT / "src/lifecycle.rs").read_text()
    pair = body(lifecycle, "initialize_pair")
    pair_attrs = attrs(lifecycle, "initialize_pair")
    require(any(tokens("#[requires(payload.wellformed() && full.frac() == PositiveReal::from_int(1))]") == x
                for x in pair_attrs), "initialize_pair lost full-lifetime premise")
    require(any(tokens("#[requires(own.val() == FMap::singleton((*own.ward()).get_timestamp(*current), (2usize, *current)))]") == x
                for x in pair_attrs), "initialize_pair lost real count-two singleton premise")
    for snippet in (
        "result.inner_logic().0.protocol()",
        "result.inner_logic().1.valid(result.inner_logic().0.public())",
        "result.inner_logic().2.valid(result.inner_logic().0.public())",
        "result.inner_logic().1.id()==0",
        "result.inner_logic().2.id()==1",
        "result.inner_logic().1.id()!=result.inner_logic().2.id()",
        "result.inner_logic().0.live_map().len()==2",
        "result.inner_logic().0.next_id()==2",
        "result.inner_logic().0.pool_fraction()==Some(",
    ):
        attrs_have(lifecycle, "initialize_pair", snippet, "initialize_pair contract")
    ordered(pair, ["full.split()", "pool.split()", "issue(&mut alive,zero,root_fraction)",
                   "issue(&mut alive,one,child_fraction)", "recovery:Some(payload)",
                   "recovery:None", "next:Int::new(2)", "(state,root,child)"], "initialize_pair")
    for snippet in ("root_token.frac()", "child_token.frac()", "alive", "Authority::<LiveFractions>::alloc()",
                    "root_token", "child_token", "pool"):
        has(pair, snippet, "initialize_pair")
    for forbidden in ("compare_exchange", "fetch_add", "fetch_sub", "on_register", "relaxed_rmw"):
        require(forbidden not in pair, f"initialize_pair unexpectedly performs a native event `{forbidden}`")
    clone_root_attrs = attrs(source, "clone_root")
    require(any(token_count(a, tokens("clone_result(")) > 0 for a in clone_root_attrs) and
            any(token_count(a, tokens("root_valid(")) > 0 for a in clone_root_attrs),
            "clone_root no longer preserves root identity and exports the checked child result")

    # Child teardown is routed through the saved child's actual drop vtable
    # and immutable binding, while root teardown consumes its one latest owner.
    child_cleanup = body(source, "cleanup_child")
    exact_body(source, "cleanup_child", r"""let native=value.vtable.drop;
        let proof=ghost!{match value.original_shared.into_inner(){
            OriginalSharedProof::Child(p)=>p,_=>{proof_assert!(false);panic!()}}};
        let mut shared=ghost!{match scope.phase.as_mut().unwrap(){
            Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}}};
        let (table,spec)=child_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
            ghost!{(proof.into_inner(),&mut shared.cursor,&mut **output)},spec);""", "cleanup_child")
    ordered(child_cleanup, ["value.vtable.drop", "Child(p)", "child_drop_registration()",
                            "proof_assert!(value.vtable==table)", "erased_call::invoke3"], "cleanup_child")
    has(child_cleanup, "(&mut value.data,value.ptr,value.len)", "child cleanup ABI")
    has(child_cleanup, "&mut shared.cursor", "child cleanup cursor")
    for snippet in ("(^scope).same_root(*scope.inner_logic())",
                    "(^scope).same_pointer_owner(*scope.inner_logic())",
                    "(*((^scope).observation())).0==(*scope.inner_logic().observation()).0.remove(value.child_id())",
                    "(^output).unwrap_logic().reclaimed()==((*scope.inner_logic().observation()).0.len()==1)"):
        attrs_have(source, "cleanup_child", snippet, "cleanup_child contract")
    child_shim = body(source, "child_drop_checked")
    exact_body(source, "child_drop_checked", r"""let (proof,cursor,output)=input.split();
        let shared=pointer_event::get_mut(data,ghost!{&*proof.binding}).cast::<Shared>();
        release_core(shared,ghost!{proof.into_inner().core},cursor,output);""", "child_drop_checked")
    ordered(child_shim, ["pointer_event::get_mut(data", "release_core(shared", "cursor", "output"],
            "child_drop_checked")
    has(child_shim, "&*proof.binding", "child readonly binding")
    require("bind_read_only" not in child_shim and "new_pointer" not in child_shim,
            "child cleanup may not mint or rebind pointer authority")

    root_cleanup = body(source, "cleanup_root")
    exact_body(source, "cleanup_root", r"""let native=value.vtable.drop;
        let descriptor=ghost!{match value.original_shared.into_inner(){
            OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
        let address=crate::provenance_specs::pointer_addr(value.ptr);
        if address&1usize==0usize{
            let (table,spec)=even_root_drop_registration();
            proof_assert!(value.vtable==table);
            erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
                ghost!{(descriptor.into_inner(),&mut **scope,&mut **output)},spec);
        }else{
            let (table,spec)=odd_root_drop_registration();
            proof_assert!(value.vtable==table);
            erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
                ghost!{(descriptor.into_inner(),&mut **scope,&mut **output)},spec);
        }""", "cleanup_root")
    ordered(root_cleanup, ["value.vtable.drop", "OriginalSharedProof::Root", "if address & 1usize==0usize",
                           "even_root_drop_registration()", "erased_call::invoke3",
                           "odd_root_drop_registration()", "erased_call::invoke3"], "cleanup_root")
    has(root_cleanup, "(&mut value.data,value.ptr,value.len)", "root cleanup ABI")
    root_shim = body(source, "root_drop_checked")
    exact_body(source, "root_drop_checked", r"""let (_descriptor,mut scope,output)=input.split();
        let (root,mut cursor,own,_current)=ghost!{
            let shared=match scope.phase.take().unwrap(){
                Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}};
            (shared.root,shared.cursor,shared.own,shared.current)
        }.split();
        let (word,_stamp)=owned_pointer::get_mut_finish(data,own);
        let kind=crate::provenance_specs::pointer_addr(word)&1usize;
        if kind==0usize{
            release_core(word.cast(),root,cursor.borrow_mut(),output);
            proof_assert!((*cursor.observation()).0.len()==0);
        }else{
            proof_assert!(false);unreachable!("updated root history excludes raw free branch")
        }""", "root_drop_checked")
    ordered(root_shim, ["scope.phase.take()", "owned_pointer::get_mut_finish(data,own)",
                        "provenance_specs::pointer_addr(word)&1usize", "release_core(word.cast(),root,cursor.borrow_mut(),output)",
                        "(*cursor.observation()).0.len()==0"], "root_drop_checked")
    require("bind_read_only" not in root_shim and "new_pointer" not in root_shim,
            "terminal root cleanup may not create/rebind a pointer permission")
    require(token_count(root_shim, tokens("owned_pointer::get_mut_finish(")) == 1,
            "root cleanup must consume its unique latest pointer owner once")
    for snippet in ("(^input.inner_logic().1).phase==None",
                    "(^input.inner_logic().2).unwrap_logic().reclaimed()",
                    "(^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.root_metadata())"):
        attrs_have(source, "root_drop_checked", snippet, "root_drop_checked contract")

    # One release path updates the same cursor with decrement and, at zero,
    # Acquire/recovery plus both distinct allocation free receipts.
    release = body(source, "release_core")
    ordered(release, ["field_event::decrement_owned", "State::on_release", "if old==1",
                      "field_event::acquire_owned", "State::on_acquire", "Pending::recover",
                      "free_recovered", "Completion::Reclaimed", "Completion::KeptAlive"], "release_core")
    has(release, "&mut **cursor", "release cursor threading", count=2)
    require(token_count(release, tokens("field_event::decrement_owned")) == 1,
            "release path must have one decrement")
    require(token_count(release, tokens("field_event::acquire_owned")) == 1,
            "last-owner branch must have one Acquire")
    for snippet in ("(^cursor).model()==cursor.inner_logic().model()",
                    "(^cursor).public()==cursor.inner_logic().public()",
                    "unwrap_logic().reclaimed()==((*cursor.inner_logic().observation()).0.len()==1)",
                    "reclaimed()==((*cursor.inner_logic().observation()).0.len()==1)"):
        attrs_have(source, "release_core", snippet, "release_core contract")
    attrs_have(source, "release_core", "(*((^cursor).observation())).0.len()==0")
    free = body(source, "free_recovered")
    for call in ("physical_projection::deallocate(shared.buf,shared.cap,bound,capabilities)",
                 "free_effect::deallocate_typed_box(pointer,owner)"):
        has(free, call, "paired free receipts")
    has(free, "FreeReceipt", "buffer free receipt")
    has(free, "TypedFreeReceipt<Shared>", "typed control free receipt")
    require(token_count(free, tokens("physical_projection::deallocate(")) == 1 and
            token_count(free, tokens("free_effect::deallocate_typed_box(")) == 1,
            "last-owner path must discharge exactly one buffer and one typed control allocation")

    # Read correspondence is an actual physical borrow with the root's range;
    # the proof performs no extra AtomicPtr read at observation time.
    read = body(source, "read_root")
    exact_body(source, "read_root", r"""let root=ghost!{match scope.phase.as_ref().unwrap(){
            Phase::Shared(s)=>&s.root,_=>{proof_assert!(false);panic!()}}};
        let region=ghost!{
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*root.physical).to_ref();
            full.borrow(&root.ticket.token)
        };
        unsafe{physical_projection::borrow(value.ptr,value.len,ghost!{&root.bound},region)}""", "read_root")
    ordered(read, ["Phase::Shared(s)", "full.borrow(&root.ticket.token)",
                   "physical_projection::borrow(value.ptr,value.len,ghost!{&root.bound},region)"], "read_root")
    for forbidden in ("load_acquire", "load_relaxed", "pointer_event::get_mut", "data.load"):
        require(forbidden not in read, f"root read introduces an unreviewed pointer observation `{forbidden}`")
    attrs_have(source, "read_root", "result@==*scope.inner_logic().descriptor.expected")

    # Closed proof client mirrors native.rs source operation sequence, adds
    # only Ghost scope/completion channels, and observes while the root lives.
    driver = body(source, "first_promotion_client")
    exact_body(source, "first_promotion_client", r"""let (original,mut scope)=from_box_scoped(input);
        let child=clone_root(&original,scope.borrow_mut());
        let mut child_completion=ghost!{None::<Completion>};
        cleanup_child(child,scope.borrow_mut(),child_completion.borrow_mut());
        proof_assert!(child_completion.inner_logic()!=None && !child_completion.inner_logic().unwrap_logic().reclaimed());
        proof_assert!((*scope.observation()).0.len()==1);
        let observed=read_root(&original,scope.borrow()).to_vec();
        let mut root_completion=ghost!{None::<Completion>};
        cleanup_root(original,scope.borrow_mut(),root_completion.borrow_mut());
        proof_assert!(scope.phase==None && root_completion.inner_logic()!=None && root_completion.inner_logic().unwrap_logic().reclaimed());
        observed""", "first_promotion_client")
    ordered(driver, ["from_box_scoped(input)", "clone_root(&original,scope.borrow_mut())",
                     "cleanup_child(child,scope.borrow_mut(),child_completion.borrow_mut())",
                     "read_root(&original,scope.borrow()).to_vec()",
                     "cleanup_root(original,scope.borrow_mut(),root_completion.borrow_mut())",
                     "root_completion.inner_logic().unwrap_logic().reclaimed()", "observed"],
            "first_promotion_client")
    has(driver, "!child_completion.inner_logic().unwrap_logic().reclaimed()", "child KeptAlive result")
    has(driver, "(*scope.observation()).0.len()==1", "one live root ticket after child retirement")
    has(driver, "scope.phase==None", "scope consumed after root retirement")
    require(token_count(driver, tokens("read_root(")) == 1,
            "closed client must read exactly once between child and root cleanup")

    # Generic erased invocation body is source matched, but correspondence of
    # native ABI/function-pointer meaning remains a disclosed generic TCB.
    erased = (ROOT / "src/erased_call.rs").read_text()
    invoke = body(erased, "invoke3")
    require(token_count(invoke, tokens("native(args.0,args.1,args.2)")) == 1,
            "invoke3 must call the registered native callback once with exactly three runtime arguments")
    require(token_count(tokens(erased), tokens("Ghost<G>")) > 0 and
            token_count(tokens(erased), tokens("registered3(native,spec.inner_logic())")) > 0,
            "erased call registration lost its proof-only argument mapping")

    return {
        "native_constructor_to_shadow": "production From<Box<[u8]>> and captured MIR audited; shadow consumes Box through raw_vec::detach_boxed_slice and branches on same pointer parity",
        "native_clone_to_shadow": "production Clone and callback MIR audited; shadow dispatches saved clone pointer through exact three-runtime-argument invoke3 registrations to Acquire/strong-CAS checked shim",
        "native_read_to_shadow": "production AsRef/as_slice audited; shadow root read uses physical_projection::borrow over original ptr/len and no additional atomic load",
        "native_child_drop_to_shadow": "production cleanup/Shared callback path audited; shadow callback uses child ReadOnlyPointer and same cursor",
        "native_root_drop_to_shadow": "production cleanup/promotable drop path audited; shadow consumes latest owned pointer permission then releases root ticket and proves cursor empty",
        "count_two_pair": "real lifecycle::State::initialize_pair body and count-two premise checked; root+child tickets, fresh IDs, map, fractions and pool are in the proof contracts/body",
        "allocation_free_result": "buffer FreeReceipt and typed Shared FreeReceipt both returned by the same last-owner path",
        "native_audit_status": native_result.get("status"),
        "remaining_tcb": [
            "generic RawVec Box detach physical-authority interpretation and generic PhysicalRegion slice/free adapter contracts",
            "generic pointer AtomicPtr field/model identity, strong-CAS history, and terminal get_mut interpretation",
            "trusted vtable table reification, proof/native callback registration and three-argument Ghost erasure correspondence",
            "Rust compiler MIR adequacy, allocator behavior, and pointer provenance/tag exposure",
            "native execution tests corroborate returned bytes but are not a mathematical proof",
        ],
    }


def audit(shadow_path: pathlib.Path) -> dict[str, Any]:
    shadow_resolved = shadow_path.resolve()
    expected_shadow = (ROOT / "src/promotion.rs").resolve()
    require(shadow_resolved == expected_shadow,
            f"selected shadow must be the live checked-in src/promotion.rs, got {shadow_resolved}")
    lib_path = ROOT / "src/lib.rs"
    shadow = shadow_resolved.read_text()
    lib = lib_path.read_text()
    # The generated extraction is checked against actual marker spans; the
    # selected proof include route is separately pinned above.
    bundle = NATIVE.load_bundle()
    native_result = NATIVE.audit_bundle(bundle)
    routes = assert_routes(lib, shadow)
    assert_closed_function_surface(shadow)
    token_surfaces = assert_reviewed_token_surfaces()
    state = assert_proof_state_types(shadow)
    trust = assert_trust_boundary(shadow)
    adapters = assert_support_adapters(lib, shadow, bundle)
    mapping = assert_operation_mapping(shadow, native_result, bundle)
    extracted = AI.audit_generated_extractions(
        bundle["production_source"], ROOT / "generated",
        (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes/vtable_record.rs").read_text(),
        (CRATE_ROOT / "src/bytes_mut.rs").read_text(),
    )
    build_pipeline = assert_public_records_build_pipeline()
    return {
        "status": "pass",
        "checker_scope": "selected AL promotion shadow to bounded native source/MIR correspondence, with independent lifecycle/affine wiring checks",
        "native_audit": native_result,
        "proof_routes": routes,
        "reviewed_token_surfaces": token_surfaces,
        "proof_state": state,
        "trusted_boundary": trust,
        "support_adapters": adapters,
        "operation_mapping": mapping,
        "production_extraction": extracted,
        "compiled_input_routes": build_pipeline,
        "limitations": [
            "this correspondence gate does not prove Creusot/Why3 obligations; the admitted proof run must do so",
            "native MIR evidence is source/compiler output correspondence, not a proof of compiler adequacy",
            "generic standard-library, AtomicPtr, raw allocation, provenance, field projection and erased callback boundaries remain the stated TCB",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--shadow", type=pathlib.Path, default=ROOT / "src/promotion.rs")
    parser.add_argument("--mapping", type=pathlib.Path,
                        help="deprecated; any supplied mapping is rejected because correspondence is derived from source")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        require(args.mapping is None, "mapping files are not accepted; checks derive from live source and native MIR")
        result = audit(args.shadow)
    except (CheckError, NATIVE.AuditError, AI.CorrespondenceError, OSError, ValueError, KeyError, TypeError) as exc:
        result = {"status": "reject", "checker_scope": "AL native/source proof correspondence", "reason": str(exc)}
        rendered = json.dumps(result, indent=2) + "\n"
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(rendered)
        print(rendered, end="")
        return 1
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
