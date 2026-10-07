#!/usr/bin/env python3
from hashlib import sha256
from pathlib import Path
from textwrap import dedent

probe = Path(__file__).resolve().parent
crate = probe.parents[3]
production = (crate / "src/bytes.rs").read_text()
source = (probe / "src/lib.rs").read_text()


def item(text: str, declaration: str) -> str:
    start = text.index(declaration)
    opening = text.index("{", start)
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "{":
            depth += 1
        elif text[pos] == "}":
            depth -= 1
            if depth == 0:
                return text[start:pos + 1]
    raise ValueError(f"unclosed item: {declaration}")


def branch_fragment(text: str, allocation_marker: str, vtable_marker: str, semicolon: bool) -> str:
    marker = text.index(allocation_marker)
    start = text.rfind("\n", 0, marker) + 1
    record = text.index("Bytes {", marker)
    opening = text.index("{", record)
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "{":
            depth += 1
        elif text[pos] == "}":
            depth -= 1
            if depth == 0:
                end = pos + 1
                if semicolon and text[end:end + 1] == ";":
                    end += 1
                if vtable_marker not in text[record:end]:
                    raise ValueError("branch record omitted requested vtable field")
                return text[start:end]
    raise ValueError("unclosed Bytes record")


prod_branch = branch_fragment(
    production, "let shared = Box::new(Shared {", "vtable: &SHARED_VTABLE", False
)
probe_branch = branch_fragment(
    source, "let boxed = Box::new(Shared {", "vtable: shared_vtable()", True
)
expected_probe_branch = dedent(prod_branch)
for before, after in (
    ("let shared = Box::new(Shared {", "let boxed = Box::new(Shared {"),
    ("ref_cnt: AtomicUsize::new(1)", "ref_cnt: atomic_usize_new(1)"),
    ("let shared = Box::into_raw(shared);", "let (shared, shared_owner) = boxed_alignment::into_raw_aligned(boxed);"),
    ("shared as usize & KIND_MASK", "crate::provenance_specs::pointer_addr(shared) & KIND_MASK"),
    (
        "// The pointer should be aligned, so this assert should\n// always succeed.\ndebug_assert!(\n    0 == (crate::provenance_specs::pointer_addr(shared) & KIND_MASK),",
        "// The pointer should be aligned, so this assert should\n// always succeed.\nlet shared_addr = crate::provenance_specs::pointer_addr(shared);\nboxed_alignment::aligned_address_has_clear_low_bit(\n    shared_addr,\n    core::mem::align_of::<Shared>(),\n);\ndebug_assert!(\n    0 == (shared_addr & KIND_MASK),",
    ),
    ("\nBytes {", "\nlet bytes = Bytes {"),
    ("data: AtomicPtr::new(shared as _)", "data: atomic_ptr_new(shared as _)"),
    ("vtable: &SHARED_VTABLE", "vtable: shared_vtable()"),
):
    expected_probe_branch = expected_probe_branch.replace(before, after)
expected_probe_branch += ";"

table_prod = item(production, "static SHARED_VTABLE: Vtable =")
table_probe = item(source, "static SHARED_VTABLE: Vtable =")
prod_shared = item(production, "struct Shared {")
probe_shared = item(source, "struct Shared {")
prod_vtable = item(production, "pub(crate) struct Vtable {")
probe_vtable = item(source, "pub(crate) struct Vtable {")
prod_bytes = item(production, "pub struct Bytes {")
probe_bytes = item(source, "pub struct Bytes {")

expected_atomic_wrappers = """#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn atomic_usize_new(value: usize) -> AtomicUsize {
    unimplemented!()
}

#[cfg(not(creusot))]
fn atomic_usize_new(value: usize) -> AtomicUsize {
    AtomicUsize::new(value)
}

#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn atomic_ptr_new<T>(value: *mut T) -> AtomicPtr<T> {
    unimplemented!()
}

#[cfg(not(creusot))]
fn atomic_ptr_new<T>(value: *mut T) -> AtomicPtr<T> {
    AtomicPtr::new(value)
}"""

checks = {
    "probe Bytes fields equal production layout": probe_bytes == prod_bytes,
    "probe Vtable fields equal production layout": probe_vtable == prod_vtable,
    "probe Shared fields equal production layout": probe_shared == prod_shared,
    "native cfg table initializer equals production initializer": table_probe == table_prod,
    "current production branch matches authorized helper substitutions": dedent(probe_branch) == expected_probe_branch,
    "proof/native atomic bridges have only true/true and exact native constructors": expected_atomic_wrappers in source,
    "production pointer_addr helper is included unchanged": (probe / "sources/provenance_specs.rs").read_text() == (crate / "src/provenance_specs.rs").read_text(),
    "production boxed_alignment helper is included unchanged": (probe / "sources/boxed_alignment.rs").read_text() == (crate / "src/ownership_proof/boxed_alignment.rs").read_text(),
    "pinned why3find config matches its saved snapshot": (probe / "sources/why3find.json").read_text() == (probe / "why3find.json").read_text(),
    "canonical probe source snapshot matches current source": (probe / "sources/lib.rs").read_text() == source,
    "production bytes source snapshot matches current source": (probe / "sources/production-bytes.rs").read_text() == production,
}
for label, ok in checks.items():
    print(f"{'PASS' if ok else 'FAIL'}: {label}")
for name, content in (
    ("src/bytes.rs", production),
    ("src/provenance_specs.rs", (crate / "src/provenance_specs.rs").read_text()),
    ("src/ownership_proof/boxed_alignment.rs", (crate / "src/ownership_proof/boxed_alignment.rs").read_text()),
    ("probe/src/lib.rs", source),
    ("production-shared-branch.rs", prod_branch),
    ("production-Shared.rs", prod_shared),
    ("production-Vtable.rs", prod_vtable),
    ("production-SHARED_VTABLE.rs", table_prod),
):
    print(f"sha256 {name}: {sha256(content.encode()).hexdigest()}")
if not all(checks.values()):
    if not checks["current production branch matches authorized helper substitutions"]:
        print(f"expected branch: {expected_probe_branch!r}")
        print(f"actual branch:   {dedent(probe_branch)!r}")
    raise SystemExit(1)
