#!/usr/bin/env python3
"""Build a source-backed public API ledger from rustdoc JSON."""

from __future__ import annotations

import json
import pathlib
import difflib
import hashlib
import tarfile
import sys


if len(sys.argv) != 3:
    raise SystemExit("usage: scripts/generate-api-inventory.py PATH_TO_HTTP_JSON OFFICIAL_CRATE_ARCHIVE")

source = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[2])
data = json.loads(source.read_text())
index = data["index"]
items: dict[int, dict[str, object]] = {}
visited_modules: set[tuple[int, str]] = set()
visited_type_paths: set[tuple[int, str]] = set()
crate_root = pathlib.Path(__file__).resolve().parents[1]
archive_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
expected_archive_hash = "918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0"
if archive_hash != expected_archive_hash:
    raise SystemExit(
        f"official http 1.5.0 archive checksum mismatch: {archive_hash}"
    )
release_files: dict[str, list[str]] = {}
line_maps: dict[str, dict[int, int]] = {}
public_type_paths: dict[str, str] = {}
evidence_path = pathlib.Path("API_EVIDENCE.json")
evidence_records = json.loads(evidence_path.read_text()).get("entries", []) if evidence_path.exists() else []

with tarfile.open(archive, "r:gz") as crate:
    for member in crate.getmembers():
        prefix = "http-1.5.0/"
        if member.isfile() and member.name.startswith(prefix) and member.name.endswith(".rs"):
            relative = member.name[len(prefix):]
            extracted = crate.extractfile(member)
            if extracted is not None:
                release_files[relative] = extracted.read().decode("utf-8").splitlines()


def release_line(source_file: str, current_line: int) -> int | None:
    """Map an unchanged current source line back to the official archive."""
    if source_file not in line_maps:
        original = release_files.get(source_file)
        current_path = crate_root / source_file
        if original is None or not current_path.is_file():
            line_maps[source_file] = {}
        else:
            current = current_path.read_text().splitlines()
            mapping = {}
            for block in difflib.SequenceMatcher(
                a=original, b=current, autojunk=False
            ).get_matching_blocks():
                for offset in range(block.size):
                    mapping[block.b + offset + 1] = block.a + offset + 1
            line_maps[source_file] = mapping
    return line_maps[source_file].get(current_line)


def get(item_id: int | str) -> dict:
    return index[str(item_id)]


def kind_of(item: dict) -> str:
    return next(iter(item["inner"]))


def trait_path(trait: dict) -> str | None:
    if not trait:
        return None
    path_info = data.get("paths", {}).get(str(trait.get("id")), {})
    path = path_info.get("path")
    return "::".join(path) if path else trait.get("path")


def format_type(type_data: dict) -> str:
    """Render the rustdoc JSON type forms used in impl identities."""
    if not isinstance(type_data, dict):
        return str(type_data)
    if "resolved_path" in type_data:
        resolved = type_data["resolved_path"]
        path_info = data.get("paths", {}).get(str(resolved.get("id")), {})
        path = public_type_paths.get(str(resolved.get("id")))
        if path is None:
            path = "::".join(path_info.get("path", [])) or resolved.get("path", "?")
        args = resolved.get("args") or {}
        angle = args.get("angle_bracketed", {})
        rendered = []
        for arg in angle.get("args", []):
            if "type" in arg:
                rendered.append(format_type(arg["type"]))
            elif "lifetime" in arg:
                rendered.append(arg["lifetime"])
            elif "const" in arg:
                const = arg["const"]
                rendered.append(str(const.get("expr", const) if isinstance(const, dict) else const))
            else:
                rendered.append("?")
        for constraint in angle.get("constraints", []):
            name = constraint.get("name", "?")
            binding = constraint.get("binding", {})
            if "type" in binding:
                rendered.append(f"{name} = {format_type(binding['type'])}")
            elif "constraints" in binding:
                rendered.append(f"{name}: ?")
        if rendered:
            path += "<" + ", ".join(rendered) + ">"
        return path
    if "generic" in type_data:
        return type_data["generic"]
    if "primitive" in type_data:
        return type_data["primitive"]
    if "borrowed_ref" in type_data:
        ref = type_data["borrowed_ref"]
        lifetime = ref.get("lifetime")
        mut = "mut " if ref.get("is_mutable") else ""
        return "&" + (lifetime + " " if lifetime else "") + mut + format_type(ref["type"])
    if "tuple" in type_data:
        return "(" + ", ".join(format_type(t) for t in type_data["tuple"]) + ")"
    if "slice" in type_data:
        return "[" + format_type(type_data["slice"]) + "]"
    if "array" in type_data:
        array = type_data["array"]
        return f"[{format_type(array['type'])}; {array.get('len', '?')}]"
    if "raw_pointer" in type_data:
        ptr = type_data["raw_pointer"]
        return ("*mut " if ptr.get("is_mutable") else "*const ") + format_type(ptr["type"])
    return "?"


