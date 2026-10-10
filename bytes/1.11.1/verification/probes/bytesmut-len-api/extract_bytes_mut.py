import hashlib
import json
import os
import pathlib

manifest = pathlib.Path(__file__).resolve().parent
crate = manifest.parents[2]
source = (crate / "src/bytes_mut.rs").read_text()


def marker(name):
    begin = f"// ORIGINAL_UNIQUE_BEGIN {name}\n"
    end = f"// ORIGINAL_UNIQUE_END {name}"
    assert source.count(begin) == 1, ("missing/duplicate begin", name)
    assert source.count(end) == 1, ("missing/duplicate end", name)
    start = source.index(begin) + len(begin)
    stop = source.index(end, start)
    return source[start:stop]


def item(signature):
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    state = "code"
    escaped = False
    index = opening
    while index < len(source):
        char = source[index]
        pair = source[index:index + 2]
        if state == "line":
            if char == "\n":
                state = "code"
        elif state == "block":
            if pair == "*/":
                state = "code"
                index += 1
        elif state == "string":
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                state = "code"
        elif pair == "//":
            state = "line"
            index += 1
        elif pair == "/*":
            state = "block"
            index += 1
        elif char == '"':
            state = "string"
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return source[start:index + 1]
        index += 1
    raise AssertionError(("unterminated item", signature))


parts = {
    "mutable_shared_record": item("struct Shared {"),
    "bytes_mut_record": item("pub struct BytesMut {"),
    "unique_model": marker("proof_model"),
    "len": marker("len"),
    "truncate": marker("truncate"),
    "set_len": marker("set_len"),
}

imports = """use alloc::vec::Vec;
use core::{ptr::NonNull, sync::atomic::AtomicUsize};
use creusot_std::prelude::*;
use crate::ownership_proof::raw_vec::{BoundPtr, RawAllocation, Recovery, PhysicalRegion, slot_known};
"""
generated = imports + "\n" + parts["mutable_shared_record"] + "\n" + parts["bytes_mut_record"] + "\n"
generated += parts["unique_model"] + "\nimpl BytesMut {\n"
generated += parts["len"] + "\n" + parts["truncate"] + "\n" + parts["set_len"] + "\n}\n"

out = pathlib.Path(os.environ["OUT_DIR"])
(out / "bytes_mut_api.rs").write_text(generated)
mapping = {
    "source": "src/bytes_mut.rs",
    "full_source_sha256": hashlib.sha256(source.encode()).hexdigest(),
    "sha256": {name: hashlib.sha256(part.encode()).hexdigest() for name, part in parts.items()},
    "included_exactly": ["mutable_shared_record", "bytes_mut_record", "unique_model", "len", "truncate", "set_len"],
    "scope": "actual extracted BytesMut len/truncate/set_len source with unique Vec invariant",
}
(out / "source-map.json").write_text(json.dumps(mapping, indent=2) + "\n")
