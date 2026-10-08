#!/usr/bin/env python3
"""Narrow source/MIR Drop correspondence elaborator for the pinned probe.

This intentionally recognizes only the three frozen straight-line examples in
native.rs.  It is an external elaborator, not a proof of Rust's general drop
semantics.  The generated mapping is a receipt; the independent checker must
re-derive every edge and call correspondence from the raw inputs.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "native.rs"
CANONICAL_MIR = ROOT / "native-mir"
CANONICAL_GENERATED = ROOT / "generated"
FEATURES = {
    "omit_drop_call",
    "duplicate_drop_call",
    "wrong_drop_target",
    "wrong_drop_summary",
    "early_drop_call",
}
RUSTC_RELEASE = "rustc 1.98.0-nightly (91fe22da8 2026-06-21)"
RUSTC_COMMIT = "91fe22da8084a1c9e993d78d4a56f22ab8396236"
MIR_STAGE = "2-2-004.ElaborateDrops.after.mir"
MIR_PREFIX = "generic_drop_native"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_text(text: str) -> str:
    return sha256_bytes(text.encode("utf-8"))


def compact(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def line_col(text: str, offset: int) -> tuple[int, int]:
    line = text.count("\n", 0, offset) + 1
    last = text.rfind("\n", 0, offset)
    return line, offset - last


def find_braced_item(source: str, marker: str) -> tuple[int, int, int, str]:
    """Return marker start, opening brace, end offset, and complete item text."""
    start = source.find(marker)
    if start < 0:
        raise ValueError(f"missing source item marker: {marker}")
    brace = source.find("{", start + len(marker))
    if brace < 0:
        raise ValueError(f"missing body for source item: {marker}")
    depth = 0
    in_string = False
    escaped = False
    line_comment = False
    block_depth = 0
    i = brace
    while i < len(source):
        ch = source[i]
        nxt = source[i + 1] if i + 1 < len(source) else ""
        if line_comment:
            if ch == "\n":
                line_comment = False
            i += 1
            continue
        if block_depth:
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                i += 2
                continue
            i += 1
            continue
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            i += 1
            continue
        if ch == "/" and nxt == "/":
            line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            i += 2
            continue
        if ch == '"':
            in_string = True
            i += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                return start, brace, end, source[start:end]
        i += 1
    raise ValueError(f"unterminated source item: {marker}")


@dataclass(frozen=True)
class SourceItem:
    marker: str
    start: int
    open_brace: int
    end: int
    text: str

    @property
    def body(self) -> str:
        return self.text[self.open_brace - self.start + 1 : -1]


def item(source: str, marker: str) -> SourceItem:
    return SourceItem(marker, *find_braced_item(source, marker))


@dataclass(frozen=True)
class EffectSpec:
    scope: str
    type_name: str
    source_binding: str
    helper: str
    drop_marker: str
    expected_drop_body: str
    helper_body: str
    value_contract: str
    expected_mir_statement: str
    source_body: str


EFFECTS = (
    EffectSpec(
        scope="set_true_scope",
        type_name="SetTrueOnDrop",
        source_binding="_guard",
        helper="set_true_drop_effect",
        drop_marker="impl Drop for SetTrueOnDrop<'_>",
        expected_drop_body="fn drop(&mut self) { set_true(self.0); }",
        helper_body="set_true(guard.0);",
        value_contract="#[ensures(*(^guard).0 == true)]",
        expected_mir_statement="_0 = const ();",
        source_body="let _guard = SetTrueOnDrop(flag);",
    ),
    EffectSpec(
        scope="toggle_scope",
        type_name="ToggleOnDrop",
        source_binding="_guard",
        helper="toggle_drop_effect",
        drop_marker="impl Drop for ToggleOnDrop<'_>",
        expected_drop_body="fn drop(&mut self) { *self.0 = !*self.0; }",
        helper_body="*guard.0 = !*guard.0;",
        value_contract="#[ensures(*(^guard).0 == !*guard.0)]",
        expected_mir_statement="_0 = const ();",
        source_body="let _guard = ToggleOnDrop(flag);",
    ),
    EffectSpec(
        scope="toggle_after_write",
        type_name="ToggleOnDrop",
        source_binding="guard",
        helper="toggle_drop_effect",
        drop_marker="impl Drop for ToggleOnDrop<'_>",
        expected_drop_body="fn drop(&mut self) { *self.0 = !*self.0; }",
        helper_body="*guard.0 = !*guard.0;",
        value_contract="#[ensures(*(^guard).0 == !*guard.0)]",
        expected_mir_statement="(*_4) = const true;",
        source_body="let guard = ToggleOnDrop(flag);\n    *guard.0 = true;",
    ),
)


def validate_source(source: str) -> tuple[dict[str, SourceItem], dict[str, SourceItem]]:
    """Reject anything except the exact closed source language of this probe."""
    forbidden = (
        "unsafe",
        "std::thread",
        "thread::",
        "spawn(",
        "mem::forget",
        "ManuallyDrop",
        "Box<",
        "impl Send",
        "impl Sync",
        "catch_unwind",
    )
    for token in forbidden:
        if token in source:
            raise ValueError(f"unsupported source construct: {token}")

    drop_headers = re.findall(r"impl\s+Drop\s+for\s+[^\{]+\{", source)
    if len(drop_headers) != 2:
        raise ValueError(f"expected exactly two Drop impls, found {len(drop_headers)}")

    drop_items: dict[str, SourceItem] = {}
    scope_items: dict[str, SourceItem] = {}
    for spec in EFFECTS:
        drop_item = item(source, spec.drop_marker)
        # Check the complete impl body, not just the selected method body.  This
        # rejects independent field destructors and additional effects.
        if compact(drop_item.body) != compact(spec.expected_drop_body):
            raise ValueError(
                f"unsupported destructor body for {spec.type_name}: {drop_item.body!r}"
            )
        drop_items[spec.type_name] = drop_item

        scope_item = item(source, f"pub fn {spec.scope}(")
        if compact(scope_item.body) != compact(spec.source_body):
            raise ValueError(
                f"unsupported body for {spec.scope}: {scope_item.body!r}"
            )
        scope_items[spec.scope] = scope_item

    if len(re.findall(r"pub\s+fn\s+(?:set_true_scope|toggle_scope|toggle_after_write)\s*\(", source)) != 3:
        raise ValueError("scope function set does not match the closed probe")
    if "pub struct SetTrueOnDrop<'a>(pub &'a mut bool);" not in source:
        raise ValueError("SetTrueOnDrop field layout changed")
    if "pub struct ToggleOnDrop<'a>(pub &'a mut bool);" not in source:
        raise ValueError("ToggleOnDrop field layout changed")
    set_true = item(source, "fn set_true(")
    if compact(set_true.body) != "*flag = true;":
        raise ValueError(f"unsupported set_true body: {set_true.body!r}")
    return drop_items, scope_items


def erase_spec_attributes(source: str) -> tuple[str, list[dict[str, object]]]:
    """Erase only exact Creusot-only syntax before direct pinned-rustc build."""
    allowed = {
        "use creusot_std::prelude::*;",
        "#[ensures(^flag == true)]",
        "#[requires(*flag == false)]",
        "#[ensures(^flag == !*flag)]",
        "#[ensures(^flag == false)]",
    }
    removed: list[dict[str, object]] = []
    output: list[str] = []
    for line_no, line in enumerate(source.splitlines(keepends=True), start=1):
        stripped = line.strip()
        if stripped.startswith("#[ensures") or stripped.startswith("#[requires") or stripped.startswith("use creusot_std"):
            if stripped not in allowed:
                raise ValueError(f"unrecognized verifier-only source syntax on line {line_no}: {stripped}")
            removed.append({"line": line_no, "text": stripped})
            output.append("\n" if line.endswith("\n") else "")
        else:
            output.append(line)
    return "".join(output), removed


def extract_native_body(mir: str, scope: str, expected_type: str, binding: str) -> dict[str, object]:
    header = f"fn {scope}(_1: &mut bool) -> () {{"
    start = mir.find(header)
    if start < 0:
        raise ValueError(f"MIR header is missing or changed: {header}")
    open_brace = mir.find("{", start)
    depth = 0
    end = -1
    for pos in range(open_brace, len(mir)):
        if mir[pos] == "{":
            depth += 1
        elif mir[pos] == "}":
            depth -= 1
            if depth == 0:
                end = pos + 1
                break
    if end < 0:
        raise ValueError(f"unterminated MIR body for {scope}")
    body = mir[open_brace + 1 : end - 1]
    if compact(body) != compact(expected_mir_body(scope, expected_type, binding)):
        raise ValueError(f"unsupported MIR statements or CFG in {scope}")
    if "switchInt(" in body or "goto ->" in body or "assert(" in body or "unwind terminate" in body:
        raise ValueError(f"unsupported control flow in MIR body for {scope}")
    if re.search(r"\bcall\b|\bFn\(|\bdrop\([^)]*\) -> \[return: bb\d+, unwind: bb\d+\];", body) is None:
        raise ValueError(f"MIR Drop terminator is missing for {scope}")
    drops = list(re.finditer(r"drop\(([^\n]+?)\) -> \[return: bb(\d+), unwind: bb(\d+)\];", body))
    if len(drops) != 1:
        raise ValueError(f"expected one Drop terminator in {scope}, found {len(drops)}")
    drop = drops[0]
    raw_place = drop.group(1)
    if raw_place != "_2":
        raise ValueError(f"wrong Drop place in {scope}: {raw_place!r}")
    declaration = re.search(rf"^\s*let(?: mut)? _2: ({re.escape(expected_type)}<'_>);$", body, flags=re.MULTILINE)
    if declaration is None:
        raise ValueError(f"Drop local _2 has the wrong or missing type in {scope}")
    normal_target, unwind_target = drop.group(2), drop.group(3)

    debug = re.findall(r"debug ([A-Za-z_][A-Za-z0-9_]*) => (_\d+);", body)
    source_local = next((name for name, local in debug if local == "_2"), None)
    if source_local != binding:
        raise ValueError(f"MIR local `_2` maps to {source_local!r}, expected {binding!r}")

    blocks = re.findall(r"^\s*(bb\d+)(?: \(cleanup\))?: \{", body, flags=re.MULTILINE)
    if blocks != ["bb0", "bb1", "bb2"]:
        raise ValueError(f"unsupported CFG in {scope}: {blocks!r}")
    expected_targets = ("1", "2")
    if (normal_target, unwind_target) != expected_targets:
        raise ValueError(f"unsupported normal/unwind edge in {scope}: {normal_target}/{unwind_target}")
    if not re.search(r"bb1: \{\s*StorageDead\(_2\);\s*return;\s*\}", body):
        raise ValueError(f"normal successor of {scope} is not a plain return")
    if not re.search(r"bb2 \(cleanup\): \{\s*resume;\s*\}", body):
        raise ValueError(f"unwind successor of {scope} is not a recorded cleanup resume")
    if body.count("resume;") != 1:
        raise ValueError(f"unexpected cleanup effects in {scope}")
    # Native MIR assignment differs only in rustc's spacing between place and =.
    expected_stmt = compact(expected_mir_statement(scope))
    if expected_stmt not in compact(body):
        raise ValueError(f"expected source operation absent from MIR for {scope}")
    # This white-list also prevents unnoticed extra calls, field drops, moves or
    # escaping values in the source function's MIR.
    permitted = {
        "set_true_scope": ("StorageLive(_2);", "StorageLive(_3);", "_3 = &'_ mut (*_1);", "_2 = SetTrueOnDrop::<'_>(move _3);", "StorageDead(_3);", "_0 = const ();"),
        "toggle_scope": ("StorageLive(_2);", "StorageLive(_3);", "_3 = &'_ mut (*_1);", "_2 = ToggleOnDrop::<'_>(move _3);", "StorageDead(_3);", "_0 = const ();"),
        "toggle_after_write": ("StorageLive(_2);", "StorageLive(_3);", "_3 = &'_ mut (*_1);", "_2 = ToggleOnDrop::<'_>(move _3);", "StorageDead(_3);", "_4 = deref_copy (_2.0: &mut bool);", "(*_4) = const true;", "_0 = const ();"),
    }[scope]
    normalized_body = compact(body)
    for statement in permitted:
        if compact(statement) not in normalized_body:
            raise ValueError(f"MIR operation missing in {scope}: {statement}")
    if re.search(r"\bcall\b|\bmove _[0-9]+\.0\b|drop\(_[0-9]+\.\d+", body):
        raise ValueError(f"unsupported call/move/field-drop in {scope}")

    # Extract the terminator's containing basic block (currently bb0), byte
    # span and text for the independent correspondence checker.
    local_offset = body.find(drop.group(0))
    preceding_blocks = list(re.finditer(r"^\s*(bb\d+)(?: \(cleanup\))?: \{", body[:local_offset], flags=re.MULTILINE))
    if not preceding_blocks:
        raise ValueError(f"Drop terminator has no basic block in {scope}")
    bb = preceding_blocks[-1].group(1)
    return {
        "raw_body": mir[start:end],
        "body_sha256": sha256_text(mir[start:end]),
        "drop": {
            "basic_block": bb,
            "place": "_2",
            "type": declaration.group(1),
            "terminator": drop.group(0),
            "normal_target": f"bb{normal_target}",
            "unwind_target": f"bb{unwind_target}",
            "unwind_interpretation": "recorded cleanup edge; not elaborated into a proof effect",
            "order_in_scope": 0,
        },
        "source_local": source_local,
    }


def expected_mir_statement(scope: str) -> str:
    if scope == "toggle_after_write":
        return "(*_4) = const true;"
    return "_0 = const ();"


def expected_mir_body(scope: str, type_name: str, binding: str) -> str:
    """Pinned MIR whitelist: reject every unrecognized operation or edge."""
    before_drop = [
        "debug flag => _1;",
        "let mut _0: ();",
        f"let _2: {type_name}<'_>;",
        "let mut _3: &mut bool;",
    ]
    if scope == "toggle_after_write":
        before_drop.append("let mut _4: &mut bool;")
    before_drop.extend(
        [
            "scope 1 {",
            f"debug {binding} => _2;",
            "}",
            "bb0: {",
            "StorageLive(_2);",
            "StorageLive(_3);",
            "_3 = &'_ mut (*_1);",
            f"_2 = {type_name}::<'_>(move _3);",
            "StorageDead(_3);",
        ]
    )
    if scope == "toggle_after_write":
        before_drop.extend(["_4 = deref_copy (_2.0: &mut bool);", "(*_4) = const true;"])
    before_drop.extend(
        [
            "_0 = const ();",
            "drop(_2) -> [return: bb1, unwind: bb2];",
            "}",
            "bb1: {",
            "StorageDead(_2);",
            "return;",
            "}",
            "bb2 (cleanup): {",
            "resume;",
            "}",
        ]
    )
    return "\n".join(before_drop)


def generated_helper(spec: EffectSpec, wrong_summary: bool) -> str:
    if spec.type_name == "SetTrueOnDrop":
        bad_contract = "#[ensures(*(^guard).0 == false)]"
    else:
        bad_contract = "#[ensures(*(^guard).0 == *guard.0)]"
    contract = bad_contract if wrong_summary else spec.value_contract
    return "\n".join(
        [
            "#[ensures(^(guard.0) == ^((^guard).0))]",
            contract,
            f"pub(crate) fn {spec.helper}<'a>(guard: &mut {spec.type_name}<'a>) {{",
            f"    {spec.helper_body}",
            "}",
        ]
    )


def call_text(spec: EffectSpec, features: set[str], indent: str = "    ") -> str:
    if "omit_drop_call" in features:
        return ""
    if "wrong_drop_target" in features and spec.type_name == "ToggleOnDrop":
        return "\n".join(
            [
                f"{indent}let mut wrong_target_value = false;",
                f"{indent}let mut wrong_target_guard = ToggleOnDrop(&mut wrong_target_value);",
                f"{indent}{spec.helper}(&mut wrong_target_guard);",
            ]
        )
    call = f"{spec.helper}(&mut {spec.source_binding});"
    if "duplicate_drop_call" in features and spec.type_name == "ToggleOnDrop":
        return f"{indent}{call}\n{indent}{call}"
    return f"{indent}{call}"


def generated_scope_body(spec: EffectSpec, features: set[str]) -> str:
    binding = spec.source_binding
    if spec.scope == "set_true_scope":
        lines = [f"    let mut {binding} = SetTrueOnDrop(flag);"]
        lines.append(call_text(spec, features))
        return "\n".join(line for line in lines if line != "")
    if spec.scope == "toggle_scope":
        lines = [f"    let mut {binding} = ToggleOnDrop(flag);"]
        lines.append(call_text(spec, features))
        return "\n".join(line for line in lines if line != "")
    if spec.scope == "toggle_after_write":
        guard = "guard"
        setup = [f"    let mut {guard} = ToggleOnDrop(flag);", f"    *{guard}.0 = true;"]
        if "early_drop_call" in features:
            setup = [
                f"    let mut {guard} = ToggleOnDrop(flag);",
                call_text(spec, features),
                f"    *{guard}.0 = true;",
            ]
        else:
            setup.append(call_text(spec, features))
        return "\n".join(line for line in setup if line != "")
    raise AssertionError(spec.scope)


def replace_body(text: str, begin: int, open_brace: int, end: int, body: str) -> str:
    relative_open = open_brace - begin
    relative_end = end - begin
    # Scope bodies in this probe have no nested syntactic item boundary; the
    # brace scanner found the exact outer close brace.
    return text[begin : begin + relative_open + 1] + "\n" + body + "\n}" + text[begin + relative_end : end]


def generate_shadow(source: str, drop_items: dict[str, SourceItem], scope_items: dict[str, SourceItem], features: set[str]) -> tuple[str, dict[str, dict[str, object]]]:
    # Remove only the two native Drop impls and the native-only test module.
    edits: list[tuple[int, int, str]] = [(x.start, x.end, "") for x in drop_items.values()]
    test_match = re.search(r"#\[cfg\(test\)\]\s*mod tests\s*\{", source)
    if not test_match:
        raise ValueError("native test module marker is missing")
    test_item = item(source, "mod tests")
    edits.append((test_match.start(), test_item.end, ""))

    # Apply scope body replacements against the untouched source before item
    # removals, then remove drop impls and tests in descending offset order.
    scope_edits: list[tuple[int, int, str]] = []
    for spec in EFFECTS:
        scope = scope_items[spec.scope]
        replacement = replace_body(
            source,
            scope.start,
            scope.open_brace,
            scope.end,
            generated_scope_body(spec, features),
        )
        scope_edits.append((scope.start, scope.end, replacement))

    current = source
    # Replacements preserve source offsets until all are applied in descending
    # order. Scope declarations do not overlap impls or tests.
    for start, end, replacement in sorted(scope_edits, reverse=True):
        current = current[:start] + replacement + current[end:]
    # Drop/test offsets need translation after scope edits. The replacements
    # above change lengths, so re-find removable items against the new text.
    for marker in ("impl Drop for SetTrueOnDrop<'_>", "impl Drop for ToggleOnDrop<'_>"):
        drop = item(current, marker)
        current = current[: drop.start] + current[drop.end :]
    test_match = re.search(r"#\[cfg\(test\)\]\s*mod tests\s*\{", current)
    if not test_match:
        raise ValueError("test module disappeared during source rewrite")
    test = item(current, "mod tests")
    current = current[: test_match.start()] + current[test.end :]

    helper_blocks: list[str] = []
    seen: set[str] = set()
    helper_specs = {spec.helper: spec for spec in EFFECTS}
    for spec in EFFECTS:
        if spec.helper in seen:
            continue
        seen.add(spec.helper)
        helper_blocks.append(generated_helper(spec, "wrong_drop_summary" in features))

    # Place helpers immediately after set_true, whose independently specified
    # body is the only direct call in one copied destructor.
    set_true = item(current, "fn set_true(")
    insertion = set_true.end
    helpers = "\n\n" + "\n\n".join(helper_blocks) + "\n"
    current = current[:insertion] + helpers + current[insertion:]

    helper_metadata: dict[str, dict[str, object]] = {}
    for helper_name, helper_spec in helper_specs.items():
        helper_item = item(current, f"pub(crate) fn {helper_name}<")
        helper_metadata[helper_name] = {
            "span_bytes": [helper_item.start, helper_item.end],
            "span_lines": [line_col(current, helper_item.start)[0], line_col(current, helper_item.end)[0]],
            "item_sha256": sha256_text(helper_item.text),
            "body": helper_item.body,
            "body_sha256": sha256_text(helper_item.body),
            "copied_from_drop_body": helper_spec.expected_drop_body,
            "body_transformation": "substitute receiver self with helper parameter guard; no statement change",
            "contracts": {
                "frame": "#[ensures(^(guard.0) == ^((^guard).0))]",
                "value": (
                    "#[ensures(*(^guard).0 == false)]"
                    if "wrong_drop_summary" in features and helper_spec.type_name == "SetTrueOnDrop"
                    else "#[ensures(*(^guard).0 == *guard.0)]"
                    if "wrong_drop_summary" in features
                    else helper_spec.value_contract
                ),
                "positive_value": helper_spec.value_contract,
                "wrong_summary_control": "old-value/no-effect summary; expected to fail helper body VC",
            },
        }
    return current, helper_metadata


def prepare_native_rustc_input(output: Path) -> dict[str, object]:
    source = SOURCE.read_text(encoding="utf-8")
    validate_source(source)
    native, removed = erase_spec_attributes(source)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(native, encoding="utf-8")
    return {
        "source_path": "native.rs",
        "source_sha256": sha256_text(source),
        "rustc_input_path": "native-mir/native-rustc-input.rs",
        "rustc_input_sha256": sha256_text(native),
        "erased_spec_syntax": removed,
        "executable_source_preserved": True,
    }


def load_effect_mir(mir_dir: Path, effect: EffectSpec) -> tuple[Path, str, dict[str, object]]:
    filename = f"{MIR_PREFIX}.{effect.scope}.{MIR_STAGE}"
    path = mir_dir / filename
    if not path.is_file():
        raise ValueError(f"missing pinned post-ElaborateDrops MIR artifact: {path}")
    content = path.read_text(encoding="utf-8")
    parsed = extract_native_body(content, effect.scope, effect.type_name, effect.source_binding)
    return path, content, parsed


def effect_mapping(source: str, drop_items: dict[str, SourceItem], scope_items: dict[str, SourceItem], helper_metadata: dict[str, dict[str, object]], mir_dir: Path, features: set[str], helper_path: str) -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    # The generated helper file is inspected later for call spans; this list
    # carries source-side identities and raw MIR body text as checker input.
    for order, spec in enumerate(EFFECTS):
        drop_item = drop_items[spec.type_name]
        scope_item = scope_items[spec.scope]
        mir_path, mir_text, parsed = load_effect_mir(mir_dir, spec)
        drop_line, _ = line_col(source, drop_item.start)
        scope_line, _ = line_col(source, scope_item.start)
        drop_method_start = drop_item.text.find("fn drop")
        source_drop_body = drop_item.body
        helper = helper_metadata[spec.helper]
        result.append(
            {
                "order": order,
                "scope": spec.scope,
                "source_scope": {
                    "signature": f"pub fn {spec.scope}",
                    "start_line": scope_line,
                    "body": scope_item.body,
                    "body_sha256": sha256_text(scope_item.body),
                    "source_binding": spec.source_binding,
                },
                "native_mir": {
                    "path": f"native-mir/{mir_path.name}",
                    "sha256": sha256_text(mir_text),
                    "body": parsed["raw_body"],
                    "body_sha256": parsed["body_sha256"],
                    "debug_local_source_name": parsed["source_local"],
                    "drop": parsed["drop"],
                },
                "source_destructor": {
                    "type": spec.type_name,
                    "impl_marker": spec.drop_marker,
                    "start_line": drop_line,
                    "body": source_drop_body,
                    "body_sha256": sha256_text(source_drop_body),
                    "method_body": spec.expected_drop_body,
                    "method_body_sha256": sha256_text(spec.expected_drop_body),
                    "method_start_byte": drop_item.start + drop_method_start,
                },
                "shadow_helper": {
                    "name": spec.helper,
                    "path": helper_path,
                    **helper,
                },
                "injection": {
                    "normal_edge": parsed["drop"]["normal_target"],
                    "unwind_edge": parsed["drop"]["unwind_target"],
                    "effect_order": 0,
                    "expected_call": f"{spec.helper}(&mut {spec.source_binding});",
                    "mode": "normal-return only; unwind edge recorded and not claimed",
                    "proof_only_binding_adjustment": {
                        "local": spec.source_binding,
                        "native_binding_mutable": False,
                        "shadow_binding_mutable": True,
                        "native_source_changed": False,
                    },
                    "active_diagnostic_features": sorted(features),
                },
            }
        )
    return result


def parse_features(raw: str | None) -> set[str]:
    if not raw:
        return set()
    features = {x.strip() for x in raw.split(",") if x.strip()}
    unknown = features - FEATURES
    if unknown:
        raise ValueError(f"unknown diagnostic feature(s): {', '.join(sorted(unknown))}")
    if len(features) > 1:
        raise ValueError("diagnostic variants are isolated: request one feature per run")
    return features


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--features", help="one diagnostic control feature; comma syntax is accepted only for one name")
    parser.add_argument("--mir-dir", type=Path, default=CANONICAL_MIR)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--prepare-native-rustc-input", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--output", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--rustc-version", default=RUSTC_RELEASE)
    parser.add_argument("--rustc-commit", default=RUSTC_COMMIT)
    parser.add_argument("--rustc-flags", default="--edition=2021 --crate-type=lib --crate-name=generic_drop_native --emit=link -Copt-level=0 -Cpanic=unwind -Zdump-mir=all -Zmir-opt-level=0 -Zidentify-regions=yes")
    args = parser.parse_args()

    if args.prepare_native_rustc_input:
        if args.output is None:
            parser.error("--output is required with --prepare-native-rustc-input")
        metadata = prepare_native_rustc_input(args.output)
        print(json.dumps(metadata, indent=2, sort_keys=True))
        return 0

    features = parse_features(args.features)
    source = SOURCE.read_text(encoding="utf-8")
    drop_items, scope_items = validate_source(source)
    for effect in EFFECTS:
        load_effect_mir(args.mir_dir, effect)

    shadow, helper_metadata = generate_shadow(source, drop_items, scope_items, features)
    if args.output_dir:
        out_dir = args.output_dir
    elif features:
        out_dir = CANONICAL_GENERATED / "controls" / "+".join(sorted(features))
    else:
        out_dir = CANONICAL_GENERATED
    out_dir.mkdir(parents=True, exist_ok=True)
    shadow_path = out_dir / "shadow.rs"
    mapping_path = out_dir / "mapping.json"
    shadow_path.write_text(shadow, encoding="utf-8")

    # Calls are gathered from generated source, rather than fabricated from
    # the intended recipe. The independent checker still parses the source.
    call_sites: dict[str, list[dict[str, object]]] = {}
    for spec in EFFECTS:
        sites = []
        scope_shadow = item(shadow, f"pub fn {spec.scope}(")
        scope_text = scope_shadow.text
        for match in re.finditer(rf"\b{re.escape(spec.helper)}\s*\(\s*&mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;", scope_text):
            absolute_start = scope_shadow.start + match.start()
            line, col = line_col(shadow, absolute_start)
            sites.append({
                "line": line,
                "column": col,
                "byte_span": [absolute_start, scope_shadow.start + match.end()],
                "text": match.group(0),
                "argument": match.group(1),
                "order_in_scope": len(sites),
                "text_sha256": sha256_text(match.group(0)),
            })
        call_sites[spec.scope] = sites

    effects = effect_mapping(
        source,
        drop_items,
        scope_items,
        helper_metadata,
        args.mir_dir,
        features,
        shadow_path.relative_to(ROOT).as_posix(),
    )
    for effect in effects:
        effect["injection"]["observed_generated_calls"] = call_sites[effect["scope"]]
        effect["injection"]["expected_call_count"] = 0 if "omit_drop_call" in features else (2 if "duplicate_drop_call" in features and effect["source_destructor"]["type"] == "ToggleOnDrop" else 1)
        effect["injection"]["generated_function_body_sha256"] = sha256_text(item(shadow, f"pub fn {effect['scope']}(").body)

    removed = erase_spec_attributes(source)[1]
    mir_dir_resolved = args.mir_dir.resolve()
    try:
        mir_dir_label = mir_dir_resolved.relative_to(ROOT).as_posix()
    except ValueError:
        mir_dir_label = str(mir_dir_resolved)
    native_input_metadata_path = args.mir_dir / "source-erasure.json"
    native_input_metadata = (
        json.loads(native_input_metadata_path.read_text(encoding="utf-8"))
        if native_input_metadata_path.is_file()
        else None
    )
    mapping = {
        "schema_version": 1,
        "generator": {
            "path": "elaborate.py",
            "sha256": sha256_bytes(Path(__file__).read_bytes()),
            "scope": "closed source/MIR structural elaborator; not a proof of arbitrary Drop elaboration",
        },
        "inputs": {
            "source_path": "native.rs",
            "source_sha256": sha256_text(source),
            "native_mir_dir": mir_dir_label,
            "native_rustc_input": native_input_metadata,
            "verifier_syntax_erased_for_native_mir": removed,
        },
        "toolchain": {
            "rustc_version": args.rustc_version,
            "rustc_commit": args.rustc_commit,
            "rustc_flags": args.rustc_flags,
            "mir_stage": MIR_STAGE,
        },
        "variant": {
            "kind": "positive" if not features else "diagnostic-control",
            "features": sorted(features),
            "shadow_path": str(shadow_path.relative_to(ROOT)),
            "shadow_sha256": sha256_text(shadow),
            "mapping_path": str(mapping_path.relative_to(ROOT)),
        },
        "effects": effects,
        "normal_return_scope_only": True,
        "unwind_edges_are_recorded_not_claimed": True,
        "trust_boundary": "external checker correspondence plus native MIR normal-edge injection; destructor/caller bodies remain ordinary proof bodies",
    }
    mapping_path.write_text(json.dumps(mapping, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {shadow_path.relative_to(ROOT)}")
    print(f"wrote {mapping_path.relative_to(ROOT)}")
    print(f"variant: {mapping['variant']['kind']} {sorted(features)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"elaborate.py: error: {error}", file=sys.stderr)
        raise SystemExit(2)