def trait_path_with_args(trait: dict) -> str | None:
    base = trait_path(trait)
    if not base:
        return None
    args = (trait.get("args") or {}).get("angle_bracketed", {})
    rendered = []
    for arg in args.get("args", []):
        if "type" in arg:
            rendered.append(format_type(arg["type"]))
        elif "lifetime" in arg:
            rendered.append(arg["lifetime"])
        elif "const" in arg:
            const = arg["const"]
            rendered.append(str(const.get("expr", const) if isinstance(const, dict) else const))
        else:
            rendered.append("?")
    for constraint in args.get("constraints", []):
        name = constraint.get("name", "?")
        binding = constraint.get("binding", {})
        if "type" in binding:
            rendered.append(f"{name} = {format_type(binding['type'])}")
        elif "constraints" in binding:
            rendered.append(f"{name}: ?")
    return base + ("<" + ", ".join(rendered) + ">" if rendered else "")


def public_self_type(public_path: str, impl_for: dict) -> str:
    resolved = impl_for.get("resolved_path", {})
    angle = (resolved.get("args") or {}).get("angle_bracketed", {})
    rendered = []
    for arg in angle.get("args", []):
        if "type" in arg:
            rendered.append(format_type(arg["type"]))
        elif "lifetime" in arg:
            rendered.append(arg["lifetime"])
        elif "const" in arg:
            const = arg["const"]
            rendered.append(str(const.get("expr", const) if isinstance(const, dict) else const))
        else:
            rendered.append("?")
    return public_path + ("<" + ", ".join(rendered) + ">" if rendered else "")


def add(item: dict, public_path: str, kind: str | None = None,
        impl_context: dict | None = None) -> None:
    if item.get("crate_id") != 0:
        return
    span = item.get("span") or {}
    file = span.get("filename", "")
    line = (span.get("begin") or [0, 0])[0]
    if not file.startswith("src/"):
        return
    item_id = int(item["id"])
    entry = items.setdefault(item_id, {
        "rustdoc_id": item_id,
        "name": item.get("name") or "",
        "kind": kind or kind_of(item),
        "source_file": file,
        "source_line": line,
        "release_source_line": release_line(file, line) if line else None,
        "declaration_origin": (
            "upstream_release" if line and release_line(file, line) is not None
            else "verification_support_added"
        ),
        "public_paths": [],
        "contract_reviewed": False,
        "body_proved": False,
        "trusted": False,
        "integrated_run": False,
        "implementation_contexts": [],
    })
    paths = entry["public_paths"]
    if public_path and public_path not in paths:
        paths.append(public_path)
    if impl_context is not None:
        contexts = entry["implementation_contexts"]
        if impl_context not in contexts:
            contexts.append(impl_context)


def add_fields(prefix: str, fields: list[int]) -> None:
    for field_id in fields:
        field = get(field_id)
        if field.get("visibility") == "public":
            add(field, f"{prefix}::{field['name']}", "field")


def add_type_members(item: dict, public_path: str) -> None:
    type_id = int(item["id"])
    visited_key = (type_id, public_path)
    if visited_key in visited_type_paths:
        return
    visited_type_paths.add(visited_key)

    inner = item["inner"]
    kind = kind_of(item)
    if kind == "struct":
        struct = inner["struct"]
        struct_kind = struct["kind"]
        if "plain" in struct_kind:
            add_fields(public_path, struct_kind["plain"].get("fields", []))
        elif "tuple" in struct_kind:
            add_fields(public_path, [f for f in struct_kind["tuple"] if isinstance(f, int)])
        impls = struct.get("impls", [])
    elif kind == "enum":
        enum = inner["enum"]
        impls = enum.get("impls", [])
        for variant_id in enum.get("variants", []):
            variant = get(variant_id)
            variant_path = f"{public_path}::{variant['name']}"
            add(variant, variant_path, "variant")
            variant_kind = variant.get("inner", {}).get("variant", {}).get("kind", {})
            if "struct" in variant_kind:
                add_fields(variant_path, variant_kind["struct"].get("fields", []))
            elif "tuple" in variant_kind:
                add_fields(variant_path, [f for f in variant_kind["tuple"] if isinstance(f, int)])
    elif kind == "union":
        union = inner["union"]
        impls = union.get("impls", [])
        add_fields(public_path, union.get("fields", []))
    else:
        return

    for impl_id in impls:
        impl = get(impl_id)["inner"]["impl"]
        impl_for = impl.get("for") or {}
        impl_for_resolved = impl_for.get("resolved_path") or {}
        # Rustdoc lists trait impls in a type's `impls` array even when that
        # type appears only as a trait argument (e.g. `From<X> for Error`).
        # Only attach associated items when this is actually the impl's Self.
        if str(impl_for_resolved.get("id")) != str(type_id):
            continue
        trait = impl.get("trait")
        trait_name = trait_path_with_args(trait) if trait else None
        if impl.get("is_synthetic") or impl.get("blanket_impl"):
            continue
        for member_id in impl.get("items", []):
            member = get(member_id)
            if member.get("crate_id") != 0:
                continue
            span = member.get("span") or {}
            if not span.get("filename", "").startswith("src/"):
                continue
            if member.get("visibility") not in ("public", "default"):
                continue
            name = member.get("name")
            if not name:
                continue
            if trait_name:
                self_type = public_self_type(public_path, impl_for)
                member_path = f"<{self_type} as {trait_name}>::{name}"
            else:
                member_path = f"{public_path}::{name}"
            span = get(impl_id).get("span") or {}
            impl_context = {
                "impl_id": int(impl_id),
                "trait": trait_name,
                "for": format_type(impl_for),
                "source_file": span.get("filename", ""),
                "source_line": (span.get("begin") or [0, 0])[0],
            }
            add(member, member_path, impl_context=impl_context)


