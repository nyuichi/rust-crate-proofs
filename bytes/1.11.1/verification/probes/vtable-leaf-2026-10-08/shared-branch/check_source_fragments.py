#!/usr/bin/env python3
from hashlib import sha256
from pathlib import Path
from textwrap import dedent

probe = Path(__file__).resolve().parent
crate = probe.parents[3]
production = (crate / "src/bytes.rs").read_text()
source = (probe / "src/lib.rs").read_text()


def shared_branch(text: str, vtable_expr: str, production_source: bool) -> str:
    if production_source:
        impl_start = text.index("impl From<Vec<u8>> for Bytes {")
    else:
        impl_start = text.index("pub fn from_vec_shared_branch")
    body_start = text.index("let shared = Box::new(Shared {", impl_start)
    start = text.rfind("\n", 0, body_start) + 1
    field = f"vtable: {vtable_expr},\n"
    field_start = text.index(field, body_start)
    record_end = text.index("}", field_start + len(field)) + 1
    return text[start:record_end]


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


def shared_struct(text: str, production_source: bool) -> str:
    if production_source:
        start = text.index("struct Shared {", text.index("impl From<Vec<u8>> for Bytes {"))
    else:
        start = text.index("struct Shared {")
    end = text.index("\n}", start) + 2
    return text[start:end]

prod_branch = shared_branch(production, "&SHARED_VTABLE", True)
prod_struct = shared_struct(production, True)
prod_vtable = item(production, "pub(crate) struct Vtable {")
prod_table = item(production, "static SHARED_VTABLE: Vtable =")
probe_branch = shared_branch(source, "shared_vtable()", False)
probe_struct = shared_struct(source, False)
probe_table = item(source, "static SHARED_VTABLE: Vtable =")
expected_probe_branch = prod_branch.replace(
    "(shared as usize & KIND_MASK)",
    "(crate::provenance_specs::pointer_addr(shared) & KIND_MASK)",
).replace(
    "            vtable: &SHARED_VTABLE,\n        }",
    "            vtable: shared_vtable(),\n        }",
)

checks = {
    "probe Bytes fields equal production Bytes layout": item(source, "pub struct Bytes {") == item(production, "pub struct Bytes {"),
    "probe Vtable equals production Vtable layout": item(source, "pub(crate) struct Vtable {") == prod_vtable,
    "native cfg table initializer equals production initializer": probe_table == prod_table,
    "included pointer_addr helper equals current production helper source": (probe / "sources/provenance_specs.rs").read_text() == (crate / "src/provenance_specs.rs").read_text(),
    "probe Shared shape equals current production Shared fields": probe_struct == prod_struct,
    "probe KIND_MASK equals production KIND_MASK": next(line for line in source.splitlines() if line.startswith("const KIND_MASK")) == next(line for line in production.splitlines() if line.startswith("const KIND_MASK")),
    "probe branch equals current production branch except address helper and vtable getter": dedent(probe_branch) == dedent(expected_probe_branch),
    "canonical probe source snapshot matches current source": (probe / "sources/lib.rs").read_text() == source,
    "saved production Shared source snapshot is current": (probe / "sources/production-shared-struct.rs").read_text() == prod_struct,
    "saved production branch source snapshot is current": (probe / "sources/production-shared-branch.rs").read_text() == prod_branch,
}
for label, ok in checks.items():
    print(f"{'PASS' if ok else 'FAIL'}: {label}")
for name, content in (
    ("src/bytes.rs", production),
    ("src/provenance_specs.rs", (crate / "src/provenance_specs.rs").read_text()),
    ("probe/src/lib.rs", source),
    ("production-shared-branch.rs", prod_branch),
    ("production-shared-struct.rs", prod_struct),
    ("production-vtable.rs", prod_vtable),
    ("production-shared-vtable.rs", prod_table),
):
    print(f"sha256 {name}: {sha256(content.encode()).hexdigest()}")
if not all(checks.values()):
    raise SystemExit(1)
