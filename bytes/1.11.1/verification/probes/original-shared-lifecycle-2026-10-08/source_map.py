#!/usr/bin/env python3
"""Audit exact production-source windows selected by this lifecycle probe.

This does not prove correspondence. It records which real bytes.rs blocks a
probe adapter is intended to represent, and fails closed if any block changes.
After an intentional production edit, review the source map and regenerate it.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "src" / "bytes.rs"
ATOMIC_ALIAS_SOURCE = ROOT / "src" / "loom.rs"
BORROWED_CORE_SOURCE = ROOT / "src" / "ownership_proof" / "raw_vec.rs"
BOX_HELPER_SOURCE = ROOT / "src" / "ownership_proof" / "boxed_alignment.rs"
OWNED_REGION_SOURCE = ROOT / "src" / "ownership_proof" / "owned_region.rs"
ALLOCATION_OPS_SOURCE = ROOT / "src" / "allocation_ops.rs"
PROBE_DIR = Path(__file__).parent
BOUNDED_PROBE_DIR = PROBE_DIR.parent / "shared-physical-lifecycle-2026-10-08"
MANIFEST = Path(__file__).with_name("source_map.json")


def line_block(source: str, start: str, end: str) -> tuple[str, int, int]:
    """Return exact source lines from a unique start through a unique end."""
    start_at = source.find(start)
    if start_at < 0 or source.find(start, start_at + len(start)) >= 0:
        raise ValueError(f"expected one start anchor: {start!r}")
    end_at = source.find(end, start_at + len(start))
    if end_at < 0:
        raise ValueError(f"missing end anchor after {start!r}: {end!r}")
    # Some anchors name the next declaration to make the closing brace unique.
    # Keep only the exact preceding `\n}` in that case.
    end_at += end.index("\n\n") if "\n\n" in end else len(end)
    block = source[start_at:end_at]
    return block, source.count("\n", 0, start_at) + 1, source.count("\n", 0, end_at) + 1


def blocks(source: str) -> dict[str, tuple[str, int, int]]:
    return {
        "from_vec_spare_capacity_constructor": line_block(
            source,
            "impl From<Vec<u8>> for Bytes {\n// ORIGINAL_FREEZE_BEGIN bytes_from_vec",
            "// ORIGINAL_FREEZE_END bytes_from_vec\n}",
        ),
        "bytes_record": line_block(
            source,
            "pub struct Bytes {\n",
            "\n}\n\npub(crate) struct Vtable",
        ),
        "shared_record": line_block(
            source,
            "struct Shared {\n",
            "\n}\n\nimpl Drop for Shared",
        ),
        "shared_drop": line_block(
            source,
            "impl Drop for Shared {\n",
            "\n}\n\n// Assert that the alignment of `Shared`",
        ),
        "shared_clone_dispatch": line_block(
            source,
            "unsafe fn shared_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {",
            "\n}\n\nunsafe fn shared_to_vec_impl",
        ),
        "shallow_clone_arc": line_block(
            source,
            "unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {",
            "\n}\n\n#[cold]",
        ),
        "release_shared": line_block(
            source,
            "// ORIGINAL_SHARED_BEGIN release_shared\n",
            "// ORIGINAL_SHARED_END release_shared",
        ),
        "free_shared": line_block(
            source,
            "// ORIGINAL_SHARED_BEGIN free_shared\n",
            "// ORIGINAL_SHARED_END free_shared",
        ),
        "shared_non_dropping_atomic_assertion": line_block(
            source,
            "const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize]",
            ";",
        ),
    }


def make_manifest() -> dict[str, object]:
    source = SOURCE.read_text()
    atomic_source = ATOMIC_ALIAS_SOURCE.read_text()
    atomic_alias, atomic_first, atomic_last = line_block(
        atomic_source,
        "#[cfg(not(feature = \"extra-platforms\"))]\n        pub(crate) use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};",
        "Ordering};",
    )
    atomic_mut, atomic_mut_first, atomic_mut_last = line_block(
        atomic_source,
        "impl<T> AtomicMut<T> for AtomicPtr<T> {",
        "\n            }\n        }\n    }\n}",
    )
    result: dict[str, object] = {
        "source": "src/bytes.rs",
        "source_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "selected_native_atomic_alias": {
            "source": "src/loom.rs",
            "configuration": "not(all(test, loom)); not(feature = extra-platforms)",
            "first_line": atomic_first,
            "last_line": atomic_last,
            "sha256": hashlib.sha256(atomic_alias.encode()).hexdigest(),
        },
        "selected_mutable_atomic_pointer_access": {
            "source": "src/loom.rs",
            "configuration": "not(all(test, loom)); not(feature = extra-platforms)",
            "first_line": atomic_mut_first,
            "last_line": atomic_mut_last,
            "sha256": hashlib.sha256(atomic_mut.encode()).hexdigest(),
            "native_operation": "AtomicMut::with_mut passes AtomicPtr::get_mut() to the closure",
        },
        "blocks": {},
        "reviewed_files": {},
        "bounded_protocol_dependency": {
            "path": "../shared-physical-lifecycle-2026-10-08",
            "source_files": {},
        },
    }
    for name, path in {
        "src/ownership_proof/raw_vec.rs": BORROWED_CORE_SOURCE,
        "src/ownership_proof/boxed_alignment.rs": BOX_HELPER_SOURCE,
        "src/ownership_proof/owned_region.rs": OWNED_REGION_SOURCE,
        "src/allocation_ops.rs": ALLOCATION_OPS_SOURCE,
        "probe/Cargo.toml": PROBE_DIR / "Cargo.toml",
        "probe/Cargo.lock": PROBE_DIR / "Cargo.lock",
        "probe/src/lib.rs": PROBE_DIR / "src" / "lib.rs",
        "probe/src/provenance_specs.rs": PROBE_DIR / "src" / "provenance_specs.rs",
        "probe/src/field_event.rs": PROBE_DIR / "src" / "field_event.rs",
        "probe/src/pointer_event.rs": PROBE_DIR / "src" / "pointer_event.rs",
        "probe/src/source_adapter.rs": PROBE_DIR / "src" / "source_adapter.rs",
        "probe/run-proof.sh": PROBE_DIR / "run-proof.sh",
        "probe/run-negative-no-acquire.sh": PROBE_DIR / "run-negative-no-acquire.sh",
        "probe/Cargo.toml (negative mutant feature)": PROBE_DIR / "Cargo.toml",
    }.items():
        result["reviewed_files"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
    result["bounded_protocol_dependency"]["manifest_sha256"] = hashlib.sha256(
        (BOUNDED_PROBE_DIR / "Cargo.toml").read_bytes()
    ).hexdigest()
    result["bounded_protocol_dependency"]["lock_sha256"] = hashlib.sha256(
        (BOUNDED_PROBE_DIR / "Cargo.lock").read_bytes()
    ).hexdigest()
    for path in sorted((BOUNDED_PROBE_DIR / "src").rglob("*.rs")):
        relative = path.relative_to(BOUNDED_PROBE_DIR).as_posix()
        result["bounded_protocol_dependency"]["source_files"][relative] = hashlib.sha256(
            path.read_bytes()
        ).hexdigest()
    for name, (block, first, last) in blocks(source).items():
        result["blocks"][name] = {
            "first_line": first,
            "last_line": last,
            "sha256": hashlib.sha256(block.encode()).hexdigest(),
        }
    return result


def main() -> int:
    actual = make_manifest()
    refresh = "--refresh" in sys.argv[1:]
    if not MANIFEST.exists() or refresh:
        MANIFEST.write_text(json.dumps(actual, indent=2) + "\n")
        print(f"wrote {MANIFEST.name}" if not refresh else f"refreshed {MANIFEST.name}")
        return 0
    expected = json.loads(MANIFEST.read_text())
    if expected != actual:
        print("production Shared source differs from the reviewed source map")
        print(json.dumps(actual, indent=2))
        return 1
    print("production and adapter source map matches")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
