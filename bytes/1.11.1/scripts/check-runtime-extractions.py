#!/usr/bin/env python3
"""Check reviewed inverse substitutions against the pinned runtime source.

This establishes source correspondence for the listed helper extractions,
proof-only contracts and cfg-gated convenience calls, and slice read overrides.
It is not a general contextual-equivalence theorem and does not prove generic
Buf defaults, adapter traits, Drop, or standard-library behavior.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "92fe500df540a2e99f150a9890a05dafb12abde2"
RESULTS = []


def compact(source: str) -> str:
    return re.sub(r"\s+", "", source)


def replace_once(source: str, pattern: str, replacement: str, label: str) -> str:
    restored, count = re.subn(pattern, replacement, source)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one inverse substitution, found {count}")
    return restored


def replace_text_once(source: str, expected: str, replacement: str, label: str) -> str:
    count = source.count(expected)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one reviewed source fragment, found {count}")
    return source.replace(expected, replacement, 1)


def replace_text_count(
    source: str, expected: str, replacement: str, expected_count: int, label: str
) -> str:
    count = source.count(expected)
    if count != expected_count:
        raise SystemExit(
            f"{label}: expected exactly {expected_count} reviewed source fragments, found {count}"
        )
    return source.replace(expected, replacement)


def remove_reviewed_cfg_contracts(source: str, path: str) -> str:
    """Remove only the proof-only imports and contracts reviewed for each adapter."""
    contracts = {
        "take": (
            "#[cfg_attr(creusot, ensures(result.inner == inner))]\n"
            "#[cfg_attr(creusot, ensures(result.limit == limit))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(*result == self.inner))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(result == self.limit))]\n",
        ),
        "chain": (
            "    #[cfg_attr(creusot, ensures(result.a == a))]\n"
            "    #[cfg_attr(creusot, ensures(result.b == b))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(*result == self.a))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(*result == self.b))]\n",
        ),
        "reader": (
            "#[cfg_attr(creusot, ensures(result.buf == buf))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(*result == self.buf))]\n",
        ),
        "writer": (
            "#[cfg_attr(creusot, ensures(result.buf == buf))]\n",
            "    #[cfg_attr(creusot, check(ghost))]\n"
            "    #[cfg_attr(creusot, ensures(*result == self.buf))]\n",
        ),
    }
    if path in contracts:
        source = replace_text_once(
            source,
            "#[cfg(creusot)]\nuse creusot_std::prelude::*;\n\n",
            "",
            f"{path} Creusot-only prelude import",
        )
        for index, contract in enumerate(contracts[path], start=1):
            source = replace_text_once(source, contract, "", f"{path} proof contract {index}")
    return source


def function_body_end(source: str, opening_brace: int) -> int:
    depth = 0
    in_string = False
    escaped = False
    in_line_comment = False
    in_block_comment = False
    index = opening_brace
    while index < len(source):
        char = source[index]
        following = source[index + 1] if index + 1 < len(source) else ""
        if in_line_comment:
            if char == "\n":
                in_line_comment = False
        elif in_block_comment:
            if char == "*" and following == "/":
                in_block_comment = False
                index += 1
        elif in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        elif char == "/" and following == "/":
            in_line_comment = True
            index += 1
        elif char == "/" and following == "*":
            in_block_comment = True
            index += 1
        elif char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index + 1
        index += 1
    raise SystemExit("slice Buf method has an unterminated body")


def method_span(impl: str, name: str) -> tuple[int, int, str]:
    matches = list(re.finditer(r"\bfn\s+" + re.escape(name) + r"\s*\(", impl))
    if len(matches) != 1:
        raise SystemExit(f"slice Buf override {name}: expected one method, found {len(matches)}")
    match = matches[0]
    opening = impl.find("{", match.end())
    if opening < 0:
        raise SystemExit(f"slice Buf override {name}: body not found")
    end = function_body_end(impl, opening)

    start = match.start()
    attribute = impl.rfind("#[inline]", 0, start)
    if attribute >= 0 and impl[attribute + len("#[inline]"):start].strip() == "":
        start = attribute
    line_start = impl.rfind("\n", 0, start) + 1
    if impl[line_start:start].strip() == "":
        start = line_start
    return start, end, impl[start:end]


def expected_fixed_pair(name: str, rust_type: str, width: int, module: str, helper: str) -> tuple[str, str]:
    get = (
        f"#[inline] fn get_{name}(&mut self) -> {rust_type} {{ "
        f"self.try_get_{name}().unwrap_or_else(|error| panic_advance(&error)) }}"
    )
    try_get = (
        f"#[inline] fn try_get_{name}(&mut self) -> Result<{rust_type}, TryGetError> {{ "
        f"crate::{module}::{helper}(self).ok_or(TryGetError {{ requested: {width}, available: self.len(), }}) }}"
    )
    return get, try_get


def expected_variable_pair(name: str, little_endian: bool) -> tuple[str, str]:
    get = (
        f"#[inline] fn {name}(&mut self, nbytes: usize) -> u64 {{ "
        f"self.try_{name}(nbytes).unwrap_or_else(|error| panic_advance(&error)) }}"
    )
    helper = "read_le_u64" if little_endian else "read_be_u64"
    try_get = (
        f"#[inline] fn try_{name}(&mut self, nbytes: usize) -> Result<u64, TryGetError> {{ "
        "let _slice_at = match 8usize.checked_sub(nbytes) { Some(slice_at) => slice_at, "
        "None => panic_does_not_fit(8, nbytes), }; "
        f"crate::variable_read_ops::{helper}(self, nbytes).ok_or(TryGetError {{ "
        "requested: nbytes, available: self.len(), }) }"
    )
    return get, try_get


def remove_reviewed_read_overrides(impl: str) -> tuple[str, list[str]]:
    expectations: dict[str, str] = {}
    fixed_reads = (
        ("u8", "u8", 1, "slice_read_ops", "read_u8"),
        ("u16", "u16", 2, "slice_read_ops", "read_be_u16"),
        ("u32", "u32", 4, "slice_read_ops", "read_be_u32"),
        ("u64", "u64", 8, "slice_wide_read_ops", "read_be_u64"),
        ("u128", "u128", 16, "slice_wide_read_ops", "read_be_u128"),
        ("u16_le", "u16", 2, "endian_ops", "read_le_u16"),
        ("u32_le", "u32", 4, "endian_ops", "read_le_u32"),
        ("u64_le", "u64", 8, "slice_wide_read_ops", "read_le_u64"),
        ("u128_le", "u128", 16, "slice_wide_read_ops", "read_le_u128"),
        ("i16", "i16", 2, "endian_ops", "read_be_i16"),
        ("i16_le", "i16", 2, "endian_ops", "read_le_i16"),
        ("i32", "i32", 4, "endian_ops", "read_be_i32"),
        ("i32_le", "i32", 4, "endian_ops", "read_le_i32"),
        ("i64", "i64", 8, "signed_wide_ops", "read_be_i64"),
        ("i64_le", "i64", 8, "signed_wide_ops", "read_le_i64"),
        ("i128", "i128", 16, "signed_wide_ops", "read_be_i128"),
        ("i128_le", "i128", 16, "signed_wide_ops", "read_le_i128"),
    )
    for name, rust_type, width, module, helper in fixed_reads:
        get, try_get = expected_fixed_pair(name, rust_type, width, module, helper)
        expectations[f"get_{name}"] = get
        expectations[f"try_get_{name}"] = try_get
    for name, little_endian in (("get_uint", False), ("get_uint_le", True)):
        get, try_get = expected_variable_pair(name, little_endian)
        expectations[name] = get
        expectations[f"try_{name}"] = try_get

    spans = []
    for name, expected in expectations.items():
        start, end, actual = method_span(impl, name)
        if compact(actual) != compact(expected):
            raise SystemExit(f"slice Buf override {name}: body differs from reviewed helper wrapper")
        spans.append((start, end, name))

    restored = impl
    for start, end, name in sorted(spans, reverse=True):
        restored = restored[:start] + restored[end:]
    return restored, sorted(expectations)


def atomic_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(value, stream, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


for name in ("take", "limit", "reader", "writer", "chain"):
    path = Path("src/buf") / f"{name}.rs"
    original = subprocess.check_output(
        ["git", "show", f"{BASELINE}:bytes/1.11.1/{path}"], cwd=ROOT, text=True
    )
    actual = (ROOT / path).read_text()
    restored = remove_reviewed_cfg_contracts(actual, name)

    if name == "take":
        # The bound is exactly Take::remaining()'s existing bounded minimum.
        # Invert only this reviewed equivalent expression; its body and panic
        # text remain checked against the pinned source below.
        restored = replace_text_once(
            restored,
            "len <= crate::bounded_ops::bounded_len(self.inner.remaining(), self.limit)",
            "len <= self.remaining()",
            "Take copy_to_bytes remaining bound",
        )
    restored = restored.replace(
        "crate::bounded_ops::bounded_chunk(bytes, self.limit)",
        "&bytes[..cmp::min(bytes.len(), self.limit)]",
    )
    restored = restored.replace("crate::bounded_ops::bounded_len(", "cmp::min(")
    for count in ("cnt", "len"):
        restored = restored.replace(
            f"crate::bounded_ops::decrease_limit(&mut self.limit, {count});",
            f"self.limit -= {count};",
        )
    if name in ("take", "limit"):
        restored = "use core::cmp;\n" + restored
        original = "\n".join(line for line in original.splitlines() if line.strip() != "use core::cmp;")
        restored = "\n".join(line for line in restored.splitlines() if line.strip() != "use core::cmp;")
    else:
        restored = restored.replace("use std::io;", "use std::{cmp, io};", 1)

    if name == "chain":
        restored = replace_text_once(
            restored,
            "crate::chain_ops::saturating_sum(self.a.remaining(), self.b.remaining())",
            "self.a.remaining().saturating_add(self.b.remaining())",
            "Chain remaining sum",
        )
        restored = replace_text_once(
            restored,
            "crate::chain_ops::saturating_sum(self.a.remaining_mut(), self.b.remaining_mut())",
            "self.a\n            .remaining_mut()\n            .saturating_add(self.b.remaining_mut())",
            "Chain remaining_mut sum",
        )
        restored = replace_text_count(
            restored,
            "cnt = crate::chain_ops::split_count(a_rem, cnt).1;",
            "cnt -= a_rem;",
            2,
            "Chain Buf and BufMut advance splits",
        )
        restored = replace_once(
            restored,
            r"#\[cfg\(creusot\)\]\s*let taken = crate::buf::proof_convenience::take\(&mut self\.b, len - a_rem\);\s*#\[cfg\(not\(creusot\)\)\]\s*let taken = \(&mut self\.b\)\.take\(len - a_rem\);\s*ret\.put\(taken\);",
            "ret.put((&mut self.b).take(len - a_rem));",
            "Chain copy_to_bytes proof-only take adapter",
        )
    if compact(restored) != compact(original):
        raise SystemExit(f"{path}: inverse substitution does not match original source")
    RESULTS.append({
        "source": str(path),
        "source_sha256": hashlib.sha256(actual.encode()).hexdigest(),
        "inverse_substitution_matches": True,
        **({
            "reviewed_equivalent_transformations": [
                "Take::copy_to_bytes checks the same bounded length as Take::remaining()."
            ]
        } if name == "take" else {}),
    })

path = Path("src/buf/buf_impl.rs")
original = subprocess.check_output(
    ["git", "show", f"{BASELINE}:bytes/1.11.1/{path}"], cwd=ROOT, text=True
)
actual = (ROOT / path).read_text()
restored = actual
restored = replace_once(
    restored,
    r"crate::slice_ops::advance_slice\(self, cnt\);",
    "*self = &self[cnt..];",
    "slice Buf advance",
)
restored = replace_once(
    restored,
    r"crate::slice_ops::copy_to_slice\(self, dst\);",
    "dst.copy_from_slice(&self[..dst.len()]);\n        self.advance(dst.len());",
    "slice Buf copy_to_slice",
)
restored = re.sub(r"crate::cursor_ops::cursor_remaining\(", "saturating_sub_usize_u64(", restored)
restored = replace_once(
    restored,
    r"crate::cursor_ops::cursor_chunk\(slice, self\.position\(\)\)",
    "let pos = min_u64_usize(self.position(), slice.len());\n        &slice[pos..]",
    "Cursor chunk",
)
restored = replace_once(
    restored,
    r"crate::cursor_ops::cursor_position_after_advance\(\s*len,\s*pos,\s*cnt,?\s*\)",
    "pos + cnt as u64",
    "Cursor position advance",
)

# Restore the ordinary Buf API around proof-only convenience substitutions.
# Each inverse is tied to one exact import, method gate, or call-site shape.
restored = replace_text_once(
    restored,
    "#[cfg(all(feature = \"std\", not(creusot)))]\n"
    "use crate::buf::{reader, Reader};",
    "#[cfg(feature = \"std\")]\nuse crate::buf::{reader, Reader};",
    "Buf reader import cfg",
)
restored = replace_text_once(
    restored,
    "#[cfg(not(creusot))]\nuse crate::buf::{take, Chain, Take};",
    "use crate::buf::{take, Chain, Take};",
    "Buf take and chain imports cfg",
)
restored = replace_once(
    restored,
    r"#\[cfg\(creusot\)\]\s*let taken = crate::buf::proof_convenience::take\(self, len\);\s*#\[cfg\(not\(creusot\)\)\]\s*let taken = self\.take\(len\);\s*ret\.put\(taken\);",
    "ret.put(self.take(len));",
    "Buf copy_to_bytes proof-only take adapter",
)
restored = replace_text_once(
    restored,
    "    #[cfg(not(creusot))]\n    fn take(self, limit: usize) -> Take<Self>",
    "    fn take(self, limit: usize) -> Take<Self>",
    "Buf take method cfg",
)
restored = replace_text_once(
    restored,
    "    #[cfg(not(creusot))]\n    fn chain<U: Buf>(self, next: U) -> Chain<Self, U>",
    "    fn chain<U: Buf>(self, next: U) -> Chain<Self, U>",
    "Buf chain method cfg",
)
restored = replace_text_once(
    restored,
    "    #[cfg(all(feature = \"std\", not(creusot)))]\n"
    "    #[cfg_attr(docsrs, doc(cfg(feature = \"std\")))]\n"
    "    fn reader(self) -> Reader<Self>",
    "    #[cfg(feature = \"std\")]\n"
    "    #[cfg_attr(docsrs, doc(cfg(feature = \"std\")))]\n"
    "    fn reader(self) -> Reader<Self>",
    "Buf reader method cfg",
)

slice_start = restored.find("impl Buf for &[u8] {")
next_impl = restored.find('\n#[cfg(feature = "std")]\nimpl<T: AsRef<[u8]>> Buf for std::io::Cursor<T>', slice_start)
if slice_start < 0 or next_impl < 0:
    raise SystemExit("cannot delimit the actual slice Buf implementation")
read_restored, removed_methods = remove_reviewed_read_overrides(restored[slice_start:next_impl])
restored = restored[:slice_start] + read_restored + restored[next_impl:]
# The Cursor extraction moved these imports out of buf_impl.rs.
original = original.replace(
    '#[cfg(feature = "std")]\nuse crate::{min_u64_usize, saturating_sub_usize_u64};\n',
    "",
)
if compact(restored) != compact(original):
    raise SystemExit(f"{path}: inverse substitution does not match original source")
RESULTS.append({
    "source": str(path),
    "source_sha256": hashlib.sha256(actual.encode()).hexdigest(),
    "inverse_substitution_matches": True,
    "removed_read_overrides": removed_methods,
    "note": (
        "Read wrappers preserve fixed-size TryGetError fields and panic_advance; "
        "variable-width wrappers check nbytes <= 8 before calling the helper. "
        "Helper source contracts are checked separately; generic Buf behavior is not claimed."
    ),
})

atomic_json(ROOT / "verification/artifacts/adapter-correspondence.json", {
    "baseline_commit": BASELINE,
    "scope": (
        "reviewed source substitutions and normal-runtime inverses for proof-only cfg changes; "
        "generic adapter proof not claimed"
    ),
    "results": RESULTS,
})
print("Adapters and slice Buf: reviewed inverse substitutions match upstream runtime")
