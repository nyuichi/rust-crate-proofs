#!/usr/bin/env python3
"""Fail-closed checker for the one admitted protocol-only client.

This checks executable source correspondence and client event completeness. It
does not prove the cursor's event contracts, final recovery, or any Bytes law.
The accepted Rust language is intentionally one closed witness; any new
executable construct requires review and an explicit checker update.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE = ROOT / "src" / "driver.rs"
LIB_SOURCE = ROOT / "src" / "lib.rs"
FACADE_SOURCE = ROOT / "src" / "scoped.rs"

# This is the admitted client grammar, not a receipt generated from the source.
# Comments and whitespace are ignored by the lexer; strings remain opaque tokens
# and cannot supply a call-shaped executable event.
EXPECTED_SOURCE = r'''use super::*;
struct Marker(u8);
impl RecoveryPayload for Marker {
    type Metadata = u8;
    #[logic] fn metadata(self) -> u8 { self.0 }
    #[logic(prophetic)] fn wellformed(self) -> bool { true }
}
#[ensures(result)]
pub fn closed_sparse_lifecycle() -> bool {
    let (registry, first, mut cursor) = Registry::new(
        ghost! { Marker(42) }, ghost! { LifetimeToken::new() });
    let (_, second) = registry.register(first.borrow(), cursor.borrow_mut());
    let (_, third) = registry.register(first.borrow(), cursor.borrow_mut());
    let (third_last, third_recovery) = registry.retire(third, cursor.borrow_mut());
    proof_assert!(!third_last && third_recovery.inner_logic() == None);
    let (first_last, first_recovery) = registry.retire(first, cursor.borrow_mut());
    proof_assert!(!first_last && first_recovery.inner_logic() == None);
    let (_, fourth) = registry.register(second.borrow(), cursor.borrow_mut());
    let (second_last, second_recovery) = registry.retire(second, cursor.borrow_mut());
    proof_assert!(!second_last && second_recovery.inner_logic() == None);
    let (fourth_last, fourth_recovery) = registry.retire(fourth, cursor.borrow_mut());
    proof_assert!(fourth_last && fourth_recovery.inner_logic() != None);
    ghost! {
        let (payload, full) = fourth_recovery.into_inner().unwrap();
        proof_assert!(payload.metadata() == 42u8);
        proof_assert!(full.frac() == PositiveReal::from_int(1));
        let _dead = full.end();
    };
    fourth_last
}'''

EXPECTED_EVENTS = [
    {"kind": "register", "source": "first", "ticket": "second"},
    {"kind": "register", "source": "first", "ticket": "third"},
    {"kind": "retire", "ticket": "third"},
    {"kind": "retire", "ticket": "first"},
    {"kind": "register", "source": "second", "ticket": "fourth"},
    {"kind": "retire", "ticket": "second"},
    {"kind": "retire", "ticket": "fourth"},
]


class CoverageError(Exception):
    pass


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def mask_rust_noncode(text: str) -> str:
    """Blank comments and literals without changing byte offsets or newlines."""
    chars = list(text)
    i = 0
    n = len(text)

    def blank(a: int, b: int) -> None:
        for j in range(a, b):
            if chars[j] != "\n":
                chars[j] = " "

    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            if j < 0:
                j = n
            blank(i, j)
            i = j
            continue
        if text.startswith("/*", i):
            start = i
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
            if depth:
                raise CoverageError("unterminated block comment")
            blank(start, i)
            continue
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", text[i:])
        if raw:
            start = i
            hashes = raw.group(1)
            i += len(raw.group(0))
            close = '"' + hashes
            end = text.find(close, i)
            if end < 0:
                raise CoverageError("unterminated raw string")
            i = end + len(close)
            blank(start, i)
            continue
        if text[i] == '"':
            start = i
            i += 1
            escaped = False
            while i < n:
                c = text[i]
                i += 1
                if escaped:
                    escaped = False
                elif c == "\\":
                    escaped = True
                elif c == '"':
                    break
            else:
                raise CoverageError("unterminated string")
            blank(start, i)
            continue
        i += 1
    return "".join(chars)


def rust_tokens(text: str) -> list[str]:
    """Small fail-closed token scanner; comments vanish, literals are opaque."""
    out: list[str] = []
    i = 0
    n = len(text)
    multi = ("::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..")
    while i < n:
        if text[i].isspace():
            i += 1
            continue
        if text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j + 1
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
            if depth:
                raise CoverageError("unterminated block comment")
            continue
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", text[i:])
        if raw:
            hashes = raw.group(1)
            i += len(raw.group(0))
            close = '"' + hashes
            end = text.find(close, i)
            if end < 0:
                raise CoverageError("unterminated raw string token")
            i = end + len(close)
            out.append("<literal>")
            continue
        if text[i] == '"':
            i += 1
            escaped = False
            while i < n:
                c = text[i]
                i += 1
                if escaped:
                    escaped = False
                elif c == "\\":
                    escaped = True
                elif c == '"':
                    break
            else:
                raise CoverageError("unterminated string token")
            out.append("<literal>")
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
        op = next((x for x in multi if text.startswith(x, i)), None)
        if op:
            out.append(op)
            i += len(op)
            continue
        out.append(text[i])
        i += 1
    return out


def mask_rust_comments_keep_literals(text: str) -> str:
    """Mask comments while preserving path strings in module attributes."""
    chars = list(text)
    i = 0
    n = len(text)

    def blank(a: int, b: int) -> None:
        for j in range(a, b):
            if chars[j] != "\n":
                chars[j] = " "

    while i < n:
        raw = re.match(r"(?:br|rb|cr|r)(#{0,})\"", text[i:])
        if raw:
            hashes = raw.group(1)
            i += len(raw.group(0))
            end = text.find('"' + hashes, i)
            if end < 0:
                raise CoverageError("unterminated raw string in crate root")
            i = end + len(hashes) + 1
            continue
        if text[i] == '"':
            i += 1
            escaped = False
            while i < n:
                c = text[i]
                i += 1
                if escaped:
                    escaped = False
                elif c == "\\":
                    escaped = True
                elif c == '"':
                    break
            else:
                raise CoverageError("unterminated string in crate root")
            continue
        if text.startswith("//", i):
            start = i
            end = text.find("\n", i)
            i = n if end < 0 else end
            blank(start, i)
            continue
        if text.startswith("/*", i):
            start = i
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
            if depth:
                raise CoverageError("unterminated block comment in crate root")
            blank(start, i)
            continue
        i += 1
    return "".join(chars)


def rust_item_body(source: str, name: str) -> tuple[list[str], int, int]:
    code = mask_rust_noncode(source)
    matches = list(re.finditer(r"\bpub\s+fn\s+" + re.escape(name) + r"\s*\(", code))
    if len(matches) != 1:
        raise CoverageError(f"expected one public {name} entrypoint, found {len(matches)}")
    start = matches[0].start()
    brace = code.find("{", matches[0].end())
    if brace < 0:
        raise CoverageError(f"missing body for {name}")
    depth = 0
    end = -1
    for i in range(brace, len(code)):
        if code[i] == "{":
            depth += 1
        elif code[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    if end < 0:
        raise CoverageError(f"unclosed body for {name}")
    return rust_tokens(source[brace + 1:end - 1]), start, end


def split_top_level(tokens: list[str], separator: str = ",") -> list[list[str]]:
    parts: list[list[str]] = [[]]
    paren = bracket = brace = 0
    for token in tokens:
        if token == separator and paren == bracket == brace == 0:
            parts.append([])
            continue
        parts[-1].append(token)
        if token == "(": paren += 1
        elif token == ")": paren -= 1
        elif token == "[": bracket += 1
        elif token == "]": bracket -= 1
        elif token == "{": brace += 1
        elif token == "}": brace -= 1
        if min(paren, bracket, brace) < 0:
            raise CoverageError("unbalanced delimiter in client")
    if paren or bracket or brace:
        raise CoverageError("unbalanced delimiter in client")
    return parts


def top_level_statements(body: list[str]) -> list[list[str]]:
    statements: list[list[str]] = []
    current: list[str] = []
    paren = bracket = brace = 0
    for token in body:
        if token == ";" and paren == bracket == brace == 0:
            if current:
                statements.append(current)
                current = []
            continue
        current.append(token)
        if token == "(": paren += 1
        elif token == ")": paren -= 1
        elif token == "[": bracket += 1
        elif token == "]": bracket -= 1
        elif token == "{": brace += 1
        elif token == "}": brace -= 1
        if min(paren, bracket, brace) < 0:
            raise CoverageError("unbalanced delimiter in client body")
    if paren or bracket or brace:
        raise CoverageError("unbalanced delimiter in client body")
    if current:
        statements.append(current)
    return statements


def invocation_args(tokens: list[str], marker: tuple[str, ...]) -> list[list[str]]:
    positions = [i for i in range(len(tokens) - len(marker) + 1)
                 if tuple(tokens[i:i + len(marker)]) == marker]
    if len(positions) != 1:
        raise CoverageError(f"expected one {'.'.join(marker[:-1])} call in statement")
    start = positions[0] + len(marker)
    parts: list[list[str]] = [[]]
    depth = 0
    i = start
    while i < len(tokens):
        token = tokens[i]
        if token == "(":
            depth += 1
            parts[-1].append(token)
        elif token == ")":
            if depth == 0:
                return parts
            depth -= 1
            parts[-1].append(token)
        elif token == "," and depth == 0:
            parts.append([])
        else:
            parts[-1].append(token)
        i += 1
    raise CoverageError("unterminated registry event call")


def event_statements(body: list[str]) -> tuple[list[dict[str, str]], int, int]:
    observed: list[dict[str, str]] = []
    register_count = retire_count = 0
    live = {"first"}
    ever_issued = {"first"}
    for statement in top_level_statements(body):
        register = "registry" in statement and "register" in statement
        retire = "registry" in statement and "retire" in statement
        if not register and not retire:
            continue
        if register and retire:
            raise CoverageError("one statement may not combine registry events")
        if statement[:1] != ["let"] or statement[1:2] != ["("]:
            raise CoverageError("registry events must bind their complete result locally")
        bind_end = statement.index(")", 2)
        bindings = split_top_level(statement[2:bind_end])
        if len(bindings) != 2 or len(statement) <= bind_end + 2 or statement[bind_end + 1] != "=":
            raise CoverageError("registry event result shape is unsupported")
        call_kind = "register" if register else "retire"
        marker = ("registry", ".", call_kind, "(")
        args = invocation_args(statement, marker)
        if len(args) != 2 or args[1] != ["cursor", ".", "borrow_mut", "(", ")"]:
            raise CoverageError(f"registry.{call_kind} must thread the one cursor local")
        if register:
            register_count += 1
            source_arg = args[0]
            if len(source_arg) != 5 or source_arg[1:] != [".", "borrow", "(", ")"]:
                raise CoverageError("registration source must be a borrowed local ticket")
            source_ticket = source_arg[0]
            ticket_binding = bindings[1]
            if len(ticket_binding) != 1 or ticket_binding[0] in {"_", "first"}:
                raise CoverageError("registration must bind one fresh affine ticket")
            new_ticket = ticket_binding[0]
            if source_ticket not in live or new_ticket in ever_issued:
                raise CoverageError("registration source/output is not live and affine")
            live.add(new_ticket)
            ever_issued.add(new_ticket)
            observed.append({"kind": "register", "source": source_ticket, "ticket": new_ticket})
        else:
            retire_count += 1
            ticket_arg = args[0]
            if len(ticket_arg) != 1:
                raise CoverageError("retirement must consume one local ticket")
            ticket = ticket_arg[0]
            if ticket not in live:
                raise CoverageError("retirement does not consume one live ticket")
            live.remove(ticket)
            observed.append({"kind": "retire", "ticket": ticket})
    if live:
        raise CoverageError(f"client returns with live tickets: {sorted(live)}")
    return observed, register_count, retire_count


def count_subsequence(haystack: list[str], needle: list[str]) -> int:
    return sum(haystack[i:i + len(needle)] == needle
               for i in range(len(haystack) - len(needle) + 1))


def check_support_identity(lib_source: str, facade_source: str) -> None:
    """Check import/module identity and the erased cursor facade signatures.

    This is interface wiring only. It does not derive facade callback bodies,
    spec-macro expansion, code generation, or a native interpretation.
    """
    lib = rust_tokens(lib_source)
    expected_external_paths = {
        "fraction_map": "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs",
        "ref_count_limit": "../../../../src/ref_count_limit.rs",
        "relaxed": "../../original-public-shared-gate-2026-10-08/src/relaxed.rs",
        "release": "../../shared-physical-lifecycle-2026-10-08/src/release.rs",
    }
    lib_with_comments_masked = mask_rust_comments_keep_literals(lib_source)
    path_rows = re.findall(r'#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;',
                           lib_with_comments_masked)
    actual_external_paths = {name: path for path, name in path_rows}
    if len(path_rows) != len(expected_external_paths) or actual_external_paths != expected_external_paths:
        raise CoverageError("src/lib.rs external #[path] module mapping changed")
    declared_modules = re.findall(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", mask_rust_noncode(lib_source))
    if sorted(declared_modules) != sorted([
            "fraction_map", "ref_count_limit", "relaxed", "release",
            "event", "lifecycle", "scoped", "driver"]):
        raise CoverageError("src/lib.rs module declarations changed or gained an alias")
    for name in ("event", "lifecycle", "scoped", "driver"):
        if count_subsequence(lib, ["mod", name, ";"]) != 1:
            raise CoverageError(f"src/lib.rs must declare exactly mod {name} from its default path")
        at = next(i for i in range(len(lib) - 2)
                  if lib[i:i + 3] == ["mod", name, ";"])
        if at > 0 and lib[at - 1] == "]":
            raise CoverageError(f"src/lib.rs changes the source path for mod {name}")
    registry_route = ["pub", "use", "scoped", "::", "Registry", ";"]
    driver_route = ["pub", "use", "driver", "::", "closed_sparse_lifecycle", ";"]
    if count_subsequence(lib, registry_route) != 1:
        raise CoverageError("src/lib.rs Registry export is not exactly scoped::Registry")
    if count_subsequence(lib, driver_route) != 1:
        raise CoverageError("src/lib.rs client export is not exactly driver::closed_sparse_lifecycle")
    if lib.count("Registry") != 1 or count_subsequence(lib, ["as", "Registry"]) != 0:
        raise CoverageError("src/lib.rs has a Registry import override or alias")

    facade = rust_tokens(facade_source)
    registry_struct = ["pub", "struct", "Registry", "<", "T", ":", "RecoveryPayload", ">",
                       "{", "atomic", ":", "ScopedEventAtomic", "<", "State", "<", "T", ">", ">", "}"]
    registry_impl = ["impl", "<", "T", ":", "RecoveryPayload", ">", "Registry", "<", "T", ">", "{"]
    if count_subsequence(facade, registry_struct) != 1:
        raise CoverageError("src/scoped.rs Registry facade representation changed")
    if count_subsequence(facade, registry_impl) != 1:
        raise CoverageError("src/scoped.rs does not contain the unique Registry implementation")

    signatures = {
        "new": "pub fn new(payload: Ghost<T>, full: Ghost<LifetimeToken>) -> (Self, Ghost<Ticket<T>>, Ghost<ScopeCursor<State<T>>>)",
        "register": "pub fn register(&self, source: Ghost<&Ticket<T>>, cursor: Ghost<&mut ScopeCursor<State<T>>>) -> (usize, Ghost<Ticket<T>>) ",
        "retire": "pub fn retire(&self, ticket: Ghost<Ticket<T>>, mut cursor: Ghost<&mut ScopeCursor<State<T>>>) -> (bool, Ghost<Option<(T, LifetimeToken)>>) ",
    }
    facade_code = mask_rust_noncode(facade_source)
    for name, expected in signatures.items():
        matches = list(re.finditer(r"\bpub\s+fn\s+" + re.escape(name) + r"\s*\(", facade_code))
        if len(matches) != 1:
            raise CoverageError(f"src/scoped.rs must have one public Registry::{name}")
        start = matches[0].start()
        brace = facade_code.find("{", matches[0].end())
        if brace < 0 or rust_tokens(facade_source[start:brace]) != rust_tokens(expected):
            raise CoverageError(f"Registry::{name} erased cursor interface changed")


def audit_source(source: str, *, source_name: str = "src/driver.rs",
                 lib_source: str | None = None, facade_source: str | None = None) -> dict[str, Any]:
    if lib_source is None:
        lib_source = LIB_SOURCE.read_text()
    if facade_source is None:
        facade_source = FACADE_SOURCE.read_text()
    check_support_identity(lib_source, facade_source)

    code_only = mask_rust_noncode(source)
    code_tokens = rust_tokens(code_only)
    forbidden = {
        "unsafe", "thread", "spawn", "forget", "ManuallyDrop", "transmute",
        "clone", "clone_from", "to_owned", "addr_of", "addr_of_mut",
        "static_mut", "asm", "global_asm", "Fn", "FnMut", "FnOnce",
    }
    present = sorted(forbidden.intersection(code_tokens))
    if present:
        raise CoverageError("unsupported callback/escape/concurrency effect: " + ", ".join(present))
    for i in range(len(code_tokens) - 2):
        if code_tokens[i:i + 3] in (["as", "*", "mut"], ["as", "*", "const"],
                                    ["*", "mut", "_"], ["*", "const", "_"]):
            raise CoverageError("raw-pointer alias is outside the closed client")
    if "|" in code_tokens or "||" in code_tokens:
        raise CoverageError("closures/callbacks are outside the closed client")

    actual = rust_tokens(source)
    expected = rust_tokens(EXPECTED_SOURCE)
    if actual != expected:
        # The caller source is the proof boundary. Its allowed executable and
        # ghost constructs are fixed above; hashes never authorize new syntax.
        raise CoverageError("driver source is outside the reviewed closed-client grammar")

    body, start, end = rust_item_body(source, "closed_sparse_lifecycle")
    observed, register_count, retire_count = event_statements(body)
    if observed != EXPECTED_EVENTS or register_count != 3 or retire_count != 4:
        raise CoverageError("observed ticket/cursor event sequence differs from the admitted witness")
    constructors = [i for i in range(len(body) - 3)
                    if body[i:i + 4] == ["Registry", "::", "new", "("]]
    if len(constructors) != 1 or body[:1] != ["let"]:
        raise CoverageError("client must begin with one fresh Registry::new cursor constructor")

    return {
        "status": "correspondence_pass",
        "checker": "check_closed_scope.py; independent fail-closed token reconstruction",
        "source": source_name,
        "source_sha256": sha(source.encode()),
        "support_identity": {
            "lib_path": "src/lib.rs",
            "crate_module_paths": {
                "event": "src/event.rs",
                "lifecycle": "src/lifecycle.rs",
                "scoped": "src/scoped.rs",
                "driver": "src/driver.rs",
                "fraction_map": "../../shared-physical-lifecycle-2026-10-08/src/fraction_map.rs",
                "ref_count_limit": "../../../../src/ref_count_limit.rs",
                "relaxed": "../../original-public-shared-gate-2026-10-08/src/relaxed.rs",
                "release": "../../shared-physical-lifecycle-2026-10-08/src/release.rs",
            },
            "facade_module_path": "mod scoped; -> src/scoped.rs",
            "registry_facade_identity": "src/scoped.rs::Registry<T: RecoveryPayload>",
            "registry_export": "pub use scoped::Registry",
            "checked_facade_interface": "Registry::new returns a Ghost<ScopeCursor>; register/retire receive that cursor by Ghost<&mut ...>",
            "lib_sha256": sha(lib_source.encode()),
            "scoped_sha256": sha(facade_source.encode()),
            "body_correspondence": "not derived for generic facade callbacks, codegen, or spec-macro expansion",
        },
        "entrypoint": "closed_sparse_lifecycle",
        "fresh_constructor_count": 1,
        "cursor_binding": "single fresh local returned by Registry::new",
        "cursor_events": register_count + retire_count,
        "register_count": register_count,
        "retire_count": retire_count,
        "events": observed,
        "live_tickets_at_normal_return": 0,
        "escaped_handles": 0,
        "ids_assumed": False,
        "semantic_scope": "driver-level source/effect completeness for this protocol-only client only; generic facade/native interpretation and codegen/spec-macro noninterference remain reviewed TCB; no native MIR mapping, final-recovery theorem, cursor-contract proof, actual Bytes coverage, or unwind claim",
        "source_span": {"start": start, "end": end},
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=pathlib.Path, default=SOURCE)
    args = parser.parse_args()
    try:
        report = audit_source(args.source.read_text(), source_name=args.source.as_posix())
    except (OSError, CoverageError) as exc:
        print(json.dumps({"status": "coverage_failure", "error": str(exc)}, indent=2))
        return 2
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