def visit(item_id: int | str, prefix: str, *, root: bool = False) -> None:
    item = get(item_id)
    kind = kind_of(item)
    name = item.get("name")

    if kind == "module":
        module_path = f"{prefix}::{name}" if name and prefix else (name or prefix)
        key = (int(item["id"]), module_path)
        if key in visited_modules:
            return
        visited_modules.add(key)
        if not root:
            add(item, module_path, "module")
        for child_id in item["inner"]["module"].get("items", []):
            child = get(child_id)
            if child.get("visibility") == "public":
                visit(child_id, module_path)
        return

    if kind == "use":
        use = item["inner"]["use"]
        alias_name = name or use.get("name") or "*"
        alias_path = f"{prefix}::{alias_name}" if prefix else alias_name
        add(item, alias_path, "reexport")
        target_id = use.get("id")
        if target_id is not None:
            target = get(target_id)
            target_kind = kind_of(target)
            if target_kind == "module":
                visit(target_id, alias_path if use.get("is_glob") else alias_path)
            elif target_kind in ("struct", "enum", "union"):
                add(target, alias_path)
            elif target_kind not in ("impl",):
                add(target, alias_path)
        return

    if not root and item.get("visibility") != "public":
        return
    public_path = f"{prefix}::{name}" if prefix and name else (name or prefix)
    add(item, public_path)
    if kind == "trait":
        for member_id in item["inner"]["trait"].get("items", []):
            member = get(member_id)
            if member.get("crate_id") == 0 and (member.get("span") or {}).get("filename", "").startswith("src/"):
                member_name = member.get("name")
                if member_name:
                    add(member, f"{public_path}::{member_name}")


visit(data["root"], "", root=True)

# Resolve aliases before building impl paths, so rustdoc's canonical path for a
# source-private module (for example `uri::port::Port`) is rendered through a
# valid public re-export (`http::uri::Port`).
for row in items.values():
    if row["kind"] in ("struct", "enum", "union", "type_alias", "trait") and row["public_paths"]:
        paths = row["public_paths"]
        public_type_paths[str(row["rustdoc_id"])] = min(
            paths, key=lambda path: (path.count("::"), len(path), path)
        )

for row in list(items.values()):
    if row["kind"] not in ("struct", "enum", "union"):
        continue
    type_item = get(row["rustdoc_id"])
    for public_path in row["public_paths"]:
        add_type_members(type_item, public_path)

# A public implementation may have a foreign `Self` type and a local type as
# a trait argument, as in `impl From<Port<T>> for u16`. Such impl items are not
# necessarily attached to `Port` by rustdoc's `impls` array, so collect them
# explicitly from the rustdoc index. They are source-backed behavior of the
# published local type and must not disappear from the API ledger.
public_type_ids = {
    int(row["rustdoc_id"])
    for row in items.values()
    if row["kind"] in ("struct", "enum", "union", "type_alias", "trait")
}


def contains_public_type(value: object) -> bool:
    if isinstance(value, dict):
        resolved = value.get("resolved_path")
        if resolved and str(resolved.get("id")) in {str(i) for i in public_type_ids}:
            return True
        if str(value.get("id")) in {str(i) for i in public_type_ids}:
            return True
        return any(contains_public_type(child) for child in value.values())
    if isinstance(value, list):
        return any(contains_public_type(child) for child in value)
    return False


