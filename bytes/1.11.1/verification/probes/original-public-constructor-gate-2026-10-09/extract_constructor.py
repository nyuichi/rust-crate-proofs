#!/usr/bin/env python3
"""Extract the actual source constructors and freeze their native routing."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import os

probe = pathlib.Path(__file__).resolve().parent
root = pathlib.Path(__file__).resolve().parents[3]
out = pathlib.Path(os.environ["OUT_DIR"])
generated = probe / "generated"
generated.mkdir(exist_ok=True)

bytes_rs = (root / "src/bytes.rs").read_text()
rows: dict[str, object] = {}


def span(source: str, begin: str, end: str, name: str) -> str:
    bmark = f"// {begin} {name}\n"
    emark = f"// {end} {name}"
    assert source.count(bmark) == 1, ("duplicate/missing begin", name)
    assert source.count(emark) == 1, ("duplicate/missing end", name)
    a = source.index(bmark)
    b = source.index(emark, a) + len(emark)
    text = source[a:b]
    rows[name] = {
        "sha256": hashlib.sha256(text.encode()).hexdigest(),
        "line": source[:a].count("\n") + 1,
    }
    return text


def item_after(source: str, needle: str, label: str) -> str:
    """Return one braced Rust item starting at needle, with balanced braces."""
    start = source.index(needle)
    open_at = source.index("{", start)
    depth = 0
    for i in range(open_at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                text = source[start : i + 1]
                rows[label] = {
                    "sha256": hashlib.sha256(text.encode()).hexdigest(),
                    "line": source[:start].count("\n") + 1,
                }
                return text
    raise AssertionError(f"unterminated source item: {label}")


vec_impl = span(bytes_rs, "ORIGINAL_SHARED_BEGIN", "ORIGINAL_SHARED_END", "bytes_from_vec_impl")
box_impl = span(bytes_rs, "ORIGINAL_CONSTRUCTOR_BEGIN", "ORIGINAL_CONSTRUCTOR_END", "bytes_from_box_impl")
new_method = span(bytes_rs, "ORIGINAL_CONSTRUCTOR_BEGIN", "ORIGINAL_CONSTRUCTOR_END", "bytes_new")
static_method = span(bytes_rs, "ORIGINAL_CONSTRUCTOR_BEGIN", "ORIGINAL_CONSTRUCTOR_END", "bytes_from_static")
slice_method = span(bytes_rs, "ORIGINAL_SHARED_BEGIN", "ORIGINAL_SHARED_END", "bytes_as_slice")
as_ref_impl = span(bytes_rs, "ORIGINAL_SHARED_BEGIN", "ORIGINAL_SHARED_END", "bytes_as_ref_impl")

# Preserve the exact original Vec branch: len < cap keeps the Shared allocation;
# len == cap turns the Vec into a boxed slice and enters the actual Box impl.
assert "return original_bytes_from_vec(vec);" in vec_impl
assert "bytes_original_constructor_gate), ensures(result.original_bytes_valid())" in vec_impl
assert "bytes_original_constructor_gate), ensures(result.original_bytes_content() == vec@)" in vec_impl
assert not re.search(r"bytes_original_constructor_gate\),\s*requires", vec_impl)
assert "Bytes::from(vec.into_boxed_slice())" in vec_impl
assert re.search(r"if\s+len\s*==\s*cap", vec_impl)
assert "original_bytes_from_box(slice)" in box_impl
assert "bytes_original_constructor_gate), ensures(result.original_bytes_valid())" in box_impl
assert "bytes_original_constructor_gate), ensures(result.original_bytes_content() == slice@)" in box_impl
assert not re.search(r"bytes_original_constructor_gate\),\s*requires", box_impl)
assert "if slice.is_empty()" in box_impl
assert "Box::into_raw(slice) as *mut u8" in box_impl
assert "if ptr as usize & 0x1 == 0" in box_impl
assert re.search(r"ptr_map\(ptr,\s*\|addr\|\s*addr\s*\|\s*KIND_VEC\)", box_impl)
assert "data: AtomicPtr::new(ptr.cast())" in box_impl
assert "vtable: &PROMOTABLE_EVEN_VTABLE" in box_impl
assert "vtable: &PROMOTABLE_ODD_VTABLE" in box_impl
assert "Bytes::from_static(EMPTY)" in new_method
assert "original_bytes_from_static(bytes)" in static_method
assert "original_bytes_as_slice(self)" in slice_method
assert "self.as_slice()" in as_ref_impl

# Record the actual public refinement surface as a group. Both From impls are
# included verbatim; neither has a constructor-gate precondition. The actual
# new/static bodies and AsRef route are included beside them.
rows["from_refinements"] = {
    "included_items": [
        "bytes_from_vec_impl",
        "bytes_from_box_impl",
        "bytes_new",
        "bytes_from_static",
        "bytes_as_slice",
        "bytes_as_ref_impl",
    ],
    "constructor_gate_preconditions": [],
    "ensures": [
        "From<Vec<u8>>: original_bytes_valid && original_bytes_content == vec@",
        "From<Box<[u8]>>: original_bytes_valid && original_bytes_content == slice@",
    ],
}

records = ""
for name in ["bytes_record.rs", "vtable_record.rs"]:
    path = root / "src/bytes" / name
    text = path.read_text()
    rows[name] = {"sha256": hashlib.sha256(text.encode()).hexdigest()}
    records += text + "\n"

mutable = (root / "src/bytes_mut.rs").read_text()


def mutable_record(start: str, name: str) -> str:
    a = mutable.index(start)
    b = mutable.index("\n}", a) + 2
    text = mutable[a:b]
    rows[name] = {
        "sha256": hashlib.sha256(text.encode()).hexdigest(),
        "line": mutable[:a].count("\n") + 1,
    }
    return text


records += (
    "mod mutable_record {\n"
    "use alloc::vec::Vec;\n"
    "use core::{ptr::NonNull,sync::atomic::AtomicUsize};\n"
    + mutable_record("struct Shared {", "bytes_mut::Shared")
    + "\n"
    + mutable_record("pub struct BytesMut {", "bytes_mut::BytesMut")
    + "\n}\nuse mutable_record::BytesMut;\n"
)

traits = (
    vec_impl
    + "\n"
    + box_impl
    + "\nimpl Bytes {\n"
    + new_method
    + "\n"
    + static_method
    + "\n"
    + slice_method
    + "\n}\n"
    + as_ref_impl
    + "\n"
)
(out / "public_records.rs").write_text(records)
(out / "public_traits.rs").write_text(traits)
(generated / "public_records.rs").write_text(records)
(generated / "public_traits.rs").write_text(traits)
for name, text in [("public_records.rs", records), ("public_traits.rs", traits)]:
    rows[f"generated/{name}"] = {"sha256": hashlib.sha256(text.encode()).hexdigest()}

# Deterministic extraction of the already reviewed Shared constructor component.
# Only the output vocabulary, proof-sum wrapper and closed-table getter name are
# substituted. Its recovery, protocol, typed field identity and physical
# obligations remain the selected Shared gate's body and contracts.
shared_source = (probe.parent / "original-public-shared-gate-2026-10-08/src/public_shared.rs").read_text()


def rust_item(source: str, start: str, label: str) -> str:
    return item_after(source, start, label)


payload_start = shared_source.index("pub(crate) struct Payload {")
payload_end = shared_source.index("\n\npub(crate) type CloneSpec", payload_start)
payload_component = shared_source[payload_start:payload_end]
proof_start = shared_source.index("pub(crate) struct OriginalSharedProof {")
proof_end = shared_source.index("\n\nimpl Bytes {", proof_start)
shared_proof = shared_source[proof_start:proof_end]
constructor_start = shared_source.index("fn original_shared_from_vec(input:Vec<u8>)->Bytes {")
constructor_body = rust_item(shared_source, "fn original_shared_from_vec(input:Vec<u8>)->Bytes", "shared_component/original_shared_from_vec")
contract_start = shared_source.rfind("\n#[requires(", 0, constructor_start)
assert contract_start >= 0
constructor = shared_source[contract_start + 1 : constructor_start] + constructor_body

assert "pub(crate) impl" not in payload_component
assert "pub(crate) fn original_shared_valid" not in shared_proof
assert "len@ <= p.capacity@" in shared_proof
assert "shared_registration()" in constructor
assert "original_shared:ghost!" in constructor

constructor = constructor.replace("fn original_shared_from_vec", "fn original_bytes_shared_from_vec")
constructor = constructor.replace("result.original_shared_valid()", "result.original_bytes_valid()")
constructor = constructor.replace("result.original_shared_bytes()", "result.original_bytes_content()")
constructor = constructor.replace("let (vtable,_spec) = shared_registration();", "let vtable = shared_table_reification();")
constructor = constructor.replace(
    "original_shared:ghost! {\n        OriginalSharedProof {",
    "original_bytes:ghost! {\n        OriginalBytesProof::Shared(OriginalSharedProof {",
)
constructor = constructor.replace(
    "data_binding}\n    }}",
    "data_binding})\n    }}",
)
assert "shared_registration()" not in constructor
assert "original_shared:" not in constructor
assert "OriginalBytesProof::Shared(OriginalSharedProof" in constructor
shared_component = payload_component + "\n\n" + shared_proof + "\n\n" + constructor + "\n"
(out / "shared_constructor_component.rs").write_text(shared_component)
(generated / "shared_constructor_component.rs").write_text(shared_component)
rows["generated/shared_constructor_component.rs"] = {
    "sha256": hashlib.sha256(shared_component.encode()).hexdigest(),
    "source": "existing original-public-shared-gate-2026-10-08/src/public_shared.rs",
    "substitutions": [
        "selected From<Vec> helper renamed and wrapped as OriginalBytesProof::Shared",
        "strong branch contract vocabulary changed to original_bytes_valid/content",
        "vtable registration call renamed; its identity contract is retained below",
    ],
}

# Exact source declarations supporting the closed table identities. These are
# audit evidence, not a claim that source hashes prove logical/native adequacy.
bindings = []
for name, needle in [
    ("shared_vtable", "static SHARED_VTABLE: Vtable = Vtable"),
    ("shared_table_native", "fn original_shared_table_native() -> &'static Vtable"),
    ("static_vtable", "const STATIC_VTABLE: Vtable = Vtable"),
    ("promotable_even_vtable", "static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable"),
    ("promotable_odd_vtable", "static PROMOTABLE_ODD_VTABLE: Vtable = Vtable"),
]:
    text = item_after(bytes_rs, needle, f"native/{name}")
    bindings.append(text)

assert "&SHARED_VTABLE" in bindings[1]
assert "clone: static_clone" in bindings[2]
assert "clone: promotable_even_clone" in bindings[3]
assert "clone: promotable_odd_clone" in bindings[4]
kind_match = re.search(r"const\s+KIND_VEC:\s*usize\s*=\s*(0b[01_]+|[0-9_]+)\s*;", bytes_rs)
assert kind_match and int(kind_match.group(1).replace("_", ""), 0) == 1
rows["native/kind_vec"] = {
    "sha256": hashlib.sha256(kind_match.group(0).encode()).hexdigest(),
    "line": bytes_rs[:kind_match.start()].count("\n") + 1,
    "value": 1,
}
(generated / "native_constructor_bindings.rs").write_text("\n\n".join(bindings) + "\n")
rows["closed_table_bindings"] = {
    "sha256": hashlib.sha256((generated / "native_constructor_bindings.rs").read_bytes()).hexdigest(),
    "status": "source-checked exact native table declarations; the restricted reification remains generic TCB",
    "tables": ["STATIC_VTABLE", "PROMOTABLE_EVEN_VTABLE", "PROMOTABLE_ODD_VTABLE", "SHARED_VTABLE"],
}

(generated / "source-map.json").write_text(json.dumps(rows, indent=2) + "\n")
