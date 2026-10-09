#!/usr/bin/env python3
"""Independent, fail-closed source/effect check for the actual Shared client.

This is intentionally a small closed-language checker, not a Rust parser or a
proof of the generic cursor, event, physical-free, or native MIR TCB. A passing
result must be derived from source text and exact extracted source spans; a
stored hash/receipt is never treated as correspondence evidence by itself.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import sys
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = pathlib.Path(__file__).resolve().parents[3]
DRIVER = ROOT / "src" / "native_client.rs"
SHADOW = ROOT / "src" / "public_shared.rs"
LIB_SOURCE = ROOT / "src" / "lib.rs"
BYTES_SOURCE = CRATE_ROOT / "src" / "bytes.rs"
SHARED_RECORD_SOURCE = CRATE_ROOT / "src" / "bytes" / "shared_record.rs"
REFCOUNT_SOURCE = CRATE_ROOT / "src" / "ref_count_ops.rs"
GENERATED = ROOT / "generated"
EXPECTED_SHADOW_BODIES_PATH = ROOT / "fixtures" / "expected-shadow-bodies.json"

EXPECTED_NATIVE_CLIENT = r'''//! Native source client mirrored by the scoped proof shadow in `public_shared.rs`.
//!
//! Keep this body as ordinary public-API use: construction remains
//! `Bytes::from`, clones remain `Clone::clone(&self)`, and explicit cleanup
//! consumes the original handles. The independent correspondence checker
//! compares this closed client with the proof shadow and the extracted
//! production constructor, trait, vtable, and callback sources.

use super::*;
use alloc::vec::Vec;

#[cfg_attr(creusot, requires(input@.len() < creusot_std::std::vec::capacity_model(input)))]
#[cfg_attr(creusot, ensures(result@ == input@))]
pub(crate) fn actual_public_shared_driver(input: Vec<u8>) -> Vec<u8> {
    let first = Bytes::from(input);
    let second = first.clone();
    let third = first.clone();
    third.cleanup();
    first.cleanup();
    let fourth = second.clone();
    let borrowed = AsRef::<[u8]>::as_ref(&fourth);
    second.cleanup();
    let observed = borrowed.to_vec();
    fourth.cleanup();
    observed
}'''

EXPECTED_EVENTS = [
    {"kind": "from_vec", "input": "input", "ticket": "first"},
    {"kind": "clone", "source": "first", "ticket": "second"},
    {"kind": "clone", "source": "first", "ticket": "third"},
    {"kind": "cleanup", "ticket": "third"},
    {"kind": "cleanup", "ticket": "first"},
    {"kind": "clone", "source": "second", "ticket": "fourth"},
    {"kind": "borrow", "source": "fourth", "view": "borrowed"},
    {"kind": "cleanup", "ticket": "second"},
    {"kind": "copy", "view": "borrowed", "source": "fourth", "result": "observed"},
    {"kind": "cleanup", "ticket": "fourth"},
    {"kind": "return", "value": "observed"},
]

EXPECTED_SHADOW_DRIVER = r'''pub(crate) fn actual_public_shared_driver(input:Vec<u8>)->Vec<u8> {
    let (first,mut cursor)=original_shared_from_vec(input);
    let second=original_shared_clone(&first,cursor.borrow_mut());
    let third=original_shared_clone(&first,cursor.borrow_mut());
    let mut third_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(third,cursor.borrow_mut(),third_receipt.borrow_mut());
    proof_assert!(third_receipt.inner_logic() != None && !third_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut first_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(first,cursor.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic() != None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    let fourth=original_shared_clone(&second,cursor.borrow_mut());
    let borrowed=original_shared_as_slice(&fourth);
    let mut second_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(second,cursor.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    let observed=borrowed.to_vec();
    let mut fourth_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(fourth,cursor.borrow_mut(),fourth_receipt.borrow_mut());
    proof_assert!(fourth_receipt.inner_logic() != None && fourth_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0.len() == 0);
    observed
}'''

EXPECTED_CLONE_WRAPPER = r'''{
    let native = source.vtable.clone;
    let (table,spec) = shared_registration();
    proof_assert!(source.vtable == table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),
        ghost! {(&*source.original_shared,&mut **cursor)},spec)
}'''

EXPECTED_CLEANUP_WRAPPER = r'''{
    let native=value.vtable.drop;
    let (table,spec)=shared_drop_registration();
    proof_assert!(value.vtable == table);
    let proof=value.original_shared;
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(proof.into_inner(),&mut **cursor,&mut **output)},spec)
}'''

EXPECTED_CLONE_REGISTRATION = r'''{
    #[cfg(not(creusot))]
    { (original_shared_table_native(), Ghost::conjure()) }
    #[cfg(creusot)]
    { unreachable!("generic closed-table ghost-erasure reification") }
}'''

EXPECTED_DROP_REGISTRATION = r'''{
    #[cfg(not(creusot))]
    { (original_shared_table_native(), Ghost::conjure()) }
    #[cfg(creusot)]
    { unreachable!("generic closed-table consuming ghost-erasure reification") }
}'''

EXPECTED_NATIVE_HARNESS = r'''extern crate alloc;
use bytes::Bytes;
#[path = "<literal>"]
mod native_client;
fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len + 19);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        assert!(input.len() < input.capacity());
        assert_eq!(native_client::actual_public_shared_driver(input), expected);
    }
    println!("actual public Shared native client: 5 inputs passed");
}'''

# Complete From<Vec<u8>> body from the pinned source marker. Comparing the
# entire body binds the native cfg-selected len/cap branch, Shared allocation,
# count-one initializer, data pointer, and shared vtable together. The
# separately checked requires attributes remain Creusot-only.
EXPECTED_FROM_VEC_BODY = r'''{
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { return original_bytes_from_vec(vec); }
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { return original_shared_from_vec(vec); }
        #[cfg(not(all(creusot, any(bytes_original_shared_gate, bytes_original_constructor_gate))))]
        {
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let mut vec = ManuallyDrop::new(vec);
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let (ptr, len, cap) = (vec.as_mut_ptr(), vec.len(), vec.capacity());
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        let (ptr, len, cap, base, capabilities) = {
            let (raw, len, capabilities) = raw_vec::detach_vec(vec);
            let (base, cap) = raw.into_bound_ptr_at_zero();
            (base.as_ptr(), len, cap, base, capabilities)
        };

        if len == cap {
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            { unreachable!("selected spare-capacity Shared path"); }
            #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
            {
                let vec = ManuallyDrop::into_inner(vec);
                return Bytes::from(vec.into_boxed_slice());
            }
        }

        let shared = Box::new(Shared {
            buf: ptr,
            cap,
            ref_cnt: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_atomic_usize_new(1) }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { AtomicUsize::new(1) }
            },
        });

        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let shared = Box::into_raw(shared);
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        let (shared, shared_owner) = boxed_alignment::into_raw_aligned(shared);
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        boxed_alignment::aligned_address_has_clear_low_bit(
            crate::provenance_specs::pointer_addr(shared), core::mem::align_of::<Shared>()
        );
        debug_assert!(
            0 == (crate::provenance_specs::pointer_addr(shared) & KIND_MASK),
            "internal: Box<Shared> should have an aligned pointer",
        );
        Bytes {
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: Some(OriginalFrozenProof { base, capabilities, shared, shared_owner }),
            ptr,
            len,
            data: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_atomic_ptr_new(shared as _) }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { AtomicPtr::new(shared as _) }
            },
            vtable: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_shared_vtable() }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { &SHARED_VTABLE }
            },
        }
        }
    }'''

EXPECTED_NATIVE_MARKERS = {
    "bytes_cleanup": r'''#[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
pub fn cleanup(self) {
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
}''',
    "bytes_clone_impl": r'''impl Clone for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_bytes() == self.original_shared_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.ptr == self.ptr && result.len == self.len))]
    #[inline]
    fn clone(&self) -> Bytes {
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { original_shared_clone(self) }
        #[cfg(not(all(creusot, bytes_original_shared_gate)))]
        unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }
    }
}''',
    "bytes_as_slice": r'''#[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
#[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.original_frozen_valid()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result@ == self.original_frozen_bytes()))]
#[cfg_attr(all(creusot, bytes_original_constructor_gate), requires(self.original_bytes_valid()))]
#[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
#[inline]
fn as_slice(&self) -> &[u8] {
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
}''',
    "bytes_as_ref_impl": r'''impl AsRef<[u8]> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[inline]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}''',
    "shared_vtable": r'''static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};''',
    "shared_table_native": r'''#[allow(dead_code)]
fn original_shared_table_native() -> &'static Vtable {
    &SHARED_VTABLE
}''',
    "shared_clone": r'''unsafe fn shared_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Relaxed);
    shallow_clone_arc(shared as _, ptr, len)
}''',
    "shallow_clone_arc": r'''unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    crate::ref_count_ops::increment(&(*shared).ref_cnt);
    Bytes {
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        original_frozen: None,
        ptr,
        len,
        data: AtomicPtr::new(shared as _),
        vtable: &SHARED_VTABLE,
    }
}''',
    "shared_drop": r'''unsafe fn shared_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    data.with_mut(|shared| {
        release_shared(shared.cast());
    });
}''',
    "release_shared": r'''unsafe fn release_shared(ptr: *mut Shared) {
    if (*ptr).ref_cnt.fetch_sub(1, Ordering::Release) != 1 {
        return;
    }
    (*ptr).ref_cnt.load(Ordering::Acquire);
    free_shared(ptr);
}''',
    "free_shared": r'''const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize];
unsafe fn free_shared(ptr: *mut Shared) {
    let buf = (*ptr).buf;
    let cap = (*ptr).cap;
    dealloc(buf, Layout::from_size_align(cap, 1).unwrap());
    dealloc(ptr.cast(), Layout::new::<Shared>());
}''',
}

SHARED_MARKERS = (
    "bytes_cleanup", "bytes_clone_impl", "bytes_from_vec_impl", "bytes_as_slice",
    "bytes_as_ref_impl", "shared_vtable", "shared_table_native", "shared_clone",
    "shared_drop", "shallow_clone_arc", "release_shared", "free_shared",
)


class CorrespondenceError(Exception):
    pass


def require(ok: bool, message: str) -> None:
    if not ok:
        raise CorrespondenceError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def rust_tokens(text: str) -> list[str]:
    """A closed token reader: comments vanish and literals are opaque."""
    out: list[str] = []
    i, n = 0, len(text)
    multi = ("::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "+=", "-=", "*=", "/=")
    while i < n:
        if text[i].isspace():
            i += 1
            continue
        if text.startswith("//", i):
            end = text.find("\n", i)
            i = n if end < 0 else end + 1
            continue
        if text.startswith("/*", i):
            depth = 1
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            require(depth == 0, "unterminated Rust block comment")
            continue
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", text[i:])
        if raw:
            hashes = raw.group(1)
            i += len(raw.group(0))
            end = text.find('"' + hashes, i)
            require(end >= 0, "unterminated Rust raw string")
            i = end + len(hashes) + 1
            out.append("<literal>")
            continue
        if text.startswith(("b\"", "c\""), i):
            i += 1
        if text[i] == '"':
            i += 1
            escaped = False
            while i < n:
                char = text[i]
                i += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    break
            else:
                raise CorrespondenceError("unterminated Rust string literal")
            out.append("<literal>")
            continue
        # A character literal is opaque; a lifetime (`'a`, `'static`) remains
        # punctuation plus an identifier because it has no adjacent closing quote.
        if text[i] == "'":
            j = i + 1
            if j < n and text[j] == "\\":
                j += 2
                while j < n and text[j].isalnum():
                    j += 1
            else:
                j += 1
            if j < n and text[j] == "'":
                out.append("<literal>")
                i = j + 1
                continue
        ident = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i:])
        if ident:
            out.append(ident.group(0))
            i += len(ident.group(0))
            continue
        number = re.match(r"[0-9][A-Za-z0-9_]*", text[i:])
        if number:
            out.append(number.group(0))
            i += len(number.group(0))
            continue
        op = next((op for op in multi if text.startswith(op, i)), None)
        if op:
            out.append(op)
            i += len(op)
            continue
        out.append(text[i])
        i += 1
    return out


def mask_noncode(text: str) -> str:
    """Blank comments and quoted literals while preserving source offsets."""
    chars = list(text)
    i, n = 0, len(text)

    def blank(a: int, b: int) -> None:
        for k in range(a, b):
            if chars[k] != "\n":
                chars[k] = " "

    while i < n:
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", text[i:])
        if raw:
            h = raw.group(1)
            a = i
            i += len(raw.group(0))
            end = text.find('"' + h, i)
            require(end >= 0, "unterminated raw string while masking Rust")
            i = end + len(h) + 1
            blank(a, i)
            continue
        if text[i] in "\"" or text.startswith(("b\"", "c\""), i):
            a = i
            if text.startswith(("b\"", "c\""), i):
                i += 1
            i += 1
            escaped = False
            while i < n:
                ch = text[i]
                i += 1
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == '"':
                    break
            else:
                raise CorrespondenceError("unterminated string while masking Rust")
            blank(a, i)
            continue
        if text[i] == "'":
            a = i
            j = i + 1
            if j < n and text[j] == "\\":
                j += 2
                while j < n and text[j].isalnum():
                    j += 1
            else:
                j += 1
            if j < n and text[j] == "'":
                blank(a, j + 1)
                i = j + 1
                continue
        if text.startswith("//", i):
            a = i
            end = text.find("\n", i)
            i = n if end < 0 else end
            blank(a, i)
            continue
        if text.startswith("/*", i):
            a, depth = i, 1
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            require(depth == 0, "unterminated block comment while masking Rust")
            blank(a, i)
            continue
        i += 1
    return "".join(chars)


def find_fn_start(tokens: list[str], name: str) -> tuple[int, int]:
    found: list[tuple[int, int]] = []
    for start in range(len(tokens) - 1):
        if tokens[start:start + 2] != ["fn", name]:
            continue
        cursor = start + 2
        if cursor < len(tokens) and tokens[cursor] == "<":
            angle = 0
            while cursor < len(tokens):
                if tokens[cursor] == "<": angle += 1
                elif tokens[cursor] == ">":
                    angle -= 1
                    if angle == 0:
                        cursor += 1
                        break
                cursor += 1
            require(angle == 0, f"generic parameter list for `{name}` is unclosed")
        if cursor < len(tokens) and tokens[cursor] == "(":
            found.append((start, cursor))
    require(len(found) == 1, f"expected one `{name}` definition, found {len(found)}")
    return found[0]


def function_outer_attributes(source: str, name: str) -> list[list[str]]:
    """Return contiguous outer attributes immediately attached to a function.

    This deliberately handles only Rust's `#[...]` outer-attribute surface and
    a plain visibility prefix. The caller compares the result to a closed
    allowlist, so unsupported decoration fails closed instead of being parsed
    approximately.
    """
    tokens = rust_tokens(source)
    start, _ = find_fn_start(tokens, name)
    cursor = start

    # Skip an optional visibility (`pub` or `pub(...)`) between attributes and
    # the function item. Its contents remain part of the checked signature.
    if cursor and tokens[cursor - 1] == ")":
        depth = 0
        i = cursor - 1
        while i >= 0:
            if tokens[i] == ")":
                depth += 1
            elif tokens[i] == "(":
                depth -= 1
                if depth == 0:
                    break
            i -= 1
        if i > 0 and tokens[i - 1] == "pub":
            cursor = i - 1
    elif cursor and tokens[cursor - 1] == "pub":
        cursor -= 1

    attributes: list[list[str]] = []
    while cursor >= 2 and tokens[cursor - 1] == "]":
        depth = 0
        i = cursor - 1
        while i >= 0:
            if tokens[i] == "]":
                depth += 1
            elif tokens[i] == "[":
                depth -= 1
                if depth == 0:
                    break
            i -= 1
        require(i > 0 and tokens[i - 1] == "#",
                f"unsupported outer decoration immediately before `{name}`")
        attributes.insert(0, tokens[i - 1:cursor])
        cursor = i - 1
    return attributes


def find_function(source: str, name: str) -> tuple[list[str], int, int]:
    tokens = rust_tokens(source)
    start, args_open = find_fn_start(tokens, name)
    brace = next((i for i in range(args_open + 1, len(tokens)) if tokens[i] == "{"), None)
    require(brace is not None, f"function `{name}` has no body")
    depth = 0
    for i in range(brace, len(tokens)):
        if tokens[i] == "{":
            depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[brace + 1:i], start, i + 1
    raise CorrespondenceError(f"function `{name}` has an unclosed body")


def function_signature(source: str, name: str) -> list[str]:
    tokens = rust_tokens(source)
    start, args_open = find_fn_start(tokens, name)
    brace = next((i for i in range(args_open + 1, len(tokens)) if tokens[i] == "{"), None)
    require(brace is not None, f"function `{name}` has no body")
    return tokens[start:brace]


def body_from_braced_snippet(snippet: str) -> list[str]:
    tokens = rust_tokens(snippet)
    require(len(tokens) >= 2 and tokens[0] == "{" and tokens[-1] == "}",
            "expected a complete braced function-body snippet")
    return tokens[1:-1]


def check_function_surface(source: str, name: str, signature: str,
                           body: str | None = None) -> None:
    require(function_signature(source, name) == rust_tokens(signature),
            f"`{name}` signature differs from the admitted interface")
    if body is not None:
        actual, _, _ = find_function(source, name)
        expected_body = (find_function(body, name)[0]
                         if any(tok == "fn" for tok in rust_tokens(body))
                         else body_from_braced_snippet(body))
        require(actual == expected_body,
                f"`{name}` call surface differs from the admitted native/ghost wiring")


def audit_native_client(source: str) -> dict[str, Any]:
    actual, start, end = find_function(source, "actual_public_shared_driver")
    expected, _, _ = find_function(EXPECTED_NATIVE_CLIENT, "actual_public_shared_driver")
    require(rust_tokens(source) == rust_tokens(EXPECTED_NATIVE_CLIENT),
            "native client is outside the single reviewed straight-line witness")
    # Independently recount operations in executable code, rather than trusting
    # the JSON receipt or treating comments/string contents as events.
    methods = [actual[i + 1] for i in range(len(actual) - 1)
               if actual[i] == "." and actual[i + 1] in {"clone", "cleanup", "to_vec"}]
    require(methods == ["clone", "clone", "cleanup", "cleanup", "clone", "cleanup", "to_vec", "cleanup"],
            f"native client operation order changed: {methods}")
    require(actual == expected, "native client body differs from admitted event source")
    live = {"first"}
    issued = {"first"}
    trace: list[dict[str, str]] = [EXPECTED_EVENTS[0]]
    borrowed_view: tuple[str, str] | None = None
    peer_retired_while_borrowed = False
    for kind, source_name, target_name in (
        ("clone", "first", "second"), ("clone", "first", "third"),
        ("cleanup", "third", ""), ("cleanup", "first", ""),
        ("clone", "second", "fourth"), ("borrow", "fourth", "borrowed"),
        ("cleanup", "second", ""), ("copy", "borrowed", "observed"),
        ("cleanup", "fourth", ""),
    ):
        if kind == "clone":
            require(source_name in live and target_name not in issued,
                    f"clone source/output is not affine: {source_name} -> {target_name}")
            live.add(target_name)
            issued.add(target_name)
            trace.append({"kind": "clone", "source": source_name, "ticket": target_name})
        elif kind == "cleanup":
            require(source_name in live, f"cleanup does not consume a live handle: {source_name}")
            require(not (borrowed_view and borrowed_view[0] == source_name),
                    f"cleanup invalidates a still-live borrowed view: {source_name}")
            if borrowed_view and borrowed_view[0] != source_name:
                peer_retired_while_borrowed = True
            live.remove(source_name)
            trace.append({"kind": "cleanup", "ticket": source_name})
        elif kind == "borrow":
            require(source_name in live, f"read uses a dead handle: {source_name}")
            require(borrowed_view is None, "native client introduces more than one borrowed view")
            borrowed_view = (source_name, target_name)
            trace.append({"kind": "borrow", "source": source_name, "view": target_name})
        else:
            require(borrowed_view is not None and borrowed_view[1] == source_name,
                    f"copy uses an unknown borrowed view: {source_name}")
            owner = borrowed_view[0]
            require(owner in live, f"copy uses a borrowed view after its owner retired: {owner}")
            require(peer_retired_while_borrowed,
                    "borrowed-slice witness does not span a peer cleanup")
            trace.append({"kind": "copy", "view": source_name, "source": owner, "result": target_name})
            # `to_vec` ends the borrow before the final explicit cleanup.
            borrowed_view = None
    require(not live, f"native client returns with live Bytes handles: {sorted(live)}")
    require(borrowed_view is None, "native client returns with an unfinished borrowed view")
    trace.append({"kind": "return", "value": "observed"})
    require(trace == EXPECTED_EVENTS, "derived native operation trace differs from the admitted trace")
    return {
        "entrypoint": "actual_public_shared_driver",
        "source_sha256": sha(source.encode()),
        "function_token_span": [start, end],
        "events": trace,
        "live_bytes_handles_at_return": 0,
        "closed_language": True,
        "strings_and_comments_are_not_code": True,
    }


def extract_marked(source: str, name: str) -> str:
    begin = f"// ORIGINAL_SHARED_BEGIN {name}\n"
    end = f"// ORIGINAL_SHARED_END {name}\n"
    require(source.count(begin) == 1 and source.count(end) == 1,
            f"production marker for `{name}` is missing or duplicated")
    start = source.index(begin)
    finish = source.index(end, start) + len(end.rstrip("\n"))
    return source[start:finish]


def contains_once(tokens: list[str], needle: list[str], label: str) -> None:
    count = sum(tokens[i:i + len(needle)] == needle for i in range(len(tokens) - len(needle) + 1))
    require(count == 1, f"{label}: expected one occurrence of {' '.join(needle)}, found {count}")


def audit_production_chain(bytes_source: str, refcount_source: str,
                           shared_record_source: str | None = None) -> dict[str, Any]:
    if shared_record_source is None:
        shared_record_source = SHARED_RECORD_SOURCE.read_text()
    spans = {name: extract_marked(bytes_source, name) for name in SHARED_MARKERS}
    tok = {name: rust_tokens(span) for name, span in spans.items()}
    for name, expected in EXPECTED_NATIVE_MARKERS.items():
        require(tok[name] == rust_tokens(expected),
                f"production marker `{name}` differs from the exact reviewed call-chain source")

    # Exact native table target and proof getter identity.
    for field, target in (("clone", "shared_clone"), ("drop", "shared_drop")):
        contains_once(tok["shared_vtable"], [field, ":", target, ","], "SHARED_VTABLE")
    contains_once(tok["shared_table_native"], ["&", "SHARED_VTABLE"], "native table getter")

    # The actual trait source retains native Clone(&self), From<Vec>, AsRef and
    # explicit consuming cleanup. These are exact extracted production spans,
    # not source-map hashes or similarly named local replacements.
    clone_t = tok["bytes_clone_impl"]
    contains_once(clone_t, ["fn", "clone", "(", "&", "self", ")", "->", "Bytes"],
                  "Bytes Clone trait item")
    contains_once(clone_t, ["self", ".", "vtable", ".", "clone"], "Bytes Clone native dispatch")
    contains_once(clone_t, ["(", "self", ".", "vtable", ".", "clone", ")", "(",
                           "&", "self", ".", "data", ",", "self", ".", "ptr", ",",
                           "self", ".", "len", ")"], "Bytes Clone actual three-argument call")
    drop_t = tok["bytes_cleanup"]
    contains_once(drop_t, ["fn", "cleanup", "(", "self", ")"], "Bytes cleanup signature")
    contains_once(drop_t, ["this", ".", "vtable", ".", "drop"], "Bytes cleanup native dispatch")
    contains_once(drop_t, ["let", "mut", "this", "=", "ManuallyDrop", "::", "new", "(", "self", ")"],
                  "Bytes cleanup suppresses later automatic Drop")
    contains_once(drop_t, ["unsafe", "{", "callback", "(", "&", "mut", "this", ".", "data", ",",
                           "ptr", ",", "len", ")", "}"], "Bytes cleanup actual callback call")
    from_t = tok["bytes_from_vec_impl"]
    contains_once(from_t, ["fn", "from", "(", "vec", ":", "Vec", "<", "u8", ">", ")", "->", "Bytes"],
                  "From<Vec<u8>> trait item")
    contains_once(from_t, ["if", "len", "==", "cap"], "From<Vec> native capacity branch")
    contains_once(from_t, ["&", "SHARED_VTABLE"], "From<Vec> native shared table")
    from_body = function_body_after_marker(spans["bytes_from_vec_impl"], "from")
    require(from_body == body_from_braced_snippet(EXPECTED_FROM_VEC_BODY),
            "From<Vec> body differs from the exact selected constructor/capacity-branch source")
    # From<Vec> retains historical, explicit Creusot-only gate annotations in
    # the archived source. They are disabled in the native cfg and are not used
    # by this proof shadow. Reject an unconditional requires and record that
    # every retained trait-item requires is gated by `creusot`.
    gated_requires = 0
    for index, token in enumerate(from_t):
        if token != "requires":
            continue
        attr_start = max((j for j in range(index) if from_t[j] == "#"), default=-1)
        attr = from_t[attr_start:index]
        require("cfg_attr" in attr and "creusot" in attr,
                "From<Vec> has a non-Creusot/unconditional requires annotation")
        gated_requires += 1

    clone_cb = tok["shared_clone"]
    contains_once(clone_cb, ["data", ".", "load", "(", "Ordering", "::", "Relaxed", ")"],
                  "shared clone Relaxed load")
    contains_once(clone_cb, ["shallow_clone_arc", "(", "shared", "as", "_", ",", "ptr", ",", "len", ")"],
                  "shared clone helper target")
    contains_once(tok["shallow_clone_arc"], ["crate", "::", "ref_count_ops", "::", "increment", "(", "&", "(", "*", "shared", ")", ".", "ref_cnt", ")"],
                  "shallow clone actual guarded refcount helper")
    contains_once(tok["shared_vtable"], ["into_vec", ":", "shared_to_vec", ","], "SHARED_VTABLE::into_vec")
    contains_once(tok["shared_vtable"], ["into_mut", ":", "shared_to_mut", ","], "SHARED_VTABLE::into_mut")
    contains_once(tok["shared_vtable"], ["is_unique", ":", "shared_is_unique", ","], "SHARED_VTABLE::is_unique")
    as_ref_t = tok["bytes_as_ref_impl"]
    contains_once(as_ref_t, ["impl", "AsRef", "<", "[", "u8", "]", ">", "for", "Bytes"],
                  "Bytes AsRef implementation")
    contains_once(as_ref_t, ["self", ".", "as_slice", "(", ")"], "Bytes AsRef native source")
    contains_once(tok["bytes_as_slice"], ["slice", "::", "from_raw_parts", "(", "self", ".", "ptr", ",", "self", ".", "len", ")"],
                  "Bytes slice native projection")
    drop_cb = tok["shared_drop"]
    contains_once(drop_cb, ["release_shared", "(", "shared", ".", "cast", "(", ")", ")"],
                  "shared drop callback target")
    release_t = tok["release_shared"]
    contains_once(release_t, ["ref_cnt", ".", "fetch_sub", "(", "1", ",", "Ordering", "::", "Release", ")"],
                  "Release decrement")
    contains_once(release_t, ["ref_cnt", ".", "load", "(", "Ordering", "::", "Acquire", ")"],
                  "Acquire finalizer load")
    contains_once(release_t, ["free_shared", "(", "ptr", ")"], "shared finalizer target")
    # Preserve ordering, not only the existence of each hook.
    release_body = function_body_after_marker(spans["release_shared"], "release_shared")
    ordered = [["fetch_sub"], ["Ordering", "::", "Acquire"], ["free_shared"]]
    positions = [find_subsequence(release_body, part) for part in ordered]
    require(all(pos is not None for pos in positions) and positions == sorted(positions),
            "shared finalizer must Release-decrement, Acquire, then free in source order")
    free_body = function_body_after_marker(spans["free_shared"], "free_shared")
    dealloc_positions = [i for i in range(len(free_body) - 1) if free_body[i:i + 2] == ["dealloc", "("]]
    require(len(dealloc_positions) == 2 and dealloc_positions[0] < dealloc_positions[1],
            "free_shared must deallocate the payload then the Shared control allocation exactly once")
    contains_once(free_body, ["dealloc", "(", "buf", ",", "Layout", "::", "from_size_align", "(", "cap", ",", "1", ")", ".", "unwrap", "(", ")", ")"],
                  "production payload free")
    contains_once(free_body, ["dealloc", "(", "ptr", ".", "cast", "(", ")", ",", "Layout", "::", "new", "::", "<", "Shared", ">", "(", ")", ")"],
                  "production typed control free")
    require(free_body == rust_tokens(
        "{ let buf = (*ptr).buf; let cap = (*ptr).cap; "
        "dealloc(buf, Layout::from_size_align(cap, 1).unwrap()); "
        "dealloc(ptr.cast(), Layout::new::<Shared>()); }" )[1:-1],
        "free_shared must directly free payload and control without running Shared::drop or field drop glue")
    shared_record = rust_tokens(shared_record_source)
    require(shared_record == rust_tokens(
        "pub(crate) struct Shared { buf: *mut u8, cap: usize, "
        "pub(crate) ref_cnt: AtomicUsize, }"),
        "production Shared fields changed; explicit free field profile is no longer admitted")
    shared_drop_item = extract_struct_source(bytes_source, "impl Drop for Shared {",
                                             "production impl Drop for Shared")
    require(rust_tokens(shared_drop_item) == rust_tokens(
        "impl Drop for Shared { fn drop(&mut self) { unsafe { "
        "dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap()) } } }"),
        "production Shared destructor no longer matches the checked buffer-deallocation body")
    drop_guard = rust_tokens(
        "const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize];")
    contains_once(tok["free_shared"], drop_guard, "AtomicUsize no-drop compile-time guard")
    helper_t = rust_tokens(refcount_source)
    contains_once(helper_t, ["fetch_update", "(", "Ordering", "::", "Relaxed", ",", "Ordering", "::", "Relaxed"],
                  "native guarded refcount helper")
    contains_once(helper_t, ["crate", "::", "ref_count_limit", "::", "next_ref_count"],
                  "native checked refcount limit")
    try_increment_body, _, _ = find_function(refcount_source, "try_increment")
    require(try_increment_body == body_from_braced_snippet(r'''{
        counter.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            crate::ref_count_limit::next_ref_count,
        )
    }'''), "ref_count_ops::try_increment has an unexpected extra/change in atomic effect")
    increment_body, _, _ = find_function(refcount_source, "increment")
    require(increment_body == body_from_braced_snippet(r'''{
        if try_increment(counter).is_err() {
            crate::abort();
        }
    }'''), "ref_count_ops::increment has an unexpected extra/change in callback/effect")
    return {
        "production_source": "src/bytes.rs",
        "production_marker_spans": {name: sha(value.encode()) for name, value in spans.items()},
        "native_clone_target": "SHARED_VTABLE.clone -> shared_clone -> shallow_clone_arc -> ref_count_ops::increment",
        "native_cleanup_target": "SHARED_VTABLE.drop -> shared_drop -> release_shared -> free_shared",
        "native_release_order": ["fetch_sub(Release)", "load(Acquire)", "free_shared"],
        "payload_and_control_frees": 2,
        "production_shared_fields": ["buf: *mut u8", "cap: usize", "ref_cnt: AtomicUsize"],
        "shared_drop_body_matches_buffer_deallocation": True,
        "free_shared_directly_bypasses_shared_drop_and_field_glue": True,
        "atomic_needs_drop_guard": True,
        "public_trait_signatures_preserved": True,
        "native_from_unrestricted": True,
        "creusot_only_historical_from_requires": gated_requires,
        "exact_native_callchain_markers": sorted(EXPECTED_NATIVE_MARKERS),
        "exact_native_constructor_body": True,
        "exact_guarded_refcount_helper_bodies": ["try_increment", "increment"],
    }


def find_subsequence(tokens: list[str], needle: list[str]) -> int | None:
    for i in range(len(tokens) - len(needle) + 1):
        if tokens[i:i + len(needle)] == needle:
            return i
    return None


def function_body_after_marker(marked: str, name: str) -> list[str]:
    tokens = rust_tokens(marked)
    marker = ["fn", name, "("]
    positions = [i for i in range(len(tokens) - len(marker) + 1) if tokens[i:i + len(marker)] == marker]
    require(len(positions) == 1, f"expected one source function `{name}` in marker")
    open_at = next((i for i in range(positions[0] + len(marker), len(tokens)) if tokens[i] == "{"), None)
    require(open_at is not None, f"source function `{name}` body missing")
    depth = 0
    for i in range(open_at, len(tokens)):
        if tokens[i] == "{": depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                return tokens[open_at + 1:i]
    raise CorrespondenceError(f"source function `{name}` body unclosed")


def extract_struct_source(source: str, start_text: str, label: str) -> str:
    require(source.count(start_text) == 1, f"expected unique {label} source declaration")
    start = source.index(start_text)
    mask = mask_noncode(source)
    open_brace = start + len(start_text) - 1
    require(mask[open_brace] == "{", f"bad struct parser anchor for {label}")
    depth = 0
    for i in range(open_brace, len(mask)):
        if mask[i] == "{": depth += 1
        elif mask[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start:i + 1]
    raise CorrespondenceError(f"unclosed {label} source declaration")


def audit_generated_extractions(bytes_source: str, generated: pathlib.Path,
                                bytes_record_source: str, vtable_record_source: str,
                                bytes_mut_source: str) -> dict[str, Any]:
    """Compare extractor output token-for-token with independently found spans."""
    source_spans = {name: extract_marked(bytes_source, name) for name in SHARED_MARKERS}
    traits_path = generated / "public_traits.rs"
    native_path = generated / "native_bindings.rs"
    records_path = generated / "public_records.rs"
    require(records_path.is_file(), f"generated production record extraction missing: {records_path}")
    require(traits_path.is_file(), f"generated production trait extraction missing: {traits_path}")
    require(native_path.is_file(), f"generated native helper extraction missing: {native_path}")
    expected_traits = (source_spans["bytes_clone_impl"] + "\n" + source_spans["bytes_from_vec_impl"] + "\n"
                       + "impl Bytes {\n" + source_spans["bytes_cleanup"] + "\n"
                       + source_spans["bytes_as_slice"] + "\n}\n" + source_spans["bytes_as_ref_impl"] + "\n")
    native_names = ("shared_vtable", "shared_table_native", "shared_clone", "shallow_clone_arc",
                    "shared_drop", "release_shared", "free_shared")
    expected_native = "\n\n".join(source_spans[name] for name in native_names) + "\n"
    require(rust_tokens(traits_path.read_text()) == rust_tokens(expected_traits),
            "generated public_traits.rs tokens differ from actual production marker spans")
    require(rust_tokens(native_path.read_text()) == rust_tokens(expected_native),
            "generated native_bindings.rs tokens differ from actual production marker spans")
    shared_record = extract_struct_source(bytes_mut_source, "struct Shared {", "BytesMut::Shared")
    mutable_record = extract_struct_source(bytes_mut_source, "pub struct BytesMut {", "BytesMut")
    expected_records = (
        bytes_record_source + "\n" + vtable_record_source + "\n" +
        "mod mutable_record {\nuse alloc::vec::Vec;\n"
        "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n" +
        shared_record + "\n" + mutable_record + "\n}\nuse mutable_record::BytesMut;\n"
    )
    require(rust_tokens(records_path.read_text()) == rust_tokens(expected_records),
            "generated public_records.rs fields differ from actual production records")
    return {
        "public_traits_source_match": True,
        "native_bindings_source_match": True,
        "public_record_fields_source_match": True,
        "generated_dir": generated.as_posix(),
    }


def audit_native_harness(source: str, manifest_source: str, harness_path: pathlib.Path,
                         client_path: pathlib.Path, production_crate_root: pathlib.Path) -> dict[str, Any]:
    """Bind the externally executed source witness to this client and crate."""
    require(rust_tokens(source) == rust_tokens(EXPECTED_NATIVE_HARNESS),
            "native harness differs from the fixed five-input output-checking driver")
    client_relpath = rust_path_for_module(source, "native_client")
    actual_client = (harness_path.parent / client_relpath).resolve()
    require(actual_client == client_path.resolve(),
            "native harness `native_client` path does not resolve to the checked client source")
    try:
        manifest = tomllib.loads(manifest_source)
        production_manifest = tomllib.loads((production_crate_root / "Cargo.toml").read_text())
    except (tomllib.TOMLDecodeError, OSError) as exc:
        raise CorrespondenceError(f"native harness manifest cannot be parsed: {exc}") from exc
    package = manifest.get("package", {})
    dependency = manifest.get("dependencies", {}).get("bytes", {})
    require(package.get("name") == "bytes-shared-native-client",
            "native harness package identity differs from the reviewed external runner")
    require(isinstance(dependency, dict) and "path" in dependency,
            "native harness must depend on the selected local bytes crate by path")
    dependency_path = (harness_path.parent / dependency["path"]).resolve()
    require(dependency_path == production_crate_root.resolve(),
            "native harness bytes dependency does not resolve to the selected production crate")
    require(production_manifest.get("package", {}).get("version") == "1.11.1",
            "native harness production crate version is not bytes 1.11.1")
    return {
        "source_sha256": sha(source.encode()),
        "manifest_sha256": sha(manifest_source.encode()),
        "client_source_path_matches": True,
        "production_dependency_path_matches": True,
        "production_crate_version": "1.11.1",
        "tested_lengths": [0, 1, 7, 63, 1024],
        "input_capacity_slack": 19,
        "checks_client_output_equals_input": True,
        "does_not_verify_runtime_output_by_itself": True,
    }


def require_once(tokens: list[str], source: str, label: str) -> None:
    contains_once(tokens, rust_tokens(source), label)


def rust_path_for_module(source: str, module: str) -> str:
    code = mask_noncode(source)
    pattern = re.compile(r"#\s*\[\s*path\s*=\s*\s*\]\s*mod\s+" +
                         re.escape(module) + r"\s*;")
    matches = list(pattern.finditer(code))
    require(len(matches) == 1, f"expected one path override for module `{module}`")
    start, end = matches[0].span()
    i = start
    while i < end:
        if source.startswith("//", i):
            newline = source.find("\n", i)
            i = end if newline < 0 else min(newline + 1, end)
            continue
        if source.startswith("/*", i):
            depth = 1
            i += 2
            while i < end and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            require(depth == 0, f"unclosed comment in `{module}` path attribute")
            continue
        if source[i] == '"':
            i += 1
            value: list[str] = []
            while i < end and source[i] != '"':
                if source[i] == "\\":
                    require(i + 1 < end, f"bad string escape in `{module}` path")
                    value.append(source[i + 1])
                    i += 2
                else:
                    value.append(source[i])
                    i += 1
            require(i < end, f"unclosed string in `{module}` path attribute")
            return "".join(value)
        i += 1
    raise CorrespondenceError(f"module `{module}` path attribute contains no Rust string")


def audit_module_wiring(lib_source: str, shadow_source: str) -> dict[str, Any]:
    """Check only the selected proof module and the support modules it names."""
    lib = rust_tokens(lib_source)
    shadow = rust_tokens(shadow_source)
    for item in (
        "mod event;", "mod field_event;", "mod physical_projection;",
        "mod free_effect;", "mod erased_call;",
        "#[cfg(creusot)] mod public_shared;",
    ):
        require_once(lib, item, f"probe lib module wiring `{item}`")
    require(lib.count("public_shared") == 1,
            "proof facade has an unexpected duplicate or alternate module name")
    require("native_client" not in lib,
            "closed native client must remain the externally executed source witness")
    expected_paths = {
        "relaxed": "../../original-public-shared-gate-2026-10-08/src/relaxed.rs",
        "ref_count_limit": "../../../../src/ref_count_limit.rs",
        "fraction_map": "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs",
        "release": "../../shared-physical-lifecycle-2026-10-08/src/release.rs",
        "lifecycle": "../../scoped-issuance-cursor-2026-10-09/src/lifecycle.rs",
        "owned_region": "../../../../src/ownership_proof/owned_region.rs",
        "raw_vec": "../../../../src/ownership_proof/raw_vec.rs",
        "boxed_alignment": "../../../../src/ownership_proof/boxed_alignment.rs",
        "pointer_event": "../../original-shared-lifecycle-2026-10-08/src/pointer_event.rs",
        "native_ref_count_ops": "../../../../src/ref_count_ops.rs",
    }
    for module, path in expected_paths.items():
        require(rust_path_for_module(lib_source, module) == path,
                f"module `{module}` no longer names its reviewed source path")
    require_once(shadow, 'include!(concat!(env!("OUT_DIR"),"/public_records.rs"));',
                 "generated source record inclusion")
    require_once(shadow, 'include!("../../../../src/bytes/shared_record.rs");',
                 "production Shared record source inclusion")
    require_once(shadow, "use crate::{event::ScopedProtocol,free_effect};",
                 "scoped event and typed-free module identity")
    return {
        "proof_module": "src/public_shared.rs",
        "cfg": "creusot only",
        "native_client_compiled_in_probe": False,
        "native_client_execution": "separate harness source checked when supplied",
        "generated_record_source": "OUT_DIR/public_records.rs",
        "generic_module_identity_tokens": True,
    }


def audit_shadow_source(source: str, lib_source: str) -> dict[str, Any]:
    """Check the closed driver and exact interface/call wiring, not proof laws.

    The source checker checks selected event and native/ghost argument routing.
    It does not validate helper contracts as mathematical facts, prove the
    helper bodies, or establish `invoke3`/native-MIR erasure correctness.
    """
    tokens = rust_tokens(source)
    expected_driver_attributes = [
        rust_tokens("#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]"),
        rust_tokens("#[ensures(result@ == input@)]"),
    ]
    require(function_outer_attributes(source, "actual_public_shared_driver") ==
            expected_driver_attributes,
            "proof driver attributes must be exactly the selected len<capacity precondition and contents summary")

    # Two trusted registration reifications are an explicit proof/codegen TCB.
    # No other helper or driver may be trusted, and proof escape hatches or
    # aliases must not be smuggled in through cfg_attr or alternate spellings.
    trusted = [i for i, token in enumerate(tokens) if token == "trusted"]
    require(len(trusted) == 2,
            "only the two reviewed native/ghost registration reifications may be trusted")
    trusted_attr = rust_tokens("#[trusted]")
    for name in ("shared_registration", "shared_drop_registration"):
        attrs = function_outer_attributes(source, name)
        require(attrs.count(trusted_attr) == 1,
                f"`{name}` must be the sole reviewed trusted registration reification")
    for forbidden in ("check", "checktrusted", "assume", "axiom", "extern_spec", "externspec"):
        require(forbidden not in tokens,
                f"proof escape or alternate trusted spelling `{forbidden}` is forbidden")

    # Literal normalized token patterns bind the selected proof helper bodies
    # to the reviewed source, rather than only checking that anchor calls occur
    # somewhere inside a body. Comments and literal contents are masked, while
    # executable syntax and call order are exact.
    expected_body_fixture = json.loads(EXPECTED_SHADOW_BODIES_PATH.read_text())
    expected_bodies = expected_body_fixture.get("functions", {})
    required_body_names = {
        "original_shared_from_vec", "original_shared_as_slice",
        "shared_clone_checked", "shared_drop_checked", "free_recovered",
    }
    require(set(expected_bodies) == required_body_names,
            "reviewed shadow-body fixture has an unexpected function set")
    for name, token_text in expected_bodies.items():
        actual_body, _, _ = find_function(source, name)
        require(actual_body == token_text.split(),
                f"proof helper `{name}` differs from the reviewed complete token body")

    aliases = (
        "type Cursor = crate::event::ScopeCursor<lifecycle::State<Payload>>;",
        "type CloneInput<'a> = (&'a OriginalSharedProof, &'a mut Cursor);",
        "type CloneSpec<'a> = fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<CloneInput<'a>>)->Bytes;",
        "type DropInput<'a> = (OriginalSharedProof, &'a mut Cursor, &'a mut Option<Completion>);",
        "type DropSpec<'a> = fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<DropInput<'a>>);",
    )
    for alias in aliases:
        require_once(tokens, alias, f"scoped interface `{alias.split('=')[0].strip()}`")

    enum_at = [i for i in range(len(tokens) - 2)
               if tokens[i:i + 3] == ["enum", "Completion", "{"]]
    require(len(enum_at) == 1, "expected one closed Completion enum")
    brace = enum_at[0] + 2
    depth = 0
    enum_body: list[str] | None = None
    for i in range(brace, len(tokens)):
        if tokens[i] == "{": depth += 1
        elif tokens[i] == "}":
            depth -= 1
            if depth == 0:
                enum_body = tokens[brace + 1:i]
                break
    require(enum_body is not None, "Completion enum body is unclosed")
    require(enum_body == rust_tokens(
        "{ KeptAlive, Reclaimed(physical_projection::FreeReceipt, "
        "free_effect::TypedFreeReceipt<Shared>), }" )[1:-1],
        "Completion must be exactly KeptAlive or the paired physical/typed receipt")

    signatures = {
        "actual_public_shared_driver": "fn actual_public_shared_driver(input:Vec<u8>)->Vec<u8>",
        "original_shared_from_vec": "fn original_shared_from_vec(input:Vec<u8>)->(Bytes,Ghost<Cursor>)",
        "original_shared_clone": "fn original_shared_clone(source:&Bytes,mut cursor:Ghost<&mut Cursor>)->Bytes",
        "original_shared_cleanup": "fn original_shared_cleanup(mut value:Bytes,mut cursor:Ghost<&mut Cursor>,mut output:Ghost<&mut Option<Completion>>) ",
        "original_shared_as_slice": "fn original_shared_as_slice(value:&Bytes)->&[u8]",
        "shared_clone_checked": "fn shared_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes",
        "shared_drop_checked": "fn shared_drop_checked(data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DropInput>)",
        "free_recovered": "fn free_recovered(pointer:*mut Shared,recovered:Ghost<(Payload,LifetimeToken)>)->Ghost<(physical_projection::FreeReceipt,free_effect::TypedFreeReceipt<Shared>)>",
        "shared_registration": "fn shared_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>)",
        "shared_drop_registration": "fn shared_drop_registration<'a>()->(&'static Vtable,Ghost<DropSpec<'a>>)",
    }
    for name, signature in signatures.items():
        check_function_surface(source, name, signature)
    check_function_surface(source, "actual_public_shared_driver",
                           signatures["actual_public_shared_driver"], EXPECTED_SHADOW_DRIVER)
    check_function_surface(source, "original_shared_clone",
                           signatures["original_shared_clone"], EXPECTED_CLONE_WRAPPER)
    check_function_surface(source, "original_shared_cleanup",
                           signatures["original_shared_cleanup"], EXPECTED_CLEANUP_WRAPPER)
    check_function_surface(source, "shared_registration",
                           signatures["shared_registration"], EXPECTED_CLONE_REGISTRATION)
    check_function_surface(source, "shared_drop_registration",
                           signatures["shared_drop_registration"], EXPECTED_DROP_REGISTRATION)

    # Structural helper checks retain exact operation identity and cursor
    # routing. They do not establish that the trusted generic event/free
    # contracts or the body proofs imply the desired Rust semantics.
    clone_body, _, _ = find_function(source, "shared_clone_checked")
    require_once(clone_body, "pointer_event::load_relaxed(data,ghost! {&*source.data_binding})",
                 "clone atomic pointer load")
    require_once(clone_body,
        "field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(shared_word.cast::<Shared>(),"
        "ghost! {(*source.control).to_ref()},ghost! {&source.ticket.token},"
        "ghost! {(*source.invariant).to_ref()},cursor,",
        "clone event uses the shared affine cursor")
    require_once(clone_body, "lifecycle::State::on_register(", "clone registration callback")

    drop_body, _, _ = find_function(source, "shared_drop_checked")
    require_once(drop_body,
        "field_event::decrement_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,ghost! {&mut **cursor},",
        "cleanup event uses the same affine cursor")
    require_once(drop_body, "if old == 1", "cleanup last-owner branch")
    require_once(drop_body,
        "field_event::acquire_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,ghost! {&mut **cursor},",
        "finalizer Acquire uses the shared affine cursor")
    require_once(drop_body, "lifecycle::State::on_release(", "cleanup Release callback")
    require_once(drop_body, "lifecycle::State::on_acquire(", "cleanup Acquire callback")
    require_once(drop_body, "lifecycle::Pending::recover(", "finalizer recovery")
    require_once(drop_body, "free_recovered(shared,recovered)", "last-owner free target")
    order = [
        find_subsequence(drop_body, rust_tokens("field_event::decrement_owned")),
        find_subsequence(drop_body, rust_tokens("field_event::acquire_owned")),
        find_subsequence(drop_body, rust_tokens("lifecycle::Pending::recover")),
        find_subsequence(drop_body, rust_tokens("free_recovered")),
    ]
    require(all(position is not None for position in order) and order == sorted(order),
            "cleanup call structure must decrement, Acquire, recover, then free")

    free_body, _, _ = find_function(source, "free_recovered")
    require_once(free_body,
        "physical_projection::deallocate(shared.buf,shared.cap,bound,capabilities)",
        "payload free route")
    require_once(free_body,
        "free_effect::deallocate_typed_box(pointer,owner)",
        "typed Shared control free route")
    payload_free = find_subsequence(free_body, rust_tokens("physical_projection::deallocate"))
    control_free = find_subsequence(free_body, rust_tokens("free_effect::deallocate_typed_box"))
    require(payload_free is not None and control_free is not None and payload_free < control_free,
            "free receipt source structure must return payload then typed control effects")

    module = audit_module_wiring(lib_source, source)
    return {
        "proof_shadow_source_sha256": sha(source.encode()),
        "proof_shadow_entrypoint": "actual_public_shared_driver",
        "driver_body_matches_closed_shadow_witness": True,
        "native_event_trace_matches": EXPECTED_EVENTS,
        "cursor_interface": "one ScopeCursor<State<Payload>> threaded through all Clone/cleanup events",
        "cleanup_receipt_interface": "normal return writes KeptAlive or (physical,typed-control) Reclaimed",
        "selected_wiring_checks": [
            "vtable clone/drop field to extracted production getter registration",
            "actual three native args plus Ghost-only clone/cleanup input at invoke3",
            "cursor passed to registration/decrement/Acquire calls",
            "Release -> conditional Acquire -> recovery -> paired free routes",
            "complete reviewed token bodies for selected constructor, slice, clone, cleanup, and free helpers",
        ],
        "module_wiring": module,
        "not_checked": [
            "mathematical truth of facade attributes/contracts",
            "generic event, scope cursor, FreeReceipt, and invoke3 TCB adequacy",
            "whether the helper VCs establish their stated contracts",
            "native interpretation of helper Ghost arguments and erased invoke3 ABI",
            "native MIR/codegen proof for Ghost erasure or callback correspondence",
            "arbitrary clients/concurrency, automatic Drop, or full Bytes admission",
        ],
    }


def audit_all(*, driver_source: str, shadow_source: str, lib_source: str, bytes_source: str,
              refcount_source: str, shared_record_source: str,
              bytes_record_source: str, vtable_record_source: str,
              bytes_mut_source: str, generated_dir: pathlib.Path | None = None,
              native_harness_source: str | None = None,
              native_manifest_source: str | None = None,
              native_harness_path: pathlib.Path | None = None,
              native_manifest_path: pathlib.Path | None = None) -> dict[str, Any]:
    client = audit_native_client(driver_source)
    production = audit_production_chain(bytes_source, refcount_source, shared_record_source)
    shadow = audit_shadow_source(shadow_source, lib_source)
    generated = (audit_generated_extractions(bytes_source, generated_dir, bytes_record_source,
                                            vtable_record_source, bytes_mut_source)
                 if generated_dir else None)
    require((native_harness_source is None) == (native_manifest_source is None),
            "native harness source and manifest must be supplied together")
    native_execution = None
    if native_harness_source is not None:
        require(native_harness_path is not None and native_manifest_path is not None,
                "native harness file paths are required with their source text")
        native_execution = audit_native_harness(
            native_harness_source, native_manifest_source, native_harness_path, DRIVER, CRATE_ROOT)
    return {
        "status": "correspondence_pass",
        "native_client": client,
        "production_native_chain": production,
        "proof_shadow": shadow,
        "generated_extractions": generated,
        "native_execution_source": native_execution,
        "semantic_scope": "one closed actual Shared len<capacity client only; source correspondence does not prove generic scoped-field/cursor adequacy, native MIR semantics, arbitrary concurrent history, automatic Drop, or full Bytes admission",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", type=pathlib.Path, default=DRIVER)
    parser.add_argument("--shadow", type=pathlib.Path, default=SHADOW)
    parser.add_argument("--lib-source", type=pathlib.Path, default=LIB_SOURCE)
    parser.add_argument("--bytes-source", type=pathlib.Path, default=BYTES_SOURCE)
    parser.add_argument("--refcount-source", type=pathlib.Path, default=REFCOUNT_SOURCE)
    parser.add_argument("--bytes-record-source", type=pathlib.Path, default=CRATE_ROOT / "src/bytes/bytes_record.rs")
    parser.add_argument("--shared-record-source", type=pathlib.Path, default=SHARED_RECORD_SOURCE)
    parser.add_argument("--vtable-record-source", type=pathlib.Path, default=CRATE_ROOT / "src/bytes/vtable_record.rs")
    parser.add_argument("--bytes-mut-source", type=pathlib.Path, default=CRATE_ROOT / "src/bytes_mut.rs")
    parser.add_argument("--generated", type=pathlib.Path,
                        default=pathlib.Path(os.environ.get("OUT_DIR", GENERATED)))
    parser.add_argument("--native-harness", type=pathlib.Path)
    parser.add_argument("--native-manifest", type=pathlib.Path)
    parser.add_argument("--skip-generated", action="store_true")
    args = parser.parse_args()
    try:
        result = audit_all(
            driver_source=args.client.read_text(),
            shadow_source=args.shadow.read_text(),
            lib_source=args.lib_source.read_text(),
            bytes_source=args.bytes_source.read_text(),
            refcount_source=args.refcount_source.read_text(),
            shared_record_source=args.shared_record_source.read_text(),
            bytes_record_source=args.bytes_record_source.read_text(),
            vtable_record_source=args.vtable_record_source.read_text(),
            bytes_mut_source=args.bytes_mut_source.read_text(),
            generated_dir=None if args.skip_generated else args.generated,
            native_harness_source=(args.native_harness.read_text() if args.native_harness else None),
            native_manifest_source=(args.native_manifest.read_text() if args.native_manifest else None),
            native_harness_path=args.native_harness,
            native_manifest_path=args.native_manifest,
        )
    except (OSError, CorrespondenceError) as exc:
        print(json.dumps({"status": "coverage_failure", "error": str(exc)}, indent=2))
        return 2
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
