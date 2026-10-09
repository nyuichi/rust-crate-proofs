#!/usr/bin/env python3
"""Fail-closed native source/MIR correspondence check for the AN witness.

This checks the bounded default-native Box/reclone client and the captured
production callback path. It is not a proof of rustc MIR adequacy, allocator or
pointer-provenance soundness, concurrent execution, or both physical pointer
parities. The companion native test is execution corroboration only.
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
AI_HELPER_FIXTURE_PATH = AI_PROBE / "fixtures" / "expected-shadow-bodies.json"
AI_HELPER_FIXTURE_SHA256 = "c4903a139f0924fc26a62c8319617db1f34cfbb876e7e775abba0f70130d26e1"

MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"
NATIVE_TEST_DIR = ROOT / "native-test"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\n"
    "commit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\n"
    "host: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\n"
    "LLVM version: 22.1.7\n"
)
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_MANIFEST = {
    "package": {"name": "bytes-promotable-reclone-native", "version": "0.0.0", "edition": "2021"},
    "workspace": {},
    "lib": {"name": "bytes_promotable_reclone_native", "path": "../native.rs"},
    "dependencies": {"bytes": {"path": "../../../../"}},
}

MIR_FILES = {
    "client": "bytes_promotable_reclone_native.promoted_reclone_scope.2-2-004.ElaborateDrops.after.mir",
    "from_box": "bytes.bytes-{impl#43}-from.2-2-004.ElaborateDrops.after.mir",
    "clone_impl": "bytes.bytes-{impl#4}-clone.2-2-004.ElaborateDrops.after.mir",
    "cleanup": "bytes.bytes-{impl#0}-cleanup.2-2-004.ElaborateDrops.after.mir",
    "bytes_drop": "bytes.bytes-{impl#3}-drop.2-2-004.ElaborateDrops.after.mir",
    "as_ref": "bytes.bytes-{impl#7}-as_ref.2-2-004.ElaborateDrops.after.mir",
    "as_slice": "bytes.bytes-{impl#0}-as_slice.2-2-004.ElaborateDrops.after.mir",
    "promotable_even_clone": "bytes.bytes-promotable_even_clone.2-2-004.ElaborateDrops.after.mir",
    "promotable_odd_clone": "bytes.bytes-promotable_odd_clone.2-2-004.ElaborateDrops.after.mir",
    "shallow_clone_vec": "bytes.bytes-shallow_clone_vec.2-2-004.ElaborateDrops.after.mir",
    "shallow_clone_arc": "bytes.bytes-shallow_clone_arc.2-2-004.ElaborateDrops.after.mir",
    "ref_count_increment": "bytes.ref_count_ops-increment.2-2-004.ElaborateDrops.after.mir",
    "promotable_even_drop": "bytes.bytes-promotable_even_drop.2-2-004.ElaborateDrops.after.mir",
    "promotable_odd_drop": "bytes.bytes-promotable_odd_drop.2-2-004.ElaborateDrops.after.mir",
    "shared_drop": "bytes.bytes-shared_drop.2-2-004.ElaborateDrops.after.mir",
    "release_shared": "bytes.bytes-release_shared.2-2-004.ElaborateDrops.after.mir",
    "free_shared": "bytes.bytes-free_shared.2-2-004.ElaborateDrops.after.mir",
    "ptr_map": "bytes.bytes-ptr_map.2-2-004.ElaborateDrops.after.mir",
    "atomic_with_mut": "bytes.loom-sync-atomic-{impl#0}-with_mut.2-2-004.ElaborateDrops.after.mir",
}

# MIR definition headers identify the symbol body selected by capture. The
# source coordinates embedded in impl-qualified names are permitted because
# they are emitted from the pinned crate source; the actual method path and
# all non-impl names are pinned here.
MIR_HEADER_PATTERNS = {
    "client": r"^fn promoted_reclone_scope\(.*\) -> Vec<u8> \{$",
    "from_box": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::from\(.*\) -> bytes::Bytes \{$",
    "clone_impl": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::clone\(.*\) -> bytes::Bytes \{$",
    "cleanup": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::cleanup\(.*\) -> \(\) \{$",
    "bytes_drop": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::drop\(.*\) -> \(\) \{$",
    "as_ref": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::as_ref\(.*\) -> &\[u8\] \{$",
    "as_slice": r"^fn bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::as_slice\(.*\) -> &\[u8\] \{$",
    "promotable_even_clone": r"^fn promotable_even_clone\(.*\) -> bytes::Bytes \{$",
    "promotable_odd_clone": r"^fn promotable_odd_clone\(.*\) -> bytes::Bytes \{$",
    "shallow_clone_vec": r"^fn shallow_clone_vec\(.*\) -> bytes::Bytes \{$",
    "shallow_clone_arc": r"^fn shallow_clone_arc\(.*\) -> bytes::Bytes \{$",
    "ref_count_increment": r"^fn increment\(.*\) -> \(\) \{$",
    "promotable_even_drop": r"^fn promotable_even_drop\(.*\) -> \(\) \{$",
    "promotable_odd_drop": r"^fn promotable_odd_drop\(.*\) -> \(\) \{$",
    "shared_drop": r"^fn shared_drop\(.*\) -> \(\) \{$",
    "release_shared": r"^fn bytes::release_shared\(.*\) -> \(\) \{$",
    "free_shared": r"^fn free_shared\(.*\) -> \(\) \{$",
    "ptr_map": r"^fn ptr_map\(.*\) -> \*mut u8 \{$",
    "atomic_with_mut": r"^fn loom::sync::atomic::<impl at src/loom\.rs:\d+:\d+: \d+:\d+>::with_mut\(.*\) -> R \{$",
}

EXPECTED_NATIVE_SOURCE = r"""use bytes::Bytes;

/// First promotion, promoted-root re-clone, lexical peer Drops, then saved return.
pub fn promoted_reclone_scope(input: Box<[u8]>) -> Vec<u8> {
    let original = Bytes::from(input);
    {
        let first = original.clone();
        let second = original.clone();
    }
    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();
    observed
}
"""

EXPECTED_NATIVE_TEST = r"""use bytes_promotable_reclone_native::promoted_reclone_scope;