public_type_id_strings = {str(i) for i in public_type_ids}
for impl_id, impl_item in index.items():
    if impl_item.get("crate_id") != 0 or kind_of(impl_item) != "impl":
        continue
    impl = impl_item["inner"]["impl"]
    if impl.get("is_synthetic") or impl.get("blanket_impl"):
        continue
    span = impl_item.get("span") or {}
    if not span.get("filename", "").startswith("src/"):
        continue
    impl_for = impl.get("for") or {}
    for_id = (impl_for.get("resolved_path") or {}).get("id")
    trait = impl.get("trait")
    # Local Self impls are already indexed while walking each public type.
    if str(for_id) in public_type_id_strings or not (
        contains_public_type(trait or {}) or contains_public_type(impl_for)
    ):
        continue
    trait_name = trait_path_with_args(trait) if trait else None
    if not trait_name:
        continue
    for member_id in impl.get("items", []):
        member = get(member_id)
        if member.get("crate_id") != 0 or member.get("visibility") not in ("public", "default"):
            continue
        member_span = member.get("span") or {}
        if not member_span.get("filename", "").startswith("src/"):
            continue
        name = member.get("name")
        if not name:
            continue
        member_path = f"<{format_type(impl_for)} as {trait_name}>::{name}"
        impl_context = {
            "impl_id": int(impl_id),
            "trait": trait_name,
            "for": format_type(impl_for),
            "source_file": span.get("filename", ""),
            "source_line": (span.get("begin") or [0, 0])[0],
        }
        add(member, member_path, impl_context=impl_context)

inventory = sorted(
    items.values(),
    key=lambda row: (row["source_file"], row["source_line"], row["name"], row["kind"]),
)
for item in inventory:
    item["public_paths"].sort()
    matched = [
        record for record in evidence_records
        if record.get("source_file") == item["source_file"]
        and set(record.get("public_paths", [])) & set(item["public_paths"])
    ]
    if matched:
        # A source declaration may have multiple public aliases; all matching
        # evidence entries must agree on proof status.
        item["proof_evidence"] = sorted({record["evidence"] for record in matched})
        for field in ("contract_reviewed", "body_proved", "trusted", "integrated_run"):
            item[field] = all(bool(record.get(field, False)) for record in matched)

json_path = pathlib.Path("API_INVENTORY.json")
json_path.write_text(json.dumps({
    "source": "modified http 1.5.0 verification source; compared against the official archive",
    "official_archive": str(archive),
    "official_archive_sha256": archive_hash,
    "rustdoc_json": str(source),
    "status_fields": ["contract_reviewed", "body_proved", "trusted", "integrated_run"],
    "items": inventory,
}, indent=2) + "\n")

counts: dict[str, int] = {}
origin_counts: dict[str, int] = {}
for item in inventory:
    counts[str(item["kind"])] = counts.get(str(item["kind"]), 0) + 1
    origin = str(item["declaration_origin"])
    origin_counts[origin] = origin_counts.get(origin, 0) + 1

lines = [
    "# `http` 1.5.0 API and verification-support inventory",
    "",
    "Generated from rustdoc JSON for the modified verification tree and compared line-by-line with the official crates.io archive. The origin field distinguishes upstream declarations from verification-support additions. Proof statuses default to `false`; the small set of accepted actual-source leaf results comes from `API_EVIDENCE.json`, and none sets `integrated_run`.",
    "",
    f"Declarations in the current modified tree: **{len(inventory)}**.",
    "",
    "| Declaration origin | Count |",
    "|---|---:|",
    f"| upstream release declaration | {origin_counts.get('upstream_release', 0)} |",
    f"| verification-support addition | {origin_counts.get('verification_support_added', 0)} |",
    "",
    "| Kind | Count |",
    "|---|---:|",
]
for kind, count in sorted(counts.items()):
    lines.append(f"| `{kind}` | {count} |")
lines.extend([
    "",
    "The machine-readable per-entry ledger is [`API_INVENTORY.json`](API_INVENTORY.json). It records source location, mapped official-archive line when unchanged, declaration origin, exported path or paths, implementation identity for trait items, proof evidence, and `contract_reviewed`, `body_proved`, `trusted`, and `integrated_run` status. Evidence overrides are maintained in [`API_EVIDENCE.json`](API_EVIDENCE.json); leaf proofs never set `integrated_run`.",
    "",
    "The inventory follows public module trees and re-exports, and includes source-defined methods and associated items from public types and traits. Each item identifies whether its declaration line matches the official release archive or was added as verification support; added View/DeepModel/Invariant/model helpers are not counted as upstream API. Trait implementation entries retain their rustdoc impl id, trait arguments, and `for` type so overloads remain distinct. It excludes inherited standard-library defaults and compiler-generated blanket/auto impls without an `http` source body.",
    "",
])
pathlib.Path("API_INVENTORY.md").write_text("\n".join(lines))
print(f"wrote {len(inventory)} current API entries ({dict(sorted(counts.items()))}); origins={dict(sorted(origin_counts.items()))}")
