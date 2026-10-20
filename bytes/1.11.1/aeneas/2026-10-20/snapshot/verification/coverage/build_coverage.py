#!/usr/bin/env python3
"""Build a source-linked coverage ledger from the frozen LLBC and proof receipts.

This is an inventory/reconciliation tool only. It does not run Aeneas, Lean, or
the prover. Unmapped or unreceipted theorem claims never count as coverage.
"""

from __future__ import annotations

import collections
import hashlib
import json
import re
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "verification"
LLBC = ROOT / "evidence" / "adapted-whole-final" / "crate.llbc"
OUT = Path(__file__).resolve().parent
EMPTY_COMPILE_TIME_MARKER_NAMES = {
    ("src/buf/buf_impl.rs", "_assert_trait_object"),
    ("src/buf/buf_mut.rs", "_assert_trait_object"),
    ("src/bytes.rs", "_split_to_must_use"),
    ("src/bytes.rs", "_split_off_must_use"),
    ("src/bytes_mut.rs", "_split_to_must_use"),
    ("src/bytes_mut.rs", "_split_off_must_use"),
    ("src/bytes_mut.rs", "_split_must_use"),
    ("src/lib.rs", "assert_fields_are_eq"),
}


def read_json(path: Path) -> Any:
    return json.loads(path.read_text())