#[test]
fn promoted_inner_drop_preserves_readable_original() {
    for len in [1usize, 2, 31, 256] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let input = expected.clone().into_boxed_slice();
        assert!(!input.is_empty(), "this witness requires a nonempty Box");
        assert_eq!(
            promoted_reclone_scope(input),
            expected,
            "contents changed at length {len}",
        );
    }
}"""

EXPECTED_BYTES_RECORD = r"""pub struct Bytes {
    ptr: *const u8,
    len: usize,
    // inlined "trait object"
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
    #[cfg(all(creusot, bytes_original_freeze_gate))]
    original_frozen: Option<OriginalFrozenProof>,
    #[cfg(all(creusot, bytes_original_constructor_gate))]
    original_bytes: creusot_std::prelude::Ghost<OriginalBytesProof>,
    #[cfg(all(creusot, bytes_original_shared_gate))]
    original_shared: creusot_std::prelude::Ghost<OriginalSharedProof>,
}
"""

EXPECTED_NATIVE_FIELD_PROFILE = r"""//! Compile-time check of the exact four default-native Bytes field types.
use core::sync::atomic::AtomicPtr;
const _: () = assert!(!core::mem::needs_drop::<*const u8>());
const _: () = assert!(!core::mem::needs_drop::<usize>());
const _: () = assert!(!core::mem::needs_drop::<AtomicPtr<()>>());
const _: () = assert!(!core::mem::needs_drop::<&'static ()>());
fn main() {
    println!("native Bytes fields have no independent drop glue");
}
"""

EXPECTED_FROM_BOX_BODY = r"""{
    #[cfg(all(creusot, bytes_original_constructor_gate))]
    { return original_bytes_from_box(slice); }
    #[cfg(not(all(creusot, bytes_original_constructor_gate)))]
    {
        if slice.is_empty() { return Bytes::new(); }
        let len = slice.len();
        let ptr = Box::into_raw(slice) as *mut u8;
        if ptr as usize & 0x1 == 0 {
            let data = ptr_map(ptr, |addr| addr | KIND_VEC);
            Bytes {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr,
                len,
                data: AtomicPtr::new(data.cast()),
                vtable: &PROMOTABLE_EVEN_VTABLE,
            }
        } else {
            Bytes {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr,
                len,
                data: AtomicPtr::new(ptr.cast()),
                vtable: &PROMOTABLE_ODD_VTABLE,
            }
        }
    }
}"""

EXPECTED_CLONE_BODY = r"""{
    #[cfg(all(creusot, bytes_original_shared_gate))]
    { original_shared_clone(self) }
    #[cfg(not(all(creusot, bytes_original_shared_gate)))]
    unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }
}"""

EXPECTED_CLEANUP_BODY = r"""{
    #[cfg(all(creusot, bytes_original_shared_gate))]
    { original_shared_cleanup(self); }
    #[cfg(not(all(creusot, bytes_original_shared_gate)))]
    {
        let mut this = ManuallyDrop::new(self);
        let ptr = this.ptr;
        let len = this.len;
        let callback = this.vtable.drop;
        unsafe { callback(&mut this.data, ptr, len) }
    }
}"""

EXPECTED_AS_REF_BODY = "{ self.as_slice() }"
EXPECTED_AS_SLICE_BODY = r"""{
    #[cfg(all(creusot, bytes_original_constructor_gate))]
    { original_bytes_as_slice(self) }
    #[cfg(all(creusot, bytes_original_shared_gate))]
    { original_shared_as_slice(self) }
    #[cfg(all(creusot, bytes_original_freeze_gate))]
    {
        let proof = self.original_frozen.as_ref().unwrap();
        unsafe { raw_vec::borrow_bound(&proof.base, self.len, ghost! { &proof.capabilities.1 }) }
    }
    #[cfg(not(any(all(creusot, bytes_original_freeze_gate), all(creusot, bytes_original_shared_gate), all(creusot, bytes_original_constructor_gate))))]
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
}"""
EXPECTED_BYTES_DROP_BODY = r"""{
    unsafe { (self.vtable.drop)(&mut self.data, self.ptr, self.len) }
}"""
EXPECTED_BYTES_DROP_IMPL = r"""impl Drop for Bytes {
    #[inline]
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(&mut self.data, self.ptr, self.len) }
    }
}"""

EXPECTED_EVEN_CLONE_BODY = r"""{
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;
    if kind == KIND_ARC {
        shallow_clone_arc(shared.cast(), ptr, len)
    } else {
        debug_assert_eq!(kind, KIND_VEC);
        let buf = ptr_map(shared.cast(), |addr| addr & !KIND_MASK);
        shallow_clone_vec(data, shared, buf, ptr, len)
    }
}"""
EXPECTED_ODD_CLONE_BODY = r"""{
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;
    if kind == KIND_ARC {
        shallow_clone_arc(shared as _, ptr, len)
    } else {
        debug_assert_eq!(kind, KIND_VEC);
        shallow_clone_vec(data, shared, shared.cast(), ptr, len)
    }
}"""
EXPECTED_SHALLOW_CLONE_VEC_BODY = r"""{
    let shared = Box::new(Shared {
        buf,
        cap: offset.offset_from(buf) as usize + len,
        ref_cnt: AtomicUsize::new(2),
    });
    let shared = Box::into_raw(shared);
    debug_assert!(0 == (shared as usize & KIND_MASK), "internal: Box<Shared> should have an aligned pointer",);
    match atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire) {
        Ok(actual) => {
            debug_assert!(core::ptr::eq(actual, ptr));
            Bytes {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr: offset,
                len,
                data: AtomicPtr::new(shared as _),
                vtable: &SHARED_VTABLE,
            }
        }
        Err(actual) => {
            let shared = Box::from_raw(shared);
            mem::forget(*shared);
            shallow_clone_arc(actual as _, offset, len)
        }
    }
}"""
EXPECTED_SHALLOW_CLONE_ARC_BODY = r"""{
    crate::ref_count_ops::increment(&(*shared).ref_cnt);
    Bytes {
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        original_frozen: None,
        ptr,
        len,
        data: AtomicPtr::new(shared as _),
        vtable: &SHARED_VTABLE,
    }
}"""
EXPECTED_EVEN_DROP_BODY = r"""{
    data.with_mut(|shared| {
        let shared = *shared;
        let kind = shared as usize & KIND_MASK;
        if kind == KIND_ARC {
            release_shared(shared.cast());
        } else {
            debug_assert_eq!(kind, KIND_VEC);
            let buf = ptr_map(shared.cast(), |addr| addr & !KIND_MASK);
            free_boxed_slice(buf, ptr, len);
        }
    });
}"""
EXPECTED_ODD_DROP_BODY = r"""{
    data.with_mut(|shared| {
        let shared = *shared;
        let kind = shared as usize & KIND_MASK;
        if kind == KIND_ARC {
            release_shared(shared.cast());
        } else {
            debug_assert_eq!(kind, KIND_VEC);
            free_boxed_slice(shared.cast(), ptr, len);
        }
    });
}"""
EXPECTED_SHARED_DROP_BODY = r"""{
    data.with_mut(|shared| {
        release_shared(shared.cast());
    });
}"""
EXPECTED_RELEASE_SHARED_BODY = r"""{
    if (*ptr).ref_cnt.fetch_sub(1, Ordering::Release) != 1 {
        return;
    }
    (*ptr).ref_cnt.load(Ordering::Acquire);
    free_shared(ptr);
}"""
EXPECTED_FREE_SHARED_BODY = r"""{
    let buf = (*ptr).buf;
    let cap = (*ptr).cap;
    dealloc(buf, Layout::from_size_align(cap, 1).unwrap());
    dealloc(ptr.cast(), Layout::new::<Shared>());
}"""
EXPECTED_SHARED_DROP_IMPL = r"""impl Drop for Shared {
    fn drop(&mut self) {
        unsafe { dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap()) }
    }
}"""
EXPECTED_SHARED_RECORD = r"""pub(crate) struct Shared {
    buf: *mut u8,
    cap: usize,
    pub(crate) ref_cnt: AtomicUsize,
}"""
EXPECTED_PROMOTABLE_EVEN_VTABLE = r"""static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {
    clone: promotable_even_clone,
    into_vec: promotable_even_to_vec,
    into_mut: promotable_even_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_even_drop,
}"""
EXPECTED_PROMOTABLE_ODD_VTABLE = r"""static PROMOTABLE_ODD_VTABLE: Vtable = Vtable {
    clone: promotable_odd_clone,
    into_vec: promotable_odd_to_vec,
    into_mut: promotable_odd_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_odd_drop,
}"""
EXPECTED_SHARED_VTABLE = r"""static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
}"""
EXPECTED_ATOMIC_MUT_IMPL = r"""impl<T> AtomicMut<T> for AtomicPtr<T> {
    fn with_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut *mut T) -> R,
    {
        f(self.get_mut())
    }
}"""
EXPECTED_TRY_INCREMENT_BODY = r"""{
    counter.fetch_update(
        Ordering::Relaxed,
        Ordering::Relaxed,
        crate::ref_count_limit::next_ref_count,
    )
}"""
EXPECTED_INCREMENT_BODY = r"""{
    if try_increment(counter).is_err() {
        crate::abort();
    }
}"""


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_ai_checker():
    require(AI_CHECKER_PATH.is_file(), f"missing frozen parser dependency: {AI_CHECKER_PATH}")
    require(sha(AI_CHECKER_PATH.read_bytes()) == AI_CHECKER_SHA256,
            "imported source tokenizer differs from the reviewed version")
    require(AI_HELPER_FIXTURE_PATH.is_file() and
            sha(AI_HELPER_FIXTURE_PATH.read_bytes()) == AI_HELPER_FIXTURE_SHA256,
            "imported parser helper fixture differs from the reviewed version")
    spec = importlib.util.spec_from_file_location("al_native_ai_source_parser", AI_CHECKER_PATH)
    require(spec is not None and spec.loader is not None, "cannot load source tokenizer")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


AI = load_ai_checker()


def token_count(tokens: list[str], needle: list[str]) -> int:
    if not needle:
        return 0
    return sum(tokens[i:i + len(needle)] == needle for i in range(len(tokens) - len(needle) + 1))


def require_tokens(actual: str | list[str], expected: str, label: str) -> None:
    got = AI.rust_tokens(actual) if isinstance(actual, str) else actual
    want = AI.rust_tokens(expected)
    require(got == want, f"{label} token structure changed")


def extract_item_tokens(source: str, prefix: str, label: str) -> list[str]:
    tokens = AI.rust_tokens(source)
    prefix_tokens = AI.rust_tokens(prefix)
    hits = [i for i in range(len(tokens) - len(prefix_tokens) + 1)
            if tokens[i:i + len(prefix_tokens)] == prefix_tokens]
    require(len(hits) == 1, f"expected one `{label}` item, found {len(hits)}")
    start = hits[0]
    brace = start + len(prefix_tokens) - 1 if prefix_tokens[-1] == "{" else next(
        (i for i in range(start + len(prefix_tokens), len(tokens)) if tokens[i] == "{"), None)
    require(brace is not None, f"`{label}` has no body")
    depth = 0
    for i in range(brace, len(tokens)):
        if tokens[i] == "{":
            depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[start:i + 1]
    raise AuditError(f"unclosed `{label}` item")


def extract_body_tokens(source: str, prefix: str, label: str) -> list[str]:
    item = extract_item_tokens(source, prefix, label)
    try:
        brace = item.index("{")
    except ValueError as exc:
        raise AuditError(f"`{label}` has no body") from exc
    return item[brace:]


def item_by_prefix(source: str, prefix: str, expected: str, label: str) -> None:
    item = extract_item_tokens(source, prefix, label)
    require(item == AI.rust_tokens(expected), f"{label} source item changed")


def body_by_prefix(source: str, prefix: str, expected: str, label: str) -> None:
    body = extract_body_tokens(source, prefix, label)
    require(body == AI.rust_tokens(expected), f"{label} source body changed")


def reject_trust_attribute_attached_to(source: str, prefix: str, label: str) -> None:
    masked = AI.mask_noncode(source)
    start = masked.find(prefix)
    require(start >= 0 and masked.find(prefix, start + 1) < 0,
            f"cannot uniquely locate `{label}` attribute region")
    previous_close = source.rfind("}", 0, start)
    attributes = source[previous_close + 1:start]
    require(re.search(r"#\s*\[\s*(?:trusted|assume|axiom|extern_spec)\b", attributes) is None,
            f"`{label}` has an unreviewed proof/trust attribute")


def mir_has(mir: str, snippet: str, label: str, *, count: int = 1) -> None:
    got = AI.rust_tokens(mir)
    want = AI.rust_tokens(snippet)
    actual = token_count(got, want)
    require(actual == count, f"MIR `{label}` expected {count} occurrence(s) of {snippet!r}, found {actual}")


def mir_blocks(mir: str, label: str) -> dict[tuple[int, bool], str]:
    """Return basic-block bodies keyed by (block number, cleanup flag)."""
    lines = mir.splitlines()
    blocks: dict[tuple[int, bool], str] = {}
    index = 0
    while index < len(lines):
        match = re.fullmatch(r"    bb(\d+)( \(cleanup\))?: \{", lines[index])
        if not match:
            index += 1
            continue
        key = (int(match.group(1)), match.group(2) is not None)
        require(key not in blocks, f"MIR `{label}` duplicates block bb{key[0]}")
        index += 1
        body_lines = []
        while index < len(lines) and lines[index] != "    }":
            body_lines.append(lines[index])
            index += 1
        require(index < len(lines), f"MIR `{label}` has unterminated block bb{key[0]}")
        blocks[key] = "\n".join(body_lines)
        index += 1
    require(bool(blocks), f"MIR `{label}` has no basic blocks")
    return blocks


def require_block_has(blocks: dict[tuple[int, bool], str], block: int,
                      snippet: str, label: str, *, cleanup: bool = False) -> None:
    key = (block, cleanup)
    require(key in blocks, f"MIR `{label}` is missing {'cleanup ' if cleanup else ''}bb{block}")
    actual = AI.rust_tokens(blocks[key])
    expected = AI.rust_tokens(snippet)
    require(token_count(actual, expected) == 1,
            f"MIR `{label}` bb{block} does not contain exactly one `{snippet}`")


def require_block_lacks(blocks: dict[tuple[int, bool], str], block: int,
                        snippet: str, label: str, *, cleanup: bool = False) -> None:
    key = (block, cleanup)
    require(key in blocks, f"MIR `{label}` is missing {'cleanup ' if cleanup else ''}bb{block}")
    actual = AI.rust_tokens(blocks[key])
    expected = AI.rust_tokens(snippet)
    require(token_count(actual, expected) == 0,
            f"MIR `{label}` bb{block} contains misplaced `{snippet}`")


def audit_selected_mir_headers(mir_sources: dict[str, str]) -> dict[str, Any]:
    require(set(mir_sources) == set(MIR_HEADER_PATTERNS),
            "selected MIR body label set differs from the reviewed 19 symbols")
    normalized_symbols = []
    for label, source in mir_sources.items():
        headers = re.findall(r"(?m)^fn ([^\n]+)$", source)
        require(len(headers) == 1, f"selected MIR `{label}` must contain exactly one top-level function definition")
        header = f"fn {headers[0]}"
        require(re.fullmatch(MIR_HEADER_PATTERNS[label], header) is not None,
                f"selected MIR `{label}` definition header names the wrong function body: {header}")
        # Remove only rustc's source-location annotation from impl paths, then
        # require that no two selected files define the same callable symbol.
        symbol = header.split("(", 1)[0]
        symbol = re.sub(r"::<impl at [^>]+>", "", symbol)
        normalized_symbols.append(symbol)
    require(len(normalized_symbols) == len(set(normalized_symbols)),
            "selected MIR definition headers contain duplicate callable symbols")
    return {"selected_definition_header_count": len(normalized_symbols),
            "selected_definition_headers_unique": True}


def path_for_receipt(value: str, expected_relative: str, expected_absolute: pathlib.Path, label: str) -> pathlib.Path:
    require(value == expected_relative, f"receipt path literal for `{label}` changed")
    relative = pathlib.Path(value)
    require(not relative.is_absolute(), f"receipt path `{label}` must remain relative")
    resolved = (ROOT / relative).resolve()
    require(resolved == expected_absolute.resolve(), f"receipt path `{label}` resolves to a different input")
    require(resolved.is_file(), f"receipt input `{label}` is missing")
    return resolved


REVIEWED_PRODUCTION_MANIFEST_SHA256 = 'e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306'

def load_bundle() -> dict[str, Any]:
    receipt = json.loads(CAPTURE_PATH.read_text())
    native_source = (ROOT / "native.rs").read_text()
    native_test_source = (ROOT / "native-test/tests/clone_witness.rs").read_text()
    native_test_log = (ROOT / "native-test/native-run.log").read_text()
    native_manifest = (ROOT / "native-test/Cargo.toml").read_text()
    native_lock = (ROOT / "native-test/Cargo.lock").read_text()
    capture_script = (ROOT / "capture-native.sh").read_text()
    production_manifest = (CRATE_ROOT / "Cargo.toml").read_text()
    production_source = (CRATE_ROOT / "src/bytes.rs").read_text()
    shared_record_source = (CRATE_ROOT / "src/bytes/shared_record.rs").read_text()
    ref_count_source = (CRATE_ROOT / "src/ref_count_ops.rs").read_text()
    loom_source = (CRATE_ROOT / "src/loom.rs").read_text()
    mir_sources = {}
    for label, filename in MIR_FILES.items():
        mir_sources[label] = (MIR_DIR / filename).read_text()
    return {
        "capture": receipt,
        "reviewed_production_manifest": (ROOT / "reviewed-production-inputs.json").read_text(),
        "production_source_inputs": {p.relative_to(CRATE_ROOT).as_posix():p.read_text() for p in (CRATE_ROOT / "src").rglob("*") if p.is_file()},
        "production_lock": (CRATE_ROOT / "Cargo.lock").read_text(),
        "native_source": native_source,
        "native_test_source": native_test_source,
        "native_test_log": native_test_log,
        "native_manifest": native_manifest,
        "native_lock": native_lock,
        "capture_script": capture_script,
        "production_manifest": production_manifest,
        "production_source": production_source,
        "shared_record_source": shared_record_source,
        "bytes_record_source": (CRATE_ROOT / "src/bytes/bytes_record.rs").read_text(),
        "native_field_profile_source": (ROOT / "native-field-profile.rs").read_text(),
        "native_field_profile_log": (ROOT / "native-field-profile.log").read_text(),
        "ref_count_source": ref_count_source,
        "loom_source": loom_source,
        "rustc_text": (MIR_DIR / "rustc-version.txt").read_text(),
        "cargo_text": (MIR_DIR / "cargo-version.txt").read_text(),
        "mir_sources": mir_sources,
    }


def audit_paths_and_receipt(data: dict[str, Any]) -> dict[str, Any]:
    receipt = data["capture"]
    require(receipt.get("stage") == "2-2-004.ElaborateDrops.after.mir", "wrong captured MIR stage")
    require(data["rustc_text"] == EXPECTED_RUSTC and receipt.get("rustc_version") == EXPECTED_RUSTC.rstrip("\n"),
            "native MIR toolchain is not the pinned rustc/host")
    require(data["cargo_text"] == EXPECTED_CARGO and receipt.get("cargo_version") == EXPECTED_CARGO.rstrip("\n"),
            "native MIR cargo version changed")

    paths = {
        "native_source": ("native.rs", ROOT / "native.rs", "native_source_sha256", data["native_source"]),
        "capture_script": ("capture-native.sh", ROOT / "capture-native.sh", "capture_script_sha256", data["capture_script"]),
        "native_manifest": ("native-test/Cargo.toml", ROOT / "native-test/Cargo.toml", "native_manifest_sha256", data["native_manifest"]),
        "native_lock": ("native-test/Cargo.lock", ROOT / "native-test/Cargo.lock", "native_lock_sha256", data["native_lock"]),
        "native_test_source": ("native-test/tests/clone_witness.rs", ROOT / "native-test/tests/clone_witness.rs", "native_test_source_sha256", data["native_test_source"]),
        "production_manifest": ("../../../Cargo.toml", CRATE_ROOT / "Cargo.toml", "production_manifest_sha256", data["production_manifest"]),
        "production_source": ("../../../src/bytes.rs", CRATE_ROOT / "src/bytes.rs", "production_source_sha256", data["production_source"]),
        "native_test_log": ("native-test/native-run.log", ROOT / "native-test/native-run.log", "native_test_log_sha256", data["native_test_log"]),
    }
    resolved = {}
    for key, (canonical, expected_abs, hash_key, content) in paths.items():
        actual = path_for_receipt(receipt.get(key, ""), canonical, expected_abs, key)
        require(receipt.get(hash_key) == sha(content.encode()), f"receipt hash for `{key}` does not match its resolved input")
        resolved[key] = str(actual)

    test_manifest = tomllib.loads(data["native_manifest"])
    require(test_manifest == EXPECTED_MANIFEST, "native test manifest package/route/dependency surface changed")
    native_path = (NATIVE_TEST_DIR / test_manifest["lib"]["path"]).resolve()
    bytes_path = (NATIVE_TEST_DIR / test_manifest["dependencies"]["bytes"]["path"]).resolve()
    require(native_path == (ROOT / "native.rs").resolve(), "native-test lib path no longer selects this native.rs")
    require(bytes_path == CRATE_ROOT.resolve(), "native-test dependency route no longer selects the probed bytes crate")
    production_manifest = tomllib.loads(data["production_manifest"])
    require(production_manifest.get("package", {}).get("name") == "bytes" and
            production_manifest.get("package", {}).get("version") == "1.11.1",
            "resolved production manifest is not bytes 1.11.1")

    lock = tomllib.loads(data["native_lock"])
    require(lock.get("version") == 4, "native lockfile format changed")
    packages = {(p.get("name"), p.get("version")) for p in lock.get("package", [])}
    require(("bytes", "1.11.1") in packages and
            ("bytes-promotable-reclone-native", "0.0.0") in packages,
            "native lockfile does not identify the probe and bytes crate")
    require(receipt.get("native_lock_sha256") == sha(data["native_lock"].encode()),
            "native lockfile hash changed")

    rows = receipt.get("selected")
    require(isinstance(rows, list) and len(rows) == len(MIR_FILES), "MIR receipt does not contain the complete selected set")
    by_label = {row.get("label"): row for row in rows}
    require(len(by_label) == len(rows) and set(by_label) == set(MIR_FILES), "MIR receipt labels changed or duplicated")
    for label, filename in MIR_FILES.items():
        row = by_label[label]
        expected_rel = f"native-mir/{filename}"
        path = path_for_receipt(row.get("path", ""), expected_rel, MIR_DIR / filename, f"MIR:{label}")
        require(row.get("sha256") == sha(data["mir_sources"][label].encode()),
                f"MIR receipt hash for `{label}` does not match its resolved input")
        require(path.read_text() == data["mir_sources"][label], f"loaded MIR text for `{label}` differs from its resolved file")

    commands = receipt.get("commands", [])
    require(commands == [
        "cargo test --locked --offline --manifest-path native-test/Cargo.toml -- --nocapture",
        "cargo rustc --locked --offline --manifest-path native-test/Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/client> -Zmir-opt-level=0 -Zidentify-regions=yes",
        "cargo rustc --locked --offline --manifest-path ../../../Cargo.toml --lib -- -Zdump-mir=all -Zdump-mir-dir=<scratch/production> -Zmir-opt-level=0 -Zidentify-regions=yes",
    ], "native build/capture command route changed")
    script = data["capture_script"]
    for literal in ("expected_rustc='rustc 1.98.0-nightly (91fe22da8 2026-06-21)'",
                    "CARGO_INCREMENTAL=0", "-Zdump-mir=all", "ElaborateDrops.after.mir",
                    "native-test/Cargo.toml", "../../../Cargo.toml"):
        require(literal in script, f"capture script is missing pinned procedure marker {literal!r}")
    require("why3" not in script.lower() and "creusot" not in script.lower(),
            "native capture script must not invoke proof tooling")
    return {"resolved_receipt_inputs": resolved, "selected_mir_count": len(rows),
            "selected_production_mir_count": len(rows) - 1,
            "toolchain_pinned": True, "manifest_literal_routes_pinned": True}


def audit_native_client(data: dict[str, Any]) -> dict[str, Any]:
    require_tokens(data["native_source"], EXPECTED_NATIVE_SOURCE, "closed native automatic client")
    require_tokens(data["native_test_source"], EXPECTED_NATIVE_TEST, "nonempty native test source")
    require(re.search(r"test promoted_inner_drop_preserves_readable_original \.\.\. ok", data["native_test_log"]) is not None,
            "native run log does not show the automatic witness passing")
    require(re.search(r"test result: ok\. 1 passed; 0 failed", data["native_test_log"]) is not None,
            "native run log does not show exactly one integration test passing")
    mir=data["mir_sources"]["client"]
    require("fn promoted_reclone_scope(_1: Box<[u8]>) -> Vec<u8> {" in mir,
            "captured MIR entrypoint/signature differs from native source")
    blocks=mir_blocks(mir,"client")
    require(set(blocks)=={(i,False) for i in range(11)}|{(i,True) for i in range(11,16)},
            "native client block/cleanup topology changed")
    normal="\n".join(blocks[(i,False)] for i in range(11))
    calls=[line.strip() for line in normal.splitlines() if "-> [return:" in line]
    expected_calls=[
        "_2 = <bytes::Bytes as From<Box<[u8]>>>::from(move _3) -> [return: bb1, unwind: bb13];",
        "_5 = <bytes::Bytes as Clone>::clone(move _6) -> [return: bb2, unwind: bb12];",
        "_7 = <bytes::Bytes as Clone>::clone(move _8) -> [return: bb3, unwind: bb11];",
        "drop(_7) -> [return: bb4, unwind: bb11];",
        "drop(_5) -> [return: bb5, unwind: bb12];",
        "_11 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _12) -> [return: bb6, unwind: bb12];",
        "_9 = slice::<impl [u8]>::to_vec(move _10) -> [return: bb7, unwind: bb12];",
        "drop(_2) -> [return: bb9, unwind: bb14];",
    ]
    require(calls==expected_calls,"native MIR normal call/drop trace changed")
    for marker in ("debug original => _2;", "debug first => _5;", "debug second => _7;",
                   "debug observed => _9;"):
        require(marker in mir, f"native local identity changed: {marker}")
    require_block_has(blocks,0,expected_calls[0],"client")
    require_block_has(blocks,1,"_6 = &'_ _2;", "client")
    require_block_has(blocks,1,expected_calls[1],"client")
    require_block_has(blocks,2,"_8 = &'_ _2;", "client")
    require_block_has(blocks,2,expected_calls[2],"client")
    require_block_has(blocks,3,expected_calls[3],"client")
    require_block_has(blocks,4,expected_calls[4],"client")
    require_block_has(blocks,5,"_13 = &'_ _2;", "client")
    require_block_has(blocks,5,"_12 = &'_ (*_13);", "client")
    require_block_has(blocks,5,expected_calls[5],"client")
    require_block_has(blocks,6,expected_calls[6],"client")
    require_block_has(blocks,7,"_0 = move _9;", "client")
    require_block_has(blocks,7,"goto -> bb8;", "client")
    require_block_has(blocks,8,expected_calls[7],"client")
    require_block_has(blocks,9,"goto -> bb10;", "client")
    require_block_has(blocks,10,"return;", "client")
    require("cleanup(" not in normal and normal.count("drop(")==3,
            "native client must have exactly three automatic handle drops and no explicit cleanup")
    cleanup_drops=[line.strip() for key,body in blocks.items() if key[1] for line in body.splitlines() if "drop(" in line]
    expected_cleanup_drops=[
        "drop(_5) -> [return: bb12, unwind terminate(cleanup)];",
        "drop(_2) -> [return: bb14, unwind terminate(cleanup)];",
    ]
    require(cleanup_drops==expected_cleanup_drops,
            "exceptional cleanup drop order/edges changed")
    for block, expected in ((11,expected_cleanup_drops[0]), (12,expected_cleanup_drops[1])):
        require_block_has(blocks,block,expected,"client",cleanup=True)
    require_block_has(blocks,13,"goto -> bb14;","client",cleanup=True)
    require_block_has(blocks,14,"goto -> bb15;","client",cleanup=True)
    require_block_has(blocks,15,"resume;","client",cleanup=True)
    return {"native_client_source_closed":True,"normal_trace":[
        "Bytes::from(Box<[u8]>)","first Clone::clone(original)","second Clone::clone(original)",
        "second automatic Drop","first automatic Drop","AsRef(original).to_vec()",
        "evaluate return Vec","root normal Drop","return"],
        "nonempty_input_tested":True,"normal_implicit_handle_drop":True,
        "normal_drop_edges":[{"block":"bb3","place":"_7","owner":"second","successor":"bb4","unwind":"bb11"},
                             {"block":"bb4","place":"_5","owner":"first","successor":"bb5","unwind":"bb12"},
                             {"block":"bb8","place":"_2","owner":"original","successor":"bb9","unwind":"bb14"}],
        "return_evaluation":{"block":"bb7","statement":"_0 = move _9;"},
        "cleanup_topology":{"first_then_original":["bb11","bb12"],"drop_original_only":["bb12","bb14"],
                            "from_before_owner_init":["bb13","bb14","bb15"]},
        "exceptional_cleanup_drops_retained":len(cleanup_drops),"unwind_proved":False}


def audit_production_sources(data: dict[str, Any]) -> dict[str, Any]:
    source = data["production_source"]
    body_by_prefix(source, "fn from(slice: Box<[u8]>) -> Bytes", EXPECTED_FROM_BOX_BODY, "From<Box<[u8]>> for Bytes")
    body_by_prefix(source, "fn clone(&self) -> Bytes", EXPECTED_CLONE_BODY, "Clone for Bytes")
    body_by_prefix(source, "pub fn cleanup(self)", EXPECTED_CLEANUP_BODY, "Bytes::cleanup")
    body_by_prefix(source, "fn as_ref(&self) -> &[u8]", EXPECTED_AS_REF_BODY, "AsRef<[u8]> for Bytes")
    body_by_prefix(source, "fn as_slice(&self) -> &[u8]", EXPECTED_AS_SLICE_BODY, "Bytes::as_slice")
    item_by_prefix(source, "impl Drop for Bytes", EXPECTED_BYTES_DROP_IMPL, "Drop for Bytes")
    body_by_prefix(source, "unsafe fn promotable_even_clone", EXPECTED_EVEN_CLONE_BODY, "promotable_even_clone")
    body_by_prefix(source, "unsafe fn promotable_odd_clone", EXPECTED_ODD_CLONE_BODY, "promotable_odd_clone")
    body_by_prefix(source, "unsafe fn shallow_clone_vec", EXPECTED_SHALLOW_CLONE_VEC_BODY, "shallow_clone_vec")
    body_by_prefix(source, "unsafe fn shallow_clone_arc", EXPECTED_SHALLOW_CLONE_ARC_BODY, "shallow_clone_arc")
    body_by_prefix(source, "unsafe fn promotable_even_drop", EXPECTED_EVEN_DROP_BODY, "promotable_even_drop")
    body_by_prefix(source, "unsafe fn promotable_odd_drop", EXPECTED_ODD_DROP_BODY, "promotable_odd_drop")
    body_by_prefix(source, "unsafe fn shared_drop", EXPECTED_SHARED_DROP_BODY, "shared_drop")
    body_by_prefix(source, "unsafe fn release_shared", EXPECTED_RELEASE_SHARED_BODY, "release_shared")
    body_by_prefix(source, "unsafe fn free_shared", EXPECTED_FREE_SHARED_BODY, "free_shared")
    item_by_prefix(source, "impl Drop for Shared", EXPECTED_SHARED_DROP_IMPL, "Shared destructor")
    item_by_prefix(data["shared_record_source"], "pub(crate) struct Shared", EXPECTED_SHARED_RECORD, "Shared field profile")
    item_by_prefix(source, "static PROMOTABLE_EVEN_VTABLE", EXPECTED_PROMOTABLE_EVEN_VTABLE,
                   "promotable even clone/drop vtable")
    item_by_prefix(source, "static PROMOTABLE_ODD_VTABLE", EXPECTED_PROMOTABLE_ODD_VTABLE,
                   "promotable odd clone/drop vtable")
    item_by_prefix(source, "static SHARED_VTABLE", EXPECTED_SHARED_VTABLE,
                   "shared clone/drop vtable")
    trusted_spans = [
        ("fn from(slice: Box<[u8]>) -> Bytes", "From<Box<[u8]>> for Bytes"),
        ("fn clone(&self) -> Bytes", "Clone for Bytes"),
        ("pub fn cleanup(self)", "Bytes::cleanup"),
        ("fn as_ref(&self) -> &[u8]", "AsRef<[u8]> for Bytes"),
        ("fn as_slice(&self) -> &[u8]", "Bytes::as_slice"),
        ("impl Drop for Bytes", "Drop for Bytes"),
        ("unsafe fn promotable_even_clone", "promotable_even_clone"),
        ("unsafe fn promotable_odd_clone", "promotable_odd_clone"),
        ("unsafe fn shallow_clone_vec", "shallow_clone_vec"),
        ("unsafe fn shallow_clone_arc", "shallow_clone_arc"),
        ("unsafe fn promotable_even_drop", "promotable_even_drop"),
        ("unsafe fn promotable_odd_drop", "promotable_odd_drop"),
        ("unsafe fn shared_drop", "shared_drop"),
        ("unsafe fn release_shared", "release_shared"),
        ("unsafe fn free_shared", "free_shared"),
    ]
    for prefix, label in trusted_spans:
        reject_trust_attribute_attached_to(source, prefix, label)

    # The explicitly selected free_shared bypasses Shared::drop. Its body
    # requires the sole remaining field with possible glue to be drop-free.
    source_tokens = AI.rust_tokens(source)
    needs_drop = AI.rust_tokens("mem::needs_drop::<AtomicUsize>()")
    require(token_count(source_tokens, needs_drop) == 1, "AtomicUsize no-drop guard is absent or duplicated")
    shared = AI.rust_tokens(data["shared_record_source"])
    require(shared.count("Shared") == 1 and
            token_count(shared, AI.rust_tokens("buf : * mut u8")) == 1 and
            token_count(shared, AI.rust_tokens("cap : usize")) == 1 and
            token_count(shared, AI.rust_tokens("pub (crate) ref_cnt : AtomicUsize")) == 1,
            "Shared struct must retain the reviewed raw buffer, capacity, and atomic refcount fields")

    ref_counts = data["ref_count_source"]
    body_by_prefix(ref_counts, "pub(crate) fn try_increment", EXPECTED_TRY_INCREMENT_BODY, "ref_count_ops::try_increment")
    body_by_prefix(ref_counts, "pub(crate) fn increment", EXPECTED_INCREMENT_BODY, "ref_count_ops::increment")
    loom = data["loom_source"]
    impl = extract_item_tokens(loom, "impl<T> AtomicMut<T> for AtomicPtr<T>", "AtomicMut<AtomicPtr> impl")
    require(impl == AI.rust_tokens(EXPECTED_ATOMIC_MUT_IMPL), "native AtomicMut adapter changed")

    # The copied source functions are a closed reviewed surface. Explicit
    # trusted/assume/axiom annotations are rejected on their executable spans.
    require("fetch_add" not in data["loom_source"] and "Ordering::SeqCst" not in data["loom_source"],
            "default native AtomicMut source acquired an unreviewed effect")

    return {"constructor_clone_cleanup_read_source_exact": True,
            "even_odd_clone_and_drop_sources_exact": True,
            "shared_arc_promotion_and_cleanup_sources_exact": True,
            "promotable_parity_vtables_bind_clone_and_drop_callbacks": True,
            "shared_vtable_binds_existing_arc_clone_and_drop_callbacks": True,
            "shared_fields_and_drop_guard_checked": True,
            "native_atomic_mut_and_refcount_increment_sources_exact": True}


def audit_native_mir(data: dict[str, Any]) -> dict[str, Any]:
    mir = data["mir_sources"]
    header_audit = audit_selected_mir_headers(mir)
    # Box constructor: both pointer-parity vtables remain represented, and the
    # test source supplies the nonempty premise selecting these branches.
    from_box = mir["from_box"]
    for snippet in (
        "core::slice::<impl [u8]>::is_empty(move _4)",
        "_0 = bytes::Bytes::new()",
        "Box::<[u8]>::into_raw(move _10)",
        "PROMOTABLE_EVEN_VTABLE",
        "PROMOTABLE_ODD_VTABLE",
    ):
        require(snippet in from_box, f"From<Box> MIR lost expected constructor branch: {snippet}")
    require(from_box.count("Box::<[u8]>::into_raw") == 1, "From<Box> MIR must transfer the nonempty Box once")

    clone = mir["clone_impl"]
    require("((*_1).3: &bytes::Vtable)" in clone and "((*_7).0:" in clone and
            "((*_1).2: core::sync::atomic::Atomic<*mut ()>)" in clone and "((*_1).0: *const u8)" in clone and
            "((*_1).1: usize)" in clone,
            "Bytes Clone MIR no longer dispatches through the stored vtable and fields")
    require(clone.count("_0 = move _2(move _3, move _5, move _6)") == 1,
            "Bytes Clone MIR has multiple callback targets")

    cleanup = mir["cleanup"]
    require("ManuallyDrop::<bytes::Bytes>::new(move _3)" in cleanup and
            "((*_20).4: for<'a> unsafe fn" in cleanup and
            "_0 = move _13(move _14, move _18, move _19)" in cleanup,
            "Bytes::cleanup MIR no longer transfers ownership to the selected destructor callback")
    require("Bytes::drop" not in cleanup, "Bytes::cleanup MIR recursively routes through Drop")
    bytes_drop = mir["bytes_drop"]
    require("((*_7).4: for<'a> unsafe fn" in bytes_drop and "move _2(move _3, move _5, move _6)" in bytes_drop,
            "Bytes::drop MIR callback target changed")
    as_ref = mir["as_ref"]
    as_slice = mir["as_slice"]
    require("Bytes::as_slice" in as_ref, "AsRef MIR no longer delegates to Bytes::as_slice")
    require("core::slice::from_raw_parts" in as_slice and "((*_1).0: *const u8)" in as_slice and
            "((*_1).1: usize)" in as_slice,
            "Bytes::as_slice MIR no longer forms the selected borrowed range")

    # After first-clone promotion the original's atomic tag selects the ARC
    # arm on the second clone. Pin both parity-specific dispatchers, including
    # their real Acquire load and both the existing-ARC and first-promotion
    # successors.
    parity_clones = {
        "promotable_even_clone": {
            "blocks": set((i, False) for i in range(15)),
            "switch": "switchInt(move _10) -> [0: bb5, otherwise: bb2];",
            "arc_call": "_0 = shallow_clone_arc(move _12, move _14, move _15) -> [return: bb4, unwind continue];",
            "arc_block": 3, "arc_join": 4, "join": 14,
            "vec_call": "_0 = shallow_clone_vec(move _41, move _42, move _44, move _45, move _46) -> [return: bb13, unwind continue];",
            "vec_block": 12, "vec_join": 13,
        },
        "promotable_odd_clone": {
            "blocks": set((i, False) for i in range(13)),
            "switch": "switchInt(move _10) -> [0: bb4, otherwise: bb2];",
            "arc_call": "_0 = shallow_clone_arc(move _12, move _15, move _16) -> [return: bb3, unwind continue];",
            "arc_block": 2, "arc_join": 3, "join": 12,
            "vec_call": "_0 = shallow_clone_vec(move _38, move _39, move _41, move _43, move _44) -> [return: bb11, unwind continue];",
            "vec_block": 10, "vec_join": 11,
        },
    }
    for label, facts in parity_clones.items():
        clone_blocks = mir_blocks(mir[label], label)
        require(set(clone_blocks) == facts["blocks"], f"{label} block topology changed")
        require_block_has(clone_blocks, 0, "_6 = core::sync::atomic::Ordering::Acquire;", label)
        require_block_has(clone_blocks, 0,
                          "_4 = Atomic::<*mut ()>::load(move _5, move _6) -> [return: bb1, unwind continue];",
                          label)
        require_block_has(clone_blocks, 1, facts["switch"], label)
        require_block_has(clone_blocks, facts["arc_block"], facts["arc_call"], label)
        require_block_has(clone_blocks, facts["arc_join"], f"goto -> bb{facts['join']};", label)
        require_block_has(clone_blocks, facts["vec_block"], facts["vec_call"], label)
        require_block_has(clone_blocks, facts["vec_join"], f"goto -> bb{facts['join']};", label)
        require("compare_exchange" not in mir[label] and "Atomic::<*mut ()>::store" not in mir[label],
                f"{label} dispatcher gained an unreviewed pointer writer")

    shallow = mir["shallow_clone_vec"]
    mir_has(shallow, "_17 = Atomic::<usize>::new(const 2_usize)", "shallow_clone_vec", count=1)
    mir_has(shallow, "_7 = bytes::Shared { buf: move _8, cap: move _9, ref_cnt: move _17 }", "shallow_clone_vec", count=1)
    mir_has(shallow, "_30 = Atomic::<*mut ()>::compare_exchange(move _31, move _32, move _35, move _38, move _39)", "shallow_clone_vec", count=1)
    mir_has(shallow, "_38 = core::sync::atomic::Ordering::AcqRel", "shallow_clone_vec", count=1)
    mir_has(shallow, "_39 = core::sync::atomic::Ordering::Acquire", "shallow_clone_vec", count=1)
    mir_has(shallow, "switchInt(move _40) -> [0: bb15, 1: bb14, otherwise: bb13]", "shallow_clone_vec", count=1)
    require("_34 = copy _2;" in shallow and "_32 = copy _33;" in shallow and
            "_37 = copy _18;" in shallow and "_35 = copy _36;" in shallow,
            "strong CAS expected/new pointer operands are no longer original/new Shared pointers")
    blocks = mir_blocks(shallow, "shallow_clone_vec")
    # The compare_exchange result's discriminant drives these specific edges:
    # Ok (0) enters bb15 and Err (1) enters bb14. Check the effects in their
    # destination blocks and then walk the important normal edges to the
    # exact returned Bytes record or the loser cleanup/ARC callback.
    require_block_has(blocks, 12,
                      "switchInt(move _40) -> [0: bb15, 1: bb14, otherwise: bb13];",
                      "shallow_clone_vec")
    require_block_has(blocks, 15, "_41 = copy ((_30 as Ok).0: *mut ());",
                      "shallow_clone_vec")
    require_block_has(blocks, 15, "switchInt(move _43) -> [0: bb20, otherwise: bb16];",
                      "shallow_clone_vec")
    require_block_has(blocks, 16,
                      "_45 = core::ptr::eq::<()>(move _46, move _48) -> [return: bb17, unwind: bb32];",
                      "shallow_clone_vec")
    require_block_has(blocks, 17, "switchInt(move _45) -> [0: bb19, otherwise: bb18];",
                      "shallow_clone_vec")
    require_block_has(blocks, 18, "goto -> bb21;", "shallow_clone_vec")
    require_block_has(blocks, 19, "_49 = panic(const \"assertion failed: core::ptr::eq(actual, ptr)\") -> bb32;",
                      "shallow_clone_vec")
    require_block_has(blocks, 20, "goto -> bb21;", "shallow_clone_vec")
    require_block_has(blocks, 14, "_59 = copy ((_30 as Err).0: *mut ());",
                      "shallow_clone_vec")
    require_block_has(blocks, 14,
                      "_61 = copy _18;\n"
                      "        _60 = Box::<bytes::Shared>::from_raw(move _61) -> [return: bb23, unwind: bb32];",
                      "shallow_clone_vec")
    require_block_has(blocks, 23,
                      "_63 = move (*_60);\n"
                      "        _62 = core::mem::forget::<bytes::Shared>(move _63) -> [return: bb24, unwind: bb29];",
                      "shallow_clone_vec")
    require_block_has(blocks, 24,
                      "_66 = copy _59;\n"
                      "        _65 = move _66 as *mut bytes::Shared (PtrToPtr);\n"
                      "        _64 = copy _65;\n"
                      "        StorageDead(_66);\n"
                      "        StorageLive(_67);\n"
                      "        _67 = copy _4;\n"
                      "        StorageLive(_68);\n"
                      "        _68 = copy _5;\n"
                      "        _0 = shallow_clone_arc(move _64, move _67, move _68) -> [return: bb25, unwind: bb30];",
                      "shallow_clone_vec")
    require_block_has(blocks, 21,
                      "_55 = copy _18;\n"
                      "        _54 = move _55 as *mut () (PtrToPtr);\n"
                      "        _53 = copy _54;\n"
                      "        StorageDead(_55);\n"
                      "        _52 = Atomic::<*mut ()>::new(move _53) -> [return: bb22, unwind: bb32];",
                      "shallow_clone_vec")
    require_block_has(blocks, 22,
                      "_58 = const {alloc302: &Vtable};\n"
                      "        _57 = &'_ (*_58);\n"
                      "        _56 = &'_ (*_57);\n"
                      "        _0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 };",
                      "shallow_clone_vec")
    require_block_has(blocks, 22, "goto -> bb27;", "shallow_clone_vec")
    require_block_has(blocks, 27, "goto -> bb28;", "shallow_clone_vec")
    require_block_has(blocks, 28, "return;", "shallow_clone_vec")
    require("alloc302 (static: bytes::SHARED_VTABLE" in shallow and
            "alloc307 (fn: shared_drop)" in shallow,
            "CAS success return record does not resolve to the SHARED_VTABLE/shared_drop allocation")
    # These operations are unique in this MIR body and may occur only on the
    # loser route. Keep their exact ordering and prohibit the winner from
    # acquiring any loser-only effect, even if global markers still exist.
    mir_has(shallow, "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 }",
            "shallow_clone_vec returned Bytes record", count=1)
    mir_has(shallow, "Box::<bytes::Shared>::from_raw(move _61)", "shallow_clone_vec loser Box recovery", count=1)
    mir_has(shallow, "core::mem::forget::<bytes::Shared>(move _63)", "shallow_clone_vec loser payload forget", count=1)
    mir_has(shallow, "_0 = shallow_clone_arc(move _64, move _67, move _68)", "shallow_clone_vec loser ARC return", count=1)
    for block in (15, 16, 17, 18, 19, 20, 21, 22, 27, 28):
        require_block_lacks(blocks, block, "Box::<bytes::Shared>::from_raw(move _61)",
                            "shallow_clone_vec")
        require_block_lacks(blocks, block, "core::mem::forget::<bytes::Shared>(move _63)",
                            "shallow_clone_vec")
        require_block_lacks(blocks, block, "_0 = shallow_clone_arc(move _64, move _67, move _68)",
                            "shallow_clone_vec")
    record_snippet = "_0 = bytes::Bytes { ptr: move _50, len: move _51, data: move _52, vtable: move _56 }"
    for (block, cleanup_block) in blocks:
        if block != 22 or cleanup_block:
            require_block_lacks(blocks, block, record_snippet, "shallow_clone_vec",
                                cleanup=cleanup_block)
    require_block_lacks(blocks, 15, "Box::<bytes::Shared>::from_raw(move _61)",
                        "shallow_clone_vec")
    require_block_lacks(blocks, 15, "core::mem::forget::<bytes::Shared>(move _63)",
                        "shallow_clone_vec")
    require_block_lacks(blocks, 15, "_0 = shallow_clone_arc(move _64, move _67, move _68)",
                        "shallow_clone_vec")
    for forbidden in ("compare_exchange_weak", "Atomic::<*mut ()>::store", "Atomic::<*mut ()>::swap"):
        require(forbidden not in shallow, f"shallow_clone_vec MIR contains unreviewed writer {forbidden}")
    require(shallow.index("Atomic::<usize>::new(const 2_usize)") <
            shallow.index("Atomic::<*mut ()>::compare_exchange"),
            "new Shared strong-count-two initialization does not precede CAS")

    arc = mir["shallow_clone_arc"]
    arc_blocks = mir_blocks(arc, "shallow_clone_arc")
    require(set(arc_blocks) == {(0, False), (1, False), (2, False)},
            "existing-ARC clone helper acquired new control-flow/allocation blocks")
    arc_calls = [line.strip() for body in arc_blocks.values() for line in body.splitlines()
                 if "-> [return:" in line]
    require(arc_calls == [
        "_4 = increment(move _5) -> [return: bb1, unwind continue];",
        "_9 = Atomic::<*mut ()>::new(move _10) -> [return: bb2, unwind continue];",
    ], "existing-ARC clone helper call sequence changed")
    require_block_has(arc_blocks, 0, arc_calls[0], "shallow_clone_arc")
    require_block_has(arc_blocks, 1, arc_calls[1], "shallow_clone_arc")
    require_block_has(arc_blocks, 2,
                      "_0 = bytes::Bytes { ptr: move _7, len: move _8, data: move _9, vtable: move _13 };",
                      "shallow_clone_arc")
    require("alloc302 (static: bytes::SHARED_VTABLE" in arc and
            "alloc307 (fn: shared_drop)" in arc,
            "existing-ARC returned handle is no longer bound to SHARED_VTABLE/shared_drop")
    for forbidden in ("Atomic::<*mut ()>::compare_exchange", "compare_exchange_weak",
                      "Box::<bytes::Shared>::new", "Box::<bytes::Shared>::new_uninit", "dealloc"):
        require(forbidden not in arc, f"existing-ARC clone helper acquired forbidden `{forbidden}`")
    increment = mir["ref_count_increment"]
    require("try_increment(move _5)" in increment and "Result::<usize, usize>::is_err" in increment and
            "abort()" in increment,
            "selected refcount increment MIR changed")
    require("compare_exchange" not in increment and "Box::" not in increment and "dealloc" not in increment,
            "guarded refcount increment MIR acquired a pointer CAS/allocation/free")

    for label in ("promotable_even_drop", "promotable_odd_drop"):
        drop_mir = mir[label]
        require("with_mut" in drop_mir and "promotable_" in drop_mir and "closure" in drop_mir,
                f"{label} MIR no longer sends data through AtomicMut callback")
    shared_drop = mir["shared_drop"]
    require("with_mut" in shared_drop and "release_shared" not in shared_drop,
            "shared_drop MIR callback adapter changed unexpectedly")
    release = mir["release_shared"]
    release_blocks = mir_blocks(release, "release_shared")
    require(set(release_blocks) == {(i, False) for i in range(7)},
            "release_shared MIR branch topology changed")
    release_calls = [line.strip() for body in release_blocks.values() for line in body.splitlines()
                     if "-> [return:" in line]
    require(release_calls == [
        "_4 = Atomic::<usize>::fetch_sub(move _5, const 1_usize, move _6) -> [return: bb1, unwind continue];",
        "_8 = Atomic::<usize>::load(move _9, move _10) -> [return: bb4, unwind continue];",
        "_11 = free_shared(move _12) -> [return: bb5, unwind continue];",
    ], "release_shared MIR decrement/acquire/free call sequence changed")
    require_block_has(release_blocks, 0, "_6 = core::sync::atomic::Ordering::Release;", "release_shared")
    require_block_has(release_blocks, 1,
                      "switchInt(move _3) -> [0: bb3, otherwise: bb2];", "release_shared")
    require_block_has(release_blocks, 2, "goto -> bb6;", "release_shared")
    require_block_lacks(release_blocks, 2, "free_shared(move _12)", "release_shared")
    require_block_has(release_blocks, 3, "_10 = core::sync::atomic::Ordering::Acquire;", "release_shared")
    require_block_has(release_blocks, 3, release_calls[1], "release_shared")
    require_block_has(release_blocks, 4, release_calls[2], "release_shared")
    require_block_has(release_blocks, 5, "goto -> bb6;", "release_shared")
    require_block_has(release_blocks, 6, "return;", "release_shared")
    free = mir["free_shared"]
    require("dealloc" in free and "Layout::from_size_align" in free and "Layout::new::<bytes::Shared>()" in free,
            "free_shared MIR no longer frees buffer and Shared allocation")
    require(free.count("dealloc") == 2, "free_shared MIR must perform exactly the two reviewed deallocations")
    ptr_map = mir["ptr_map"]
    require("PointerExposeProvenance" in ptr_map and "PointerWithExposedProvenance" in ptr_map,
            "ptr_map MIR no longer exposes and reconstructs pointer address")
    with_mut = mir["atomic_with_mut"]
    require("f(self.get_mut())" in with_mut or "get_mut" in with_mut,
            "AtomicMut::with_mut MIR no longer invokes closure on the actual atomic storage")

    return {"native_client_mir_order_exact": True,
            **header_audit,
            "selected_mir_count_including_client": len(mir),
            "selected_production_mir_count": len(mir) - 1,
            "new_shared_count_two_precedes_strong_CAS": True,
            "CAS_success_and_loser_blocks_and_edges_checked": True,
            "release_acquire_free_path_checked": True,
            "promotable_drop_arc_paths_retained_in_source_and_outer_MIR": True}


def audit_reviewed_production_inputs(data: dict[str, Any]) -> dict[str, Any]:
    raw=data["reviewed_production_manifest"]
    require(sha(raw.encode())==REVIEWED_PRODUCTION_MANIFEST_SHA256,
            "reviewed production source manifest changed")
    manifest=json.loads(raw)
    require(manifest.get("schema")=="reviewed-original-production-inputs-v1" and
            manifest.get("base_commit")=="361c7cd261507ac0a705b3b836f73240070891c6" and
            manifest.get("crate")=="bytes/1.11.1",
            "reviewed production ancestry identity changed")
    files=manifest["files"]
    source_paths={p for p in files if p.startswith("src/")}
    require(len(source_paths)==61 and set(files)==source_paths|{"Cargo.toml","Cargo.lock"},
            "reviewed production manifest path inventory changed")
    inputs=data["production_source_inputs"]
    require(set(inputs)==source_paths,"actual native production source inventory changed")
    for path in source_paths:
        require(sha(inputs[path].encode())==files[path],f"reviewed native production input changed: {path}")
    overrides={"src/bytes.rs":data["production_source"],
        "src/bytes/bytes_record.rs":data["bytes_record_source"],
        "src/bytes/shared_record.rs":data["shared_record_source"],
        "src/loom.rs":data["loom_source"],"src/ref_count_ops.rs":data["ref_count_source"],
        "Cargo.toml":data["production_manifest"],"Cargo.lock":data["production_lock"]}
    for path,source in overrides.items():
        require(sha(source.encode())==files[path],f"loaded native input differs from reviewed source identity: {path}")
    return {"manifest_sha256":REVIEWED_PRODUCTION_MANIFEST_SHA256,
            "base_commit":manifest["base_commit"],"source_files":len(source_paths),
            "manifest_and_lock_pinned":True,"global_import_macro_and_include_bindings_frozen":True}


def audit_terminal_field_profile(data: dict[str, Any]) -> dict[str, Any]:
    require_tokens(data["bytes_record_source"], EXPECTED_BYTES_RECORD,
                   "exact Bytes field/configuration profile")
    require_tokens(data["native_field_profile_source"], EXPECTED_NATIVE_FIELD_PROFILE,
                   "native field no-drop compile assertions")
    receipt=data["capture"]
    require(receipt.get("native_field_profile_source")=="native-field-profile.rs" and
            receipt.get("native_field_profile_log")=="native-field-profile.log",
            "field-profile capture routes changed")
    require(receipt.get("native_field_profile_sha256")==sha(data["native_field_profile_source"].encode()) and
            receipt.get("native_field_profile_log_sha256")==sha(data["native_field_profile_log"].encode()),
            "native field-profile source/log hashes differ")
    require(data["native_field_profile_log"]=="native Bytes fields have no independent drop glue\n",
            "native field-profile compiled executable did not pass")
    require('rustc --edition=2021 native-field-profile.rs -o "$scratch_dir/native-field-profile"' in data["capture_script"] and
            '"$scratch_dir/native-field-profile" > native-field-profile.log' in data["capture_script"],
            "native field-profile capture procedure changed")
    return {"exact_native_Bytes_field_profile_checked":True,
            "default_native_fields":["*const u8","usize","AtomicPtr<()>","&'static Vtable"],
            "compile_time_no_independent_field_drop_glue":True,
            "cfg_only_proof_Ghost_fields_erased":True,
            "reference_no_drop_for_any_referent_is_generic_language_boundary":True}


def audit_bundle(data: dict[str, Any]) -> dict[str, Any]:
    reviewed = audit_reviewed_production_inputs(data)
    paths = audit_paths_and_receipt(data)
    client = audit_native_client(data)
    source = audit_production_sources(data)
    mir = audit_native_mir(data)
    profile = audit_terminal_field_profile(data)
    return {
        "status": "pass",
        "checker_scope": "closed default-native witness/source/MIR correspondence; no mathematical proof claim",
        "paths_and_capture": paths,
        "reviewed_production_inputs": reviewed,
        "client": client,
        "production_source": source,
        "native_mir": mir,
        "terminal_field_profile": profile,
        "limitations": [
            "native output tests are execution corroboration, not a bytes proof",
            "no claim that test allocations exercise both address parity branches",
            "the nested promotable-drop closure bodies are source-checked; selected MIR records the outer AtomicMut closure invocation",
            "compiler/MIR adequacy, Rust memory model, allocator behavior, and pointer provenance remain outside this checker",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    try:
        result = audit_bundle(load_bundle())
    except (AuditError, OSError, ValueError, KeyError, TypeError) as exc:
        print(f"native correspondence audit failed: {exc}", file=sys.stderr)
        return 1
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