def untag_span(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        return {}
    return value.get("Untagged", {}) if isinstance(value.get("Untagged"), dict) else {}


def span_info(value: Any, files: dict[int, dict[str, Any]]) -> dict[str, Any] | None:
    tagged = untag_span(value)
    data = tagged.get("data")
    if data is None and isinstance(value, dict):
        data = value.get("data") or (value if "file_id" in value else None)
    if not isinstance(data, dict):
        return None
    file_id = data.get("file_id")
    file = files.get(file_id, {}) if isinstance(file_id, int) else {}
    file_name = file.get("name", {})
    if isinstance(file_name, dict):
        file_name = file_name.get("Local", str(file_name))
    return {
        "file": file_name,
        "line": (data.get("beg") or {}).get("line"),
        "column": (data.get("beg") or {}).get("col"),
        "end_line": (data.get("end") or {}).get("line"),
        "generated_from": span_info(tagged.get("generated_from_span") or (value.get("generated_from_span") if isinstance(value, dict) else None), files)
        if isinstance(tagged.get("generated_from_span") or (value.get("generated_from_span") if isinstance(value, dict) else None), dict)
        else None,
    }


def name_parts(raw: Any, traits: dict[int, str]) -> list[str]:
    parts: list[str] = []
    if not isinstance(raw, list):
        return parts
    for item in raw:
        if not isinstance(item, dict):
            continue
        ident = item.get("Ident")
        if isinstance(ident, list) and ident:
            parts.append(str(ident[0]))
            continue
        impl = item.get("Impl")
        if isinstance(impl, dict):
            trait = impl.get("Trait")
            if isinstance(trait, int):
                # In item names, `Impl.Trait` carries the TraitImpl id. It is
                # not the trait declaration id. Resolve through the
                # trait-implementation table before rendering the trait name.
                parts.append(f"<impl {traits.get(trait, 'Trait')}#{trait}>")
            elif "Ty" in impl:
                parts.append("<impl type>")
            else:
                parts.append("<impl>")
    return parts


def source_module(path: str | None) -> str:
    if not path:
        return "unknown"
    if path.startswith("src/"):
        return path[4:].removesuffix(".rs").replace("/", "::")
    return path


def item_path(entry: dict[str, Any], traits: dict[int, str]) -> list[str]:
    meta = entry.get("item_meta") or {}
    return name_parts(meta.get("name"), traits)


def final_name(parts: list[str]) -> str:
    return parts[-1] if parts else "<anonymous>"


def location_key(span: Any, files: dict[int, dict[str, Any]]) -> tuple[Any, ...] | None:
    info = span_info(span, files)
    if not info:
        return None
    return (info.get("file"), info.get("line"), info.get("column"), info.get("end_line"))


def has_builtin(raw_name: Any, builtin: str) -> bool:
    if isinstance(raw_name, list):
        return any(has_builtin(part, builtin) for part in raw_name)
    if isinstance(raw_name, dict):
        return any((key == "Builtin" and isinstance(value, list) and value and value[0] == builtin)
                   or has_builtin(value, builtin) for key, value in raw_name.items())
    return False


def local_inventory(llbc: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    translated = llbc["translated"]
    files = {item["id"]: item for item in translated["files"]}
    trait_declarations = {}
    for trait in translated.get("trait_decls", []):
        if isinstance(trait, dict):
            parts = name_parts((trait.get("item_meta") or {}).get("name"), {})
            trait_declarations[trait.get("def_id")] = "::".join(parts) if parts else f"Trait{trait.get('def_id')}"
    trait_impls = {impl.get("def_id"): impl for impl in translated.get("trait_impls", [])
                   if isinstance(impl, dict)}
    # Short names are Charon-generated stable identities for concrete impls;
    # they distinguish source-generated overloads such as Index<Range> from
    # Index<RangeFrom>, whose expanded bodies share a macro source span.
    trait_impl_short_names = {}
    for item in translated.get("short_names", []):
        key = item.get("key") if isinstance(item, dict) else None
        value = item.get("value") if isinstance(item, dict) else None
        if isinstance(key, dict) and isinstance(key.get("TraitImpl"), int):
            ids = [part.get("Ident", [None])[0] for part in (value or [])
                   if isinstance(part, dict) and isinstance(part.get("Ident"), list)]
            if ids:
                trait_impl_short_names[key["TraitImpl"]] = "::".join(map(str, ids))
    trait_impl_trait_names = {}
    trait_impl_source_headers = {}
    for impl_id, impl in trait_impls.items():
        impl_trait = impl.get("impl_trait") or {}
        trait_id = impl_trait.get("id")
        if isinstance(trait_id, int):
            trait_impl_trait_names[impl_id] = trait_declarations.get(trait_id, f"Trait{trait_id}")
        source_text = (impl.get("item_meta") or {}).get("source_text")
        if isinstance(source_text, str) and source_text.strip():
            trait_impl_source_headers[impl_id] = source_text.strip().splitlines()[0].strip()

    local_globals = [g for g in translated.get("global_decls", [])
                     if isinstance(g, dict) and (g.get("item_meta") or {}).get("is_local")]
    global_keys: dict[tuple[Any, ...], dict[str, Any]] = {}
    for global_decl in local_globals:
        global_meta = global_decl.get("item_meta") or {}
        global_keys[(json.dumps(global_meta.get("name"), sort_keys=True),
                     location_key(global_meta.get("span"), files))] = global_decl

    functions: list[dict[str, Any]] = []
    for fn in translated.get("fun_decls", []):
        if not isinstance(fn, dict):
            continue
        meta = fn.get("item_meta") or {}
        if not meta.get("is_local"):
            continue
        parts = item_path(fn, trait_impl_trait_names)
        declared = span_info(meta.get("span"), files)
        effective = declared.get("generated_from") if declared and declared.get("generated_from") else declared
        generated = bool(declared and declared.get("generated_from"))
        module = source_module((effective or {}).get("file"))
        attr = meta.get("attr_info") or {}
        opacity = meta.get("opacity", "Unknown")
        generated_forwarder = generated and module in ("buf::buf_impl", "buf::buf_mut")
        raw_name = meta.get("name")
        matching_global = global_keys.get((json.dumps(raw_name, sort_keys=True), location_key(meta.get("span"), files)))
        is_global_initializer = matching_global is not None
        is_drop_glue = has_builtin(raw_name, "DropGlue")
        if is_drop_glue:
            rust_kind = "compiler_generated_drop_glue"
        elif is_global_initializer:
            global_kind = matching_global.get("global_kind")
            rendered_name = "::".join(name_parts(raw_name, trait_impl_trait_names)).upper()
            if "VTABLE" in rendered_name:
                rust_kind = "trait_dictionary_or_vtable_initializer"
            elif has_builtin(raw_name, "PromotedConst") or global_kind == "AnonConst":
                rust_kind = "promoted_or_anonymous_constant_initializer"
            elif isinstance(global_kind, dict) and "Static" in global_kind:
                rust_kind = "static_initializer"
            else:
                rust_kind = "named_constant_initializer"
        elif generated_forwarder:
            rust_kind = "source_generated_forwarding_method"
        elif generated:
            rust_kind = "source_generated_method"
        else:
            rust_kind = "rust_method" if any(p.startswith("<impl") for p in parts) else "rust_function"
        nonfunction_kinds = {
            "trait_dictionary_or_vtable_initializer", "promoted_or_anonymous_constant_initializer",
            "static_initializer", "named_constant_initializer", "compiler_generated_drop_glue",
        }
        is_source_function = rust_kind not in nonfunction_kinds
        if not is_source_function:
            record_status = "not a source Rust function"
            record_reason = "LLBC non-function initializer slot" if is_global_initializer else "compiler-generated drop glue"
        else:
            record_status = "blocked" if opacity == "Opaque" else "not yet specified"
            record_reason = "fixed opaque implementation boundary" if opacity == "Opaque" else "no mapped, receipted body theorem or full API contract"
        fn_id = f"llbc_fun_{fn.get('def_id')}"
        source_impl = (fn.get("src") or {}).get("TraitImpl") if isinstance(fn.get("src"), dict) else None
        impl_id = None
        trait_id = None
        trait_name = None
        if isinstance(source_impl, dict):
            impl_ref = source_impl.get("impl_ref") or {}
            impl_id = impl_ref.get("id")
            trait_id = (source_impl.get("trait_ref") or {}).get("id")
            trait_name = trait_declarations.get(trait_id) if isinstance(trait_id, int) else None
        functions.append({
            "id": fn_id,
            "llbc_def_id": fn.get("def_id"),
            "rust_path": "::".join(parts),
            "trait_impl_id": impl_id,
            "trait_id": trait_id,
            "trait_name": trait_name,
            "trait_impl_short_name": trait_impl_short_names.get(impl_id),
            "trait_impl_source_header": trait_impl_source_headers.get(impl_id),
            "name": final_name(parts),
            "module": module,
            "source_file": (effective or {}).get("file"),
            "source_line": (effective or {}).get("line"),
            "declared_at": {
                "file": (declared or {}).get("file"),
                "line": (declared or {}).get("line"),
            },
            "generated_from": declared.get("generated_from") if declared else None,
            "declaration_kind": rust_kind,
            "is_source_rust_function": is_source_function,
            "matched_global_initializer": is_global_initializer,
            "matched_global_kind": matching_global.get("global_kind") if matching_global else None,
            "compiler_generated_drop_glue": is_drop_glue,
            "visibility": "public" if attr.get("public") else "private",
            "opacity": opacity,
            "has_transparent_body": isinstance(fn.get("body"), dict),
            "status": record_status,
            "status_reason": record_reason,
            "evidence": [],
        })

    nonfunctions: dict[str, Any] = {}
    for kind in ("global_decls", "type_decls", "trait_decls", "trait_impls"):
        records = [x for x in translated.get(kind, []) if isinstance(x, dict)]
        local = [x for x in records if (x.get("item_meta") or {}).get("is_local")]
        nonfunctions[kind] = {
            "record_count": len(records),
            "local_count": len(local),
            "nonlocal_count": len(records) - len(local),
        }
        if kind == "global_decls":
            def global_kind_label(value: Any) -> str:
                return value if isinstance(value, str) else json.dumps(value, sort_keys=True)
            nonfunctions[kind]["local_kinds"] = dict(collections.Counter(global_kind_label(x.get("global_kind", "unknown")) for x in local))
            nonfunctions[kind]["local_opacity"] = dict(collections.Counter((x.get("item_meta") or {}).get("opacity", "Unknown") for x in local))
        elif kind == "type_decls":
            nonfunctions[kind]["local_opacity"] = dict(collections.Counter((x.get("item_meta") or {}).get("opacity", "Unknown") for x in local))
    return functions, {
        "actual_function_records": sum(isinstance(x, dict) for x in translated.get("fun_decls", [])),
        "nondeclaration_fun_slots": sum(x is None for x in translated.get("fun_decls", [])),
        "local_functions": len(functions),
        "local_source_rust_functions": sum(f["is_source_rust_function"] for f in functions),
        "local_nonfunction_llbc_fun_records": sum(not f["is_source_rust_function"] for f in functions),
        "local_opacity": dict(collections.Counter(f["opacity"] for f in functions)),
        "source_function_opacity": dict(collections.Counter(f["opacity"] for f in functions if f["is_source_rust_function"])),
        "local_visibility": dict(collections.Counter(f["visibility"] for f in functions)),
        "local_declaration_kind": dict(collections.Counter(f["declaration_kind"] for f in functions)),
        "visibility_by_opacity_and_kind": {
            f"{vis}/{opacity}/{kind}": count
            for (vis, opacity, kind), count in collections.Counter(
                (f["visibility"], f["opacity"], f["declaration_kind"]) for f in functions
            ).items()
        },
        "nonfunction_declarations": nonfunctions,
        "other_nonfunction_slots": {
            "global_decls": sum(x is None for x in translated.get("global_decls", [])),
            "type_decls": sum(x is None for x in translated.get("type_decls", [])),
        },
    }


def proof_assessment(assumptions: list[Any], axioms: list[str] | None, tier: str) -> tuple[str, list[str]]:
    reasons = []
    axioms = list(axioms or [])
    std = {"propext", "Quot.sound", "Classical.choice"}
    extra = [a for a in axioms if a not in std]
    if assumptions:
        reasons.append("explicit theorem preconditions: " + "; ".join(map(str, assumptions)))
    if extra:
        reasons.append("nonstandard/model axiom dependencies: " + ", ".join(extra))
    if tier == "full_contract" and not reasons:
        return "proved", reasons
    if tier == "body_property" and not reasons:
        return "proved", reasons
    return "conditional proved", reasons


def source_selector(raw: str, file_hint: str | None = None) -> tuple[str | None, str | None, str]:
    """Resolve known API spellings to a source file and final Rust function name."""
    raw = raw.strip()
    line = None
    # Charon's generic paths legitimately contain `@T`; only a numeric
    # trailing suffix is our optional source-line hint.
    line_match = re.search(r"@(\d+)$", raw)
    if line_match:
        line = line_match.group(1)
        raw = raw[:line_match.start()]
    if file_hint:
        file = file_hint
    else:
        low = raw.lower()
        owner = re.search(r"\bfor\s+(.+?)\}\s*::", raw)
        owner_type = owner.group(1).strip() if owner else ""
        owner_low = owner_type.lower()
        if "::buf_impl::buf::" in low:
            file = "src/buf/buf_impl.rs"
        elif "::buf_mut::bufmut::" in low or "::buf_mut::bufmut::" in low:
            file = "src/buf/buf_mut.rs"
        elif "vecdeque" in owner_low or "vec_deque" in owner_low:
            file = "src/buf/vec_deque.rs"
        elif "buf::chain::chain" in owner_low:
            file = "src/buf/chain.rs"
        elif "buf::take::take" in owner_low:
            file = "src/buf/take.rs"
        elif "buf::limit::limit" in owner_low:
            file = "src/buf/limit.rs"
        elif "buf::iter::intoiter" in owner_low:
            file = "src/buf/iter.rs"
        elif "bytesmut" in owner_low or "bytes_mut::bytesmut" in owner_low:
            file = "src/bytes_mut.rs"
        elif re.search(r"(?:^|::)bytes::bytes(?:$|\b)", owner_low):
            file = "src/bytes.rs"
        elif "bytes_mut" in low or "bytesmut" in low:
            file = "src/bytes_mut.rs"
        elif "bytes::bytes" in low or "bytes::len" in low or "bytes::is_empty" in low:
            file = "src/bytes.rs"
        elif "buf::chain" in low or "chain::" in low or "chain::" in raw.lower():
            file = "src/buf/chain.rs"
        elif "buf::take" in low or "take::" in low:
            file = "src/buf/take.rs"
        elif "buf_mut" in low or "bufmut" in low:
            file = "src/buf/buf_mut.rs"
        elif "uninit_slice" in low:
            file = "src/buf/uninit_slice.rs"
        elif "buf_impl" in low or "::buf::buf" in low:
            file = "src/buf/buf_impl.rs"
        elif "buf::limit" in low:
            file = "src/buf/limit.rs"
        elif "buf::iter" in low or "intoiter" in low:
            file = "src/buf/iter.rs"
        elif "vec_deque" in low:
            file = "src/buf/vec_deque.rs"
        elif "saturating_sub_usize_u64" in low or "min_u64_usize" in low:
            file = "src/lib.rs"
        else:
            file = None
    # Strip a Charon target suffix while preserving its method/function basename.
    cleaned = re.sub(r"::Method$|::Function$", "", raw)
    cleaned = re.sub(r"::impl\[[^]]+\]", "", cleaned)
    final = cleaned.split("::")[-1]
    return file, line, final


def candidates(functions: list[dict[str, Any]], target: str, source_file: str | None = None,
               source_span: str | None = None, source_impl_short_name: str | None = None) -> list[dict[str, Any]]:
    file_hint = source_file
    line = None
    if source_span:
        match = re.match(r"(src/[^:]+):(\d+)", source_span)
        if match:
            file_hint, line = match.group(1), int(match.group(2))
    file, embedded_line, basename = source_selector(target, file_hint)
    if line is None and embedded_line:
        try:
            line = int(embedded_line)
        except ValueError:
            pass
    pool = [f for f in functions if f["name"] == basename and f["is_source_rust_function"]]
    if file:
        pool = [f for f in pool if f["source_file"] == file]
    if line:
        exact = [f for f in pool if f["source_line"] == line or f["declared_at"].get("line") == line]
        if exact:
            pool = exact
    # Trait default methods share a basename/source file with generated
    # forwarding impls. An explicit `Buf::method` or `BufMut::method` target
    # names the trait default and should resolve to that source body alone.
    normalized_target = re.sub(r"^.*?::buf::", "", target)
    if normalized_target.startswith("buf_impl::Buf::") or normalized_target.startswith("buf_mut::BufMut::"):
        exact_path = [f for f in pool if f["rust_path"].endswith(normalized_target)]
        if len(exact_path) == 1:
            pool = exact_path
    if source_impl_short_name:
        pool = [f for f in pool if f.get("trait_impl_short_name") == source_impl_short_name]
    return pool


def add_evidence(functions: list[dict[str, Any]], entry: dict[str, Any], selector: str,
                 source_file: str | None, tier: str, assumptions: list[Any], axioms: list[str] | None,
                 theorem: str, group: str, note: str | None = None,
                 source_span: str | None = None,
                 source_impl_short_name: str | None = None) -> dict[str, Any]:
    matches = candidates(functions, selector, source_file, source_span, source_impl_short_name)
    if len(matches) != 1:
        return {"theorem": theorem, "selector": selector, "group": group,
                "mapped": False, "candidate_count": len(matches),
                "candidate_ids": [f["id"] for f in matches]}
    fn = matches[0]
    if fn["opacity"] == "Opaque" and tier != "full_contract":
        return {"theorem": theorem, "selector": selector, "group": group,
                "mapped": False, "candidate_count": 1, "candidate_ids": [fn["id"]],
                "reason": "fixed opaque boundary has no translated body to prove"}
    status, reasons = proof_assessment(assumptions, axioms, tier)
    ev = {
        "theorem": theorem,
        "group": group,
        "tier": tier,
        "coverage_scope": "full Rust function contract" if tier == "full_contract" else "mapped body property",
        "status": status,
        "claim": note,
        "assumptions": assumptions,
        "axiom_dependencies": axioms or [],
        "qualification": reasons,
    }
    fn["evidence"].append(ev)
    if fn["opacity"] == "Opaque":
        fn["status"] = "blocked"
        fn["status_reason"] = "fixed opaque implementation boundary"
    else:
        evidence_statuses = {item["status"] for item in fn["evidence"]}
        fn["status"] = "proved" if "proved" in evidence_statuses else "conditional proved"
        reasons_all = [r for item in fn["evidence"] for r in item.get("qualification", [])]
        fn["status_reason"] = "mapped kernel-checked theorem" if fn["status"] == "proved" else "; ".join(dict.fromkeys(reasons_all)) or "proof has a declared scope restriction"
    return {"theorem": theorem, "selector": selector, "group": group,
            "mapped": True, "function_id": fn["id"], "status": status}


def load_native_evidence(functions: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[str]]:
    native_receipt_path = VERIFY / "evidence" / "native-result.json"
    native_receipt = read_json(native_receipt_path) if native_receipt_path.exists() else {}
    checked = set()
    if native_receipt.get("status") == "kernel_checked":
        for info in native_receipt.get("proof_files", {}).values():
            checked.update(info.get("theorems", []))
    mapping: list[dict[str, Any]] = []
    unmapped: list[str] = []
    basic_path = VERIFY / "basic" / "manifest.json"
    if basic_path.exists():
        basic = read_json(basic_path)
        for prop in basic.get("properties", []):
            theorem_full = prop.get("theorem", "")
            theorem = theorem_full.split(".")[-1]
            if theorem not in checked:
                continue
            rust_fn = prop.get("rust_function", "")
            src = prop.get("source", "")
            source_file = "src/" + src.split("src/", 1)[1].split(":", 1)[0] if "src/" in src else None
            line_match = re.search(r":(\d+)", src)
            line = line_match.group(1) if line_match else None
            selector = rust_fn
            if line:
                selector += "@" + line
            bundle = re.search(r"::\{([^}]+)\}", rust_fn)
            if bundle:
                names = [name.strip() for name in bundle.group(1).split(",")]
                line_ranges = re.findall(r"(\d+)(?:-(\d+))?", src.split(":", 1)[-1])
                for index, name in enumerate(names):
                    selected_line = line
                    if index < len(line_ranges):
                        selected_line = line_ranges[index][0]
                    item_selector = rust_fn.split("::{", 1)[0] + "::" + name + ("@" + selected_line if selected_line else "")
                    result = add_evidence(functions, prop, item_selector, source_file, "body_property",
                                          basic.get("scope", {}).get("explicit_assumptions", []),
                                          prop.get("print_axioms", []), theorem_full, "native_basic",
                                          prop.get("specification"))
                    mapping.append(result)
                    if not result["mapped"]:
                        unmapped.append(theorem_full + " -> " + name)
            else:
                result = add_evidence(functions, prop, selector, source_file, "body_property",
                                      basic.get("scope", {}).get("explicit_assumptions", []),
                                      prop.get("print_axioms", []), theorem_full, "native_basic",
                                      prop.get("specification"))
                mapping.append(result)
                if not result["mapped"]:
                    unmapped.append(theorem_full)

    adapters_path = VERIFY / "adapters" / "manifest.json"
    adapter_map = {
        "take_new_fields": ("bytes::buf::take::new", "src/buf/take.rs"),
        "take_limit_returns_field": ("bytes::buf::take::Take::limit", "src/buf/take.rs"),
        "take_into_inner_returns_field": ("bytes::buf::take::Take::into_inner", "src/buf/take.rs"),
        "take_get_ref_returns_field": ("bytes::buf::take::Take::get_ref", "src/buf/take.rs"),
        "take_set_limit_changes_only_limit": ("bytes::buf::take::Take::set_limit", "src/buf/take.rs"),
        "take_remaining_uses_min": ("bytes::buf::buf_impl::Buf::remaining", "src/buf/take.rs"),
        "take_remaining_is_bounded": ("bytes::buf::buf_impl::Buf::remaining", "src/buf/take.rs"),
        "take_chunk_returns_bounded_prefix": ("bytes::buf::buf_impl::Buf::chunk", "src/buf/take.rs"),
        "take_advance_updates_inner_and_limit": ("bytes::buf::buf_impl::Buf::advance", "src/buf/take.rs"),
        "take_advance_drops_observed_prefix": ("bytes::buf::buf_impl::Buf::advance", "src/buf/take.rs"),
        "chain_new_fields": ("bytes::buf::chain::Chain::new", "src/buf/chain.rs"),
        "chain_remaining_is_saturating_sum": ("bytes::buf::buf_impl::Buf::remaining", "src/buf/chain.rs"),
        "chain_advance_within_first": ("bytes::buf::buf_impl::Buf::advance", "src/buf/chain.rs"),
        "chain_advance_crosses_boundary": ("bytes::buf::buf_impl::Buf::advance", "src/buf/chain.rs"),
        "chain_advance_starts_in_second": ("bytes::buf::buf_impl::Buf::advance", "src/buf/chain.rs"),
    }
    if adapters_path.exists():
        adapters = read_json(adapters_path)
        axioms_by_set = adapters.get("axiom_sets", {})
        for theorem in adapters.get("theorems", []):
            theorem_name = theorem.get("name", "")
            if theorem_name not in checked:
                continue
            if theorem_name not in adapter_map:
                continue  # supporting arithmetic helper, not a bytes Rust function
            selector, source_file = adapter_map[theorem_name]
            result = add_evidence(functions, theorem, selector, source_file, "body_property",
                                  theorem.get("preconditions", []),
                                  axioms_by_set.get(theorem.get("axiom_set"), []),
                                  theorem_name, "native_adapters", theorem.get("postcondition"))
            mapping.append(result)
            if not result["mapped"]:
                unmapped.append(theorem_name)

    audit_path = VERIFY / "elicit-integration" / "audit-selected-artifacts" / "receipt.json"
    if audit_path.exists():
        audit = read_json(audit_path)
        target_reports = (audit.get("report") or {}).get("backend_evidence", {}).get("targets", {})
        audit_ids = {g.get("target"): g.get("id") for g in (audit.get("goals") or [])}
        for target, report in target_reports.items():
            if not (report.get("checked") and report.get("full")):
                continue
            if report.get("outcome") not in (None, "success"):
                continue
            goal_id = audit_ids.get(target, target)
            audit_owners = {
                "Chain": ("src/buf/chain.rs", "chain"),
                "Take": ("src/buf/take.rs", "take"),
                "Limit": ("src/buf/limit.rs", "limit"),
                "IntoIter": ("src/buf/iter.rs", "iter"),
            }
            audit_owner = goal_id.split("::", 1)[0]
            owner_info = audit_owners.get(audit_owner)
            if owner_info is None:
                continue
            source_file, owner_module = owner_info
            # Audit ids are stable API names such as Chain::first_ref / Take::new.
            method = goal_id.split("::")[-1]
            selector = "bytes::buf::" + owner_module + "::" + method
            result = add_evidence(functions, {}, selector, source_file, "full_contract", [],
                                  [], target, "strict_elicit_audit",
                                  "original Rust contract checked for all translated inputs")
            mapping.append(result)
            if not result["mapped"]:
                unmapped.append(target)
    return mapping, unmapped


def receipt_theorems(receipt: dict[str, Any]) -> tuple[set[str], dict[str, list[str]]]:
    """Read the loose receipt shapes used by the native and extension runners."""
    names: set[str] = set()
    axioms: dict[str, list[str]] = {}

    def visit(value: Any, context: str = "") -> None:
        if isinstance(value, dict):
            for k, v in value.items():
                if k in {"theorems", "compiled_theorems", "kernel_checked_theorems", "successful_theorems"} and isinstance(v, list):
                    for item in v:
                        if isinstance(item, str):
                            names.add(item)
                        elif isinstance(item, dict):
                            name = item.get("theorem") or item.get("name")
                            if name and item.get("checked", item.get("status") in ("ok", "success", "kernel_checked", "proved")):
                                names.add(name)
                                ax = item.get("axioms") or item.get("axiom_dependencies") or []
                                if isinstance(ax, list):
                                    axioms[name] = ax
                visit(v, str(k))
        elif isinstance(value, list):
            for item in value:
                visit(item, context)

    visit(receipt)
    receipt_axioms = receipt.get("axioms") or receipt.get("axiom_dependencies") or {}
    if isinstance(receipt_axioms, dict):
        for theorem, values in receipt_axioms.items():
            if isinstance(values, list):
                axioms[theorem] = values
    return names, axioms


def load_extension_evidence(functions: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[str], list[dict[str, Any]]]:
    results: list[dict[str, Any]] = []
    blocked_roots: list[dict[str, Any]] = []
    unmapped: list[str] = []
    receipt_statuses: list[dict[str, Any]] = []
    for manifest_path in sorted((VERIFY / "extensions").glob("*/manifest.json")):
        manifest = read_json(manifest_path)
        group = manifest.get("group", manifest_path.parent.name)
        receipt_ref = manifest.get("receipt")
        if receipt_ref:
            requested_receipt = Path(receipt_ref)
            candidates_to_check = [requested_receipt] if requested_receipt.is_absolute() else [VERIFY / requested_receipt, manifest_path.parent / requested_receipt]
            receipt_path = next((p for p in candidates_to_check if p.exists()), candidates_to_check[0])
        else:
            receipt_path = manifest_path.parent / "receipt.json"
        if not receipt_path.exists():
            receipt_statuses.append({"group": group, "status": "missing_receipt", "receipt": str(receipt_path.relative_to(VERIFY)) if receipt_path.is_relative_to(VERIFY) else str(receipt_path)})
            continue
        receipt = read_json(receipt_path)
        mismatches = []
        for relative, expected_hash in (receipt.get("files") or {}).items():
            proof_file = receipt_path.parent / relative
            if not proof_file.is_file():
                mismatches.append({"file": relative, "reason": "missing"})
                continue
            observed_hash = hashlib.sha256(proof_file.read_bytes()).hexdigest()
            if observed_hash != expected_hash:
                mismatches.append({"file": relative, "reason": "sha256 mismatch", "expected": expected_hash, "observed": observed_hash})
        boundary_hash = receipt.get("source_boundary_sha256")
        baseline_path = VERIFY / "opaque-baseline.json"
        if boundary_hash and hashlib.sha256(baseline_path.read_bytes()).hexdigest() != boundary_hash:
            mismatches.append({"file": "opaque-baseline.json", "reason": "source boundary hash mismatch"})
        receipt_valid = receipt.get("status") == "kernel_checked" and not mismatches
        receipt_statuses.append({
            "group": group,
            "status": "kernel_checked_current" if receipt_valid else (
                "stale_receipt_hashes" if mismatches else receipt.get("status", "unknown")),
            "receipt": str(receipt_path.relative_to(VERIFY)) if receipt_path.is_relative_to(VERIFY) else str(receipt_path),
            "validated_file_count": len(receipt.get("files") or {}),
            "mismatches": mismatches,
        })
        if not receipt_valid:
            blocked_roots.extend({"group": group, **b} for b in manifest.get("blocked_roots", []))
            continue
        checked_names, receipt_axioms = receipt_theorems(receipt)
        for proof in manifest.get("proofs", []):
            theorem = proof.get("theorem", "")
            if not theorem or theorem not in checked_names:
                continue
            assumptions = proof.get("assumptions", [])
            # The runner's kernel receipt is the authority for actual axiom
            # dependencies. Manifest summaries are useful documentation, but
            # may be incomplete or stale and cannot upgrade a proof's status.
            extra_axioms = receipt_axioms.get(theorem, [])
            claims = proof.get("rust_functions", [])
            for claim in claims:
                if isinstance(claim, dict):
                    target = claim.get("target") or claim.get("rust_function") or ""
                    file = claim.get("source_file") or proof.get("source_file")
                    span = claim.get("source_span") or proof.get("source_span")
                    impl_short_name = claim.get("source_impl_short_name") or proof.get("source_impl_short_name")
                else:
                    target = str(claim)
                    file = proof.get("source_file")
                    span = proof.get("source_span")
                    impl_short_name = proof.get("source_impl_short_name")
                result = add_evidence(functions, proof, target, file, "body_property", assumptions,
                                      extra_axioms, theorem, group, proof.get("verified_behavior"), span,
                                      impl_short_name)
                results.append(result)
                if not result["mapped"]:
                    unmapped.append(theorem + " -> " + target)
        blocked_roots.extend({"group": group, **b} for b in manifest.get("blocked_roots", []))
    return results, blocked_roots, unmapped, receipt_statuses


def owner_shape(raw: str | None) -> tuple[bool, bool, str] | None:
    """Return a lightweight type shape for matching a blocker to an impl."""
    if not raw:
        return None
    value = re.sub(r"'[_A-Za-z0-9]+", "", raw.strip()).replace(" ", "")
    is_ref = value.startswith("&")
    value = value[1:] if is_ref else value
    is_mut = value.startswith("mut")
    value = value[3:] if is_mut else value
    if value.startswith("[") and "]" in value:
        inner = value[1:value.index("]")]
        inner = inner.split("::")[-1]
        inner = inner.split("<", 1)[0]
        return is_ref, is_mut, f"slice:{inner}"
    value = value.split("::")[-1]
    value = value.split("<", 1)[0]
    return is_ref, is_mut, value


def blocked_root_source_file(root: dict[str, Any]) -> str:
    source_text = root.get("source", "")
    source_match = re.search(r"(src/[^,\s;]+\.rs)", source_text)
    if source_match:
        return source_match.group(1)
    target = root.get("rust_function", "")
    selected, _, _ = source_selector(target)
    return selected or "unknown"


def blocked_root_source_span(root: dict[str, Any], source_file: str) -> str | None:
    source_text = root.get("source", "")
    line_match = re.search(r"\blines?\s+(\d+)", source_text)
    if line_match and source_file != "unknown":
        return f"{source_file}:{line_match.group(1)}"
    return None


def resolve_blocked_root(functions: list[dict[str, Any]], root: dict[str, Any]) -> dict[str, Any]:
    """Annotate a manifest blocker against the current receipted function ledger."""
    result = dict(root)
    target = root.get("rust_function", "")
    source_file = blocked_root_source_file(root)
    source_span = blocked_root_source_span(root, source_file)
    matches = candidates(functions, target, source_file, source_span)

    # A Charon target's `for ...` type is more specific than its rendered LLBC
    # impl path. Use the source impl header where available to distinguish, for
    # example, the two mutable-slice BufMut implementations and owned versus
    # borrowed IntoIterator implementations.
    owner_match = re.search(r"\bfor\s+(.+?)\}\s*::", target)
    owner = owner_shape(owner_match.group(1)) if owner_match else None
    if owner:
        exact = []
        for fn in matches:
            header = fn.get("trait_impl_source_header") or ""
            header_match = re.search(r"\bfor\s+(.+?)\s*\{", header)
            header_owner = owner_shape(header_match.group(1)) if header_match else None
            if header_owner == owner:
                exact.append(fn)
        if exact:
            matches = exact

    ids = [fn["id"] for fn in matches]
    statuses = sorted({fn["status"] for fn in matches})
    result["source_file"] = source_file
    if source_span:
        result["source_span_hint"] = source_span
    result["candidate_function_ids"] = ids
    result["candidate_statuses"] = statuses
    if not matches:
        result["resolution_status"] = "unmapped_blocked_root"
        result["resolution_detail"] = "no source Rust function matched the selector and source hints"
    elif all(fn["opacity"] == "Opaque" for fn in matches):
        result["resolution_status"] = "fixed_opaque_boundary"
    elif all(fn["status"] in ("proved", "conditional proved") for fn in matches):
        result["resolution_status"] = "resolved_by_receipted_body_property"
        result["resolution_detail"] = "all matching source bodies have current receipted body-property evidence"
    elif all(fn["declaration_kind"] == "source_generated_forwarding_method" for fn in matches):
        result["resolution_status"] = "unresolved_blanket_forwarder"
        result["overlaps_blanket_forwarder_family"] = source_file in ("src/buf/buf_impl.rs", "src/buf/buf_mut.rs")
    else:
        result["resolution_status"] = "unresolved_function_body"
    return result


def add_known_report_only_boundaries(functions: list[dict[str, Any]], blocked_by_file: dict[str, list[dict[str, Any]]]) -> None:
    """Record documented translation limits that are not extension manifest roots."""
    known_compiler = {
        ("src/loom.rs", 16, "with_mut"): {
            "group": "whole_crate_compiler_integration",
            "reason": "The whole generated module does not type-check this translated `AtomicMut::with_mut` body: its mutable callback is emitted as a state/value pair, but `FnOnce::call_once` returns only `R`.",
            "detail": "`verification/backend/WholeGenerated.lean:11405` reports `expected a product type`; the emitted body destructures `(t, p1)` from `call_once` although the interface expects `R`.",
            "evidence": "verification/backend/whole-generated-lean.log:981",
        },
    }
    known_models = {
        ("src/buf/chain.rs", 29, "fmt"),
        ("src/buf/iter.rs", 20, "fmt"),
        ("src/buf/limit.rs", 8, "fmt"),
        ("src/buf/take.rs", 12, "fmt"),
        ("src/buf/uninit_slice.rs", 211, "fmt"),
        ("src/lib.rs", 137, "fmt"),
        ("src/lib.rs", 147, "fmt"),
    }
    format_model = {
        "group": "aeneas_core_format_model",
        "reason": "Aeneas models `core::fmt::Arguments` as `Unit` and its formatter operations as output-erasing placeholders, so this body cannot establish the rendered debug/display message.",
        "evidence": "aeneas/backends/lean/Aeneas/Std/Core/Fmt.lean:50-60, 93-135",
    }
    for fn in functions:
        key = (fn.get("source_file"), fn.get("source_line"), fn.get("name"))
        if fn.get("status") != "not yet specified":
            continue
        if key in known_compiler:
            metadata = known_compiler[key]
            kind = "compiler_or_extraction_blocker"
            resolution = "unresolved_function_body"
        elif key in known_models:
            metadata = format_model
            kind = "translation_model_boundary"
            resolution = "unresolved_translation_model_boundary"
        else:
            continue
        blocked_by_file[key[0]].append({
            "kind": kind,
            "resolution_status": resolution,
            "source_file": key[0],
            "source_span_hint": f"{key[0]}:{key[1]}",
            "rust_function": fn.get("rust_path"),
            "candidate_function_ids": [fn.get("id")],
            "candidate_statuses": [fn.get("status")],
            **metadata,
        })


def build_gaps(functions: list[dict[str, Any]], blocked_roots: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_file: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    for fn in functions:
        if fn["is_source_rust_function"]:
            by_file[fn.get("source_file") or "unknown"].append(fn)
    blocked_by_file: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    resolved_by_file: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    for root in blocked_roots:
        if not root.get("resolution_status"):
            root = resolve_blocked_root(functions, root)
        reason = root.get("reason", "")
        source_file = root["source_file"]
        reason_lower = reason.lower()
        if "opaque" in reason_lower or "frozen baseline" in reason_lower:
            kind = "fixed_opaque_boundary"
        elif "axiom" in reason_lower or "aeneas model" in reason_lower or "interface method" in reason_lower:
            kind = "translation_model_boundary"
        elif any(word in reason_lower for word in ("charon", "extract", "translation", "backend", "unsupported", "signature mismatch")):
            kind = "compiler_or_extraction_blocker"
        else:
            kind = "proof_or_specification_blocker"
        root["kind"] = kind
        if root.get("resolution_status") == "resolved_by_receipted_body_property":
            resolved_by_file[source_file].append(root)
            continue
        blocked_by_file[source_file].append(root)
    add_known_report_only_boundaries(functions, blocked_by_file)
    gaps = []
    for source_file, group in sorted(by_file.items()):
        total = len(group)
        transparent = [f for f in group if f["opacity"] == "Transparent"]
        covered = [f for f in transparent if f["status"] in ("proved", "conditional proved")]
        missing = [f for f in transparent if f["status"] == "not yet specified"]
        empty_markers = [f for f in missing if (source_file, f["name"]) in EMPTY_COMPILE_TIME_MARKER_NAMES]
        family_invocation_lines = {
            "src/buf/buf_impl.rs": {2882, 2886},
            "src/buf/buf_mut.rs": {1478, 1482},
        }
        family_member_ids = {
            f["id"] for f in missing
            if f["declaration_kind"] == "source_generated_forwarding_method"
            and f["declared_at"].get("line") in family_invocation_lines.get(source_file, set())
        }
        known_blocked_ids = {
            function_id
            for blocker in blocked_by_file[source_file]
            if blocker.get("resolution_status") in {
                "unresolved_function_body", "unresolved_translation_model_boundary",
                "unresolved_blanket_forwarder", "unresolved_unmapped_blocked_root",
            }
            for function_id in blocker.get("candidate_function_ids", [])
        }
        semantic_missing = [
            f for f in missing
            if f not in empty_markers and f["id"] not in family_member_ids | known_blocked_ids
        ]
        named_compiler_missing_ids = {
            function_id
            for blocker in blocked_by_file[source_file]
            if blocker["kind"] == "compiler_or_extraction_blocker"
            and blocker.get("resolution_status") != "fixed_opaque_boundary"
            and not blocker.get("overlaps_blanket_forwarder_family")
            for function_id in blocker.get("candidate_function_ids", [])
        } & {f["id"] for f in missing}
        model_missing_ids = {
            function_id
            for blocker in blocked_by_file[source_file]
            if blocker["kind"] == "translation_model_boundary"
            and blocker.get("resolution_status") != "fixed_opaque_boundary"
            for function_id in blocker.get("candidate_function_ids", [])
        } & {f["id"] for f in missing}
        family_blocked_missing_ids = family_member_ids
        compiler_blockers = [b for b in blocked_by_file[source_file] if b["kind"] == "compiler_or_extraction_blocker"]
        additional_compiler_blockers = [b for b in compiler_blockers if not b.get("overlaps_blanket_forwarder_family")]
        model_blockers = [b for b in blocked_by_file[source_file] if b["kind"] == "translation_model_boundary"]
        other_blockers = [b for b in blocked_by_file[source_file] if b["kind"] == "proof_or_specification_blocker"]
        opaque_blockers = [b for b in blocked_by_file[source_file] if b["kind"] == "fixed_opaque_boundary"]
        remaining = []
        if semantic_missing:
            remaining.append(f"specification/proof missing for {len(semantic_missing)} semantic transparent function bodies")
        if empty_markers:
            remaining.append(f"{len(empty_markers)} empty compile-time marker bodies have no runtime API behavior")
        if any(f["opacity"] == "Opaque" for f in group):
            remaining.append(f"{sum(f['opacity'] == 'Opaque' for f in group)} source bodies remain at fixed opaque boundaries")
        if named_compiler_missing_ids:
            remaining.append(f"{len(named_compiler_missing_ids)} additional transparent function bodies are compiler/backend blocked")
        if model_missing_ids:
            remaining.append(f"{len(model_missing_ids)} transparent function bodies lack a meaningful Aeneas model")
        forwarder_tool_limits = []
        if source_file in ("src/buf/buf_impl.rs", "src/buf/buf_mut.rs"):
            impl_roots = (
                [(2882, "Buf for &mut T"), (2886, "Buf for Box<T>")]
                if source_file == "src/buf/buf_impl.rs" else
                [(1478, "BufMut for &mut T"), (1482, "BufMut for Box<T>")]
            )
            roots = []
            for invocation_line, impl_name in impl_roots:
                members = [f for f in group if f["declaration_kind"] == "source_generated_forwarding_method"
                           and f["declared_at"].get("line") == invocation_line]
                member_statuses = collections.Counter(f["status"] for f in members if f["opacity"] == "Transparent")
                unproved_members = member_statuses.get("not yet specified", 0)
                roots.append({
                    "implementation": impl_name,
                    "macro_invocation": f"{source_file}:{invocation_line}",
                    "source_function_records": len(members),
                    "transparent_records": sum(f["opacity"] == "Transparent" for f in members),
                    "opaque_records": sum(f["opacity"] == "Opaque" for f in members),
                    "transparent_status_counts": dict(member_statuses),
                    "not_yet_specified_transparent_records": unproved_members,
                })
            if source_file == "src/buf/buf_impl.rs":
                reachability = "current Charon selection rejects these generic blanket impl roots; broad Buf extraction also fails in Aeneas. This is a current tooling limit, not a claim that each forwarding body is unprovable."
                root_state = "individual_root_unreachable_with_current_selection"
                evidence_refs = [
                    {"path": "verification/extensions/readers/manifest.json", "detail": "blocked_roots record the `&mut T` path parse rejection and Box root selection/broad-module failure"},
                    {"path": "evidence/extension-readers-bufmodule/aeneas.log", "lines": [3, 26, 27], "detail": "broad Buf extraction reports mixed recursive declarations and unsupported nested mutable borrows"},
                    {"path": "verification/evidence/generated-build-initial.log", "lines": [2847, 2849], "detail": "whole generated Lean module reports callback-closure state triples where the trait interface expects plain returned state"},
                    {"path": "verification/elicit-integration/ElicitGenerated.failed.log", "lines": [1096, 1099], "detail": "whole generated Lean module cannot elaborate the Buf trait interface/signature"},
                ]
            else:
                reachability = "the exact Box<T>::remaining_mut root and a broader Box<T> forwarding-root selection both appear in LLBC, but Aeneas fails before a complete module is generated: the broad run reports 885 errors (21 unique), including internal translation errors at the &mut T and Box<T> forwarding methods. This is direct evidence of a current Aeneas blocker for the selected family, not a claim that the method bodies are unprovable."
                root_state = "selected_roots_aeneas_blocked"
                evidence_refs = [
                    {"path": "evidence/extension-bufmut-box/charon.exit", "detail": "exact Box<T>::remaining_mut root is successfully extracted"},
                    {"path": "evidence/extension-bufmut-box/aeneas.log", "lines": [205, 6121], "detail": "exact-root translation reports a Box<T>::remaining_mut signature error and emits only partial Lean after 879 errors"},
                    {"path": "evidence/extension-bufmut-box-broad/aeneas.log", "lines": [205, 213, 221, 229, 3167, 3175, 6169], "detail": "broader Box<T> forwarding-root selection reports errors for all four selected methods and both &mut T / Box<T> remaining_mut, then emits only partial Lean after 885 errors"},
                    {"path": "evidence/adapted-filtered/aeneas.log", "lines": [18, 19], "detail": "Aeneas warns that the recursive BufMut trait declaration will be extracted but its model will not type-check"},
                    {"path": "verification/elicit-integration/ElicitGenerated.failed.log", "detail": "whole generated module also fails on the Buf interface before it can establish full crate integration"},
                ]
            forwarder_tool_limits.append({
                "kind": "blanket_forwarder_family_tool_limit",
                "current_tool_state": root_state,
                "summary": reachability,
                "implementation_roots": roots,
                "affected_transparent_functions": sum(r["not_yet_specified_transparent_records"] for r in roots),
                "evidence": evidence_refs,
            })
            affected = sum(r["not_yet_specified_transparent_records"] for r in roots)
            if affected:
                remaining.append(f"blanket forwarding impl extraction/model limits affect {affected} not-yet-specified transparent methods")
        gaps.append({
            "api_family": {
                "src/bytes.rs": "Bytes handle and ownership operations",
                "src/bytes_mut.rs": "BytesMut storage, cursor, allocation, and conversion operations",
                "src/buf/buf_impl.rs": "Buf trait defaults and deref forwarding methods",
                "src/buf/buf_mut.rs": "BufMut trait defaults, writers, and deref forwarding methods",
                "src/buf/chain.rs": "Chain adapter",
                "src/buf/take.rs": "Take adapter",
                "src/buf/limit.rs": "Limit adapter",
                "src/buf/iter.rs": "Buf iterator adapter",
                "src/buf/reader.rs": "Reader adapter",
                "src/buf/writer.rs": "Writer adapter",
                "src/buf/uninit_slice.rs": "UninitSlice and indexing",
                "src/buf/vec_deque.rs": "VecDeque Buf implementation",
                "src/fmt/debug.rs": "Debug formatting",
                "src/fmt/hex.rs": "Hex formatting",
                "src/fmt/mod.rs": "Formatting support and forwarding implementations",
                "src/lib.rs": "crate helpers and TryGetError conversions",
                "src/loom.rs": "Loom compatibility helper",
            }.get(source_file, source_file),
            "source_file": source_file,
            "local_functions": total,
            "public_functions": sum(f["visibility"] == "public" for f in group),
            "source_generated_forwarding_methods": sum(f["declaration_kind"] == "source_generated_forwarding_method" for f in group),
            "other_source_generated_methods": sum(f["declaration_kind"] == "source_generated_method" for f in group),
            "transparent_bodies": len(transparent),
            "opaque_boundaries": sum(f["opacity"] == "Opaque" for f in group),
            "proved_functions": sum(f["status"] == "proved" for f in transparent),
            "conditional_proved_functions": sum(f["status"] == "conditional proved" for f in transparent),
            "transparent_not_yet_specified": len(transparent) - len(covered),
            "semantic_transparent_functions_missing_specification_or_proof": len(semantic_missing),
            "transparent_not_yet_specified_family_tool_blocked": len(family_blocked_missing_ids),
            "transparent_not_yet_specified_named_tool_blocked": len(named_compiler_missing_ids),
            "transparent_not_yet_specified_model_blocked": len(model_missing_ids),
            "empty_compile_time_marker_functions": [
                {"id": f["id"], "rust_path": f["rust_path"], "source_file": f["source_file"], "source_line": f["source_line"]}
                for f in empty_markers
            ],
            "full_contracts_proved": sum(f["full_function_contract_status"] == "proved" for f in group),
            "unproved_generated_forwarders": sum(f["declaration_kind"] == "source_generated_forwarding_method" and f["status"] == "not yet specified" for f in transparent),
            "unproved_other_generated_methods": sum(f["declaration_kind"] == "source_generated_method" and f["status"] == "not yet specified" for f in transparent),
            "compiler_or_extraction_blockers": compiler_blockers,
            "additional_compiler_or_extraction_blockers": additional_compiler_blockers,
            "translation_model_boundaries": model_blockers,
            "other_proof_or_specification_blockers": other_blockers,
            "explicit_fixed_opaque_roots": opaque_blockers,
            "resolved_blocked_roots": resolved_by_file[source_file],
            "family_level_tool_blockers": forwarder_tool_limits,
            "primary_remaining_gap": "; ".join(remaining) if remaining else "no uncovered transparent source body or named blocker in this family",
        })
    for source_file, roots in blocked_by_file.items():
        if source_file not in by_file:
            gaps.append({"source_file": source_file, "compiler_or_extraction_blockers": roots})
    return gaps


def finalize_function_statuses(functions: list[dict[str, Any]], unresolved_audits: list[dict[str, Any]]) -> None:
    unresolved_api = {g.get("api"): g for g in unresolved_audits}
    for fn in functions:
        if not fn["is_source_rust_function"]:
            fn["full_function_contract_status"] = "not applicable: nonfunction LLBC record"
            fn["coverage_scope"] = fn["status_reason"]
            continue
        full_contracts = [e for e in fn["evidence"] if e.get("tier") == "full_contract"]
        fn["full_function_contract_status"] = "proved" if full_contracts else (
            "blocked_by_opaque_boundary" if fn["opacity"] == "Opaque" else "not audited"
        )
        if full_contracts:
            fn["coverage_scope"] = "full Rust function contract"
        elif fn["evidence"]:
            fn["coverage_scope"] = "one or more mapped body properties; full function contract not established"
        elif fn["opacity"] == "Opaque":
            fn["coverage_scope"] = "implementation body is behind a fixed opaque boundary"
        else:
            fn["coverage_scope"] = "no mapped, receipted specification or proof"
        for api, record in unresolved_api.items():
            if api == "Bytes::len" and fn["source_file"] == "src/bytes.rs" and fn["name"] == "len":
                fn["full_function_contract_status"] = "unresolved_contract_mapping"
                fn["unresolved_contract_reason"] = record.get("summary")
            elif api == "BytesMut::len" and fn["source_file"] == "src/bytes_mut.rs" and fn["name"] == "len":
                fn["full_function_contract_status"] = "unresolved_contract_mapping"
                fn["unresolved_contract_reason"] = record.get("summary")


def opaque_parity(llbc: dict[str, Any], baseline: dict[str, Any]) -> dict[str, Any]:
    translated = llbc["translated"]
    actual = []
    for fn in translated.get("fun_decls", []):
        if not isinstance(fn, dict):
            continue
        meta = fn.get("item_meta") or {}
        if meta.get("is_local") and meta.get("opacity") == "Opaque":
            actual.append({"name": meta.get("name"), "signature": fn.get("signature"), "source": meta.get("span")})
    key = lambda x: (json.dumps(x.get("name"), sort_keys=True), json.dumps(x.get("signature"), sort_keys=True), json.dumps(x.get("source"), sort_keys=True))
    actual_set = {key(x) for x in actual}
    frozen_set = {key(x) for x in baseline.get("opaque_functions", [])}
    metadata_path = ROOT / "adapted-opaque-functions.json"
    current_metadata = read_json(metadata_path) if metadata_path.exists() else []
    current_by_id = {x.get("id"): x for x in current_metadata}
    current_matches = 0
    for fn in translated.get("fun_decls", []):
        if not isinstance(fn, dict):
            continue
        meta = fn.get("item_meta") or {}
        if not (meta.get("is_local") and meta.get("opacity") == "Opaque"):
            continue
        item = current_by_id.get(fn.get("def_id"))
        if item and item.get("name") == meta.get("name") and item.get("span") == meta.get("span"):
            current_matches += 1
    patterns_path = ROOT / "adapted-opacity.json"
    current_patterns = read_json(patterns_path) if patterns_path.exists() else []
    frozen_patterns = baseline.get("opaque_patterns", [])
    return {
        "current_opaque_local_functions": len(actual),
        "frozen_opaque_local_functions": baseline.get("opaque_local_function_count"),
        "exact_name_signature_source_match": actual_set == frozen_set,
        "exact_function_records_match_count": current_matches,
        "current_metadata_function_count": len(current_metadata),
        "current_opacity_pattern_count": len(current_patterns),
        "frozen_opacity_pattern_count": len(frozen_patterns),
        "exact_frozen_opacity_pattern_set_match": set(current_patterns) == set(frozen_patterns),
    }


def gap_partition(functions: list[dict[str, Any]], gaps: list[dict[str, Any]]) -> dict[str, Any]:
    totals = collections.Counter()
    marker_functions = []
    for gap in gaps:
        totals["blanket_forwarder_tool_blocked"] += gap.get("transparent_not_yet_specified_family_tool_blocked", 0)
        totals["named_tool_or_backend_blocked"] += gap.get("transparent_not_yet_specified_named_tool_blocked", 0)
        totals["translation_model_blocked"] += gap.get("transparent_not_yet_specified_model_blocked", 0)
        totals["missing_specification_or_proof"] += gap.get("semantic_transparent_functions_missing_specification_or_proof", 0)
        markers = gap.get("empty_compile_time_marker_functions", [])
        totals["empty_compile_time_markers"] += len(markers)
        marker_functions.extend(markers)
    total_missing = sum(
        fn["opacity"] == "Transparent" and fn["is_source_rust_function"] and fn["status"] == "not yet specified"
        for fn in functions
    )
    partition_total = sum(totals.values())
    return {
        "not_yet_specified_transparent_functions": total_missing,
        **dict(totals),
        "partition_total": partition_total,
        "reconciles": partition_total == total_missing,
        "empty_compile_time_marker_functions": marker_functions,
        "interpretation": "Functions are counted once in mutually exclusive buckets: blanket forwarders, named tool/backend limits, Aeneas model limits, missing specifications/proofs, or empty compile-time markers.",
    }


def main() -> None:
    llbc = read_json(LLBC)
    functions, counts = local_inventory(llbc)
    native_maps, native_unmapped = load_native_evidence(functions)
    extension_maps, blocked_roots, extension_unmapped, extension_receipt_statuses = load_extension_evidence(functions)
    blocked_roots = [resolve_blocked_root(functions, root) for root in blocked_roots]

    by_file: dict[str, dict[str, Any]] = {}
    for fn in functions:
        file = fn.get("source_file") or "unknown"
        group = by_file.setdefault(file, {
            "local_functions": 0,
            "public_functions": 0,
            "source_generated_forwarding_methods": 0,
            "other_source_generated_methods": 0,
            "transparent": 0,
            "opaque": 0,
            "proved": 0,
            "conditional_proved": 0,
            "blocked": 0,
            "not_yet_specified": 0,
        })
        if not fn["is_source_rust_function"]:
            nonfunction_records = group.setdefault("nonfunction_llbc_fun_records", collections.Counter())
            nonfunction_records[fn["declaration_kind"]] += 1
            continue
        group["local_functions"] += 1
        group["public_functions"] += fn["visibility"] == "public"
        group["source_generated_forwarding_methods"] += fn["declaration_kind"] == "source_generated_forwarding_method"
        group["other_source_generated_methods"] += fn["declaration_kind"] == "source_generated_method"
        group["transparent"] += fn["opacity"] == "Transparent"
        group["opaque"] += fn["opacity"] == "Opaque"
        key = fn["status"].replace(" ", "_")
        if key in group:
            group[key] += 1

    opaque_baseline = read_json(VERIFY / "opaque-baseline.json")
    opaque_check = opaque_parity(llbc, opaque_baseline)
    source_provenance = read_json(ROOT / "source-provenance.json")
    # Only paths belonging to adapted Rust source are summarized here; Cargo metadata
    # edits do not change function source correspondence.
    source_patch = ROOT / "adapted-source.patch"
    edited_rs = sorted(set(re.findall(r"\+\+\+ adapted/(src/[^\n]+)", source_patch.read_text()))) if source_patch.exists() else []
    native_theorem_total = len({m["theorem"] for m in native_maps if m.get("mapped") and m.get("group") != "strict_elicit_audit"})
    extension_theorem_total = len({m["theorem"] for m in extension_maps if m.get("mapped")})
    audit_success_count = sum(m.get("group") == "strict_elicit_audit" and m.get("mapped") for m in native_maps)
    result_manifest_path = VERIFY / "results.json"
    result_manifest = read_json(result_manifest_path) if result_manifest_path.exists() else {}
    unresolved_audits = [goal for goal in (result_manifest.get("cargo_elicit", {}).get("goals") or []) if goal.get("outcome") == "unresolved"]
    finalize_function_statuses(functions, unresolved_audits)
    actionable_gaps = build_gaps(functions, blocked_roots)
    gap_partition_summary = gap_partition(functions, actionable_gaps)
    function_status_counts = collections.Counter(f["status"] for f in functions if f["is_source_rust_function"])
    nonfunction_record_counts = collections.Counter(f["declaration_kind"] for f in functions if not f["is_source_rust_function"])
    pipeline_blockers = [{
        "kind": "whole_crate_compiler_integration",
        "status": "blocked",
        "scope": "the full emitted crate module",
        "reason": "the whole emitted crate module does not type-check; selected API-scoped generated modules and proof files are compiled independently",
        "evidence": "verification/elicit-integration/check-whole-crate-failed.log",
    }]
    pipeline_blockers.extend({
        "kind": "full_api_contract_unresolved",
        "status": "unresolved_contract_mapping",
        "target": goal.get("target"),
        "api": goal.get("api"),
        "reason": goal.get("summary"),
        "axioms": goal.get("axioms"),
    } for goal in unresolved_audits)
    data = {
        "schema": "bytes-coverage-ledger-v1",
        "target": {
            "crate": "bytes",
            "version": "1.11.1",
            "translated_input": "evidence/adapted-whole-final/crate.llbc",
            "feature_set": ["default", "std"],
            "target": "x86_64-unknown-linux-gnu",
        },
        "source_correspondence": {
            "upstream_archive_sha256": source_provenance.get("archive_sha256"),
            "source_file_count": source_provenance.get("src_file_count"),
            "snapshot_identical_to_archive": source_provenance.get("src_identical_to_archive"),
            "adapted_rust_files_with_changes": edited_rs,
            "adapted_source_patch": "adapted-source.patch",
            "selected_native_and_audited_method_bodies_unchanged": True,
            "basis": "the recorded patch changes a function-pointer wrapper type and two generic bounds; the selected len/emptiness, Take, Chain and extension methods are unchanged",
        },
        "inventory_counts": {
            **counts,
            "opaque_baseline_pattern_count": len(opaque_baseline.get("opaque_patterns", [])),
            "opaque_baseline_function_count": opaque_baseline.get("opaque_local_function_count"),
            "matches_frozen_opaque_function_count": counts["local_opacity"].get("Opaque", 0) == opaque_baseline.get("opaque_local_function_count"),
            "opaque_boundary_parity": opaque_check,
            "transparent_body_function_count": counts["local_opacity"].get("Transparent", 0),
            "source_generated_forwarding_methods": counts["local_declaration_kind"].get("source_generated_forwarding_method", 0),
            "other_source_generated_methods": counts["local_declaration_kind"].get("source_generated_method", 0),
        },
        "coverage_receipts": {
            "native_receipted_body_theorems_mapped_to_rust_functions": native_theorem_total,
            "strict_api_audit_successes_mapped": audit_success_count,
            "extension_receipted_theorems_mapped_to_rust_functions": extension_theorem_total,
            "extension_receipt_validation": extension_receipt_statuses,
            "unmapped_native_claims": native_unmapped,
            "unmapped_extension_claims": extension_unmapped,
            "extension_blocked_roots": blocked_roots,
            "unique_rust_functions_with_receipted_coverage": sum(f["status"] in ("proved", "conditional proved") for f in functions),
        },
        "pipeline_blockers": pipeline_blockers,
        "coverage_status_counts": dict(function_status_counts),
        "nonfunction_llbc_fun_record_counts": dict(nonfunction_record_counts),
        "coverage_by_source_file": by_file,
        "actionable_gaps": actionable_gaps,
        "transparent_gap_partition": gap_partition_summary,
        "functions": sorted(functions, key=lambda fn: (fn.get("source_file") or "", fn.get("source_line") or 0, fn.get("llbc_def_id") or 0)),
    }
    output = OUT / "ledger.json"
    output.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    summary = {
        "ledger": "ledger.json",
        "local_functions": counts["local_source_rust_functions"],
        "local_llbc_function_records": counts["local_functions"],
        "transparent": counts["local_opacity"].get("Transparent", 0),
        "opaque": counts["local_opacity"].get("Opaque", 0),
        "source_function_transparent": counts["source_function_opacity"].get("Transparent", 0),
        "source_function_opaque": counts["source_function_opacity"].get("Opaque", 0),
        "source_generated_forwarding_methods": counts["local_declaration_kind"].get("source_generated_forwarding_method", 0),
        "proved": sum(f["status"] == "proved" for f in functions if f["is_source_rust_function"]),
        "conditional_proved": sum(f["status"] == "conditional proved" for f in functions if f["is_source_rust_function"]),
        "blocked": sum(f["status"] == "blocked" for f in functions if f["is_source_rust_function"]),
        "not_yet_specified": sum(f["status"] == "not yet specified" for f in functions if f["is_source_rust_function"]),
        "mapped_native_theorems": native_theorem_total,
        "strict_api_audit_successes": audit_success_count,
        "mapped_extension_theorems": extension_theorem_total,
        "transparent_gap_partition": gap_partition_summary,
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
