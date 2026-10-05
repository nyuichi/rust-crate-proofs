use std::{env, fs, path::PathBuf};

struct Fragment<'a> {
    name: &'static str,
    text: &'a str,
    start: usize,
    end: usize,
}

fn matching_brace(source: &str, open: usize) -> usize {
    let bytes = source.as_bytes();
    assert_eq!(bytes[open], b'{');
    let mut depth = 0usize;
    let mut i = open;
    let mut state = 0u8; // code, line comment, block comment, string, char
    let mut block_depth = 0usize;
    let mut escaped = false;

    while i < bytes.len() {
        let byte = bytes[i];
        match state {
            0 => match (byte, bytes.get(i + 1).copied()) {
                (b'/', Some(b'/')) => {
                    state = 1;
                    i += 2;
                    continue;
                }
                (b'/', Some(b'*')) => {
                    state = 2;
                    block_depth = 1;
                    i += 2;
                    continue;
                }
                (b'"', _) => state = 3,
                (b'\'', _) => state = 4,
                (b'{', _) => depth += 1,
                (b'}', _) => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1;
                    }
                }
                _ => {}
            },
            1 => {
                if byte == b'\n' {
                    state = 0;
                }
            }
            2 => match (byte, bytes.get(i + 1).copied()) {
                (b'/', Some(b'*')) => {
                    block_depth += 1;
                    i += 2;
                    continue;
                }
                (b'*', Some(b'/')) => {
                    block_depth -= 1;
                    i += 2;
                    if block_depth == 0 {
                        state = 0;
                    }
                    continue;
                }
                _ => {}
            },
            3 | 4 => {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if (state == 3 && byte == b'"') || (state == 4 && byte == b'\'') {
                    state = 0;
                }
            }
            _ => unreachable!(),
        }
        i += 1;
    }
    panic!("unterminated Rust item starting at byte {open}");
}

fn unique_index(source: &str, marker: &str) -> usize {
    let first = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    assert_eq!(
        source[first + marker.len()..].find(marker),
        None,
        "duplicate marker: {marker}"
    );
    first
}

fn extract_item<'a>(
    source: &'a str,
    name: &'static str,
    start_marker: &str,
    body_marker: &str,
) -> Fragment<'a> {
    let start = unique_index(source, start_marker);
    let body = unique_index(source, body_marker);
    assert!(start <= body, "item prefix follows its body marker: {name}");
    let open = body + source[body..].find('{').expect("item body opening brace");
    let end = matching_brace(source, open);
    Fragment {
        name,
        text: &source[start..end],
        start,
        end,
    }
}

fn extract_line<'a>(source: &'a str, name: &'static str, marker: &str) -> Fragment<'a> {
    let start = unique_index(source, marker);
    let end = start + source[start..].find('\n').unwrap_or(source.len() - start);
    Fragment {
        name,
        text: &source[start..end],
        start,
        end,
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let bytes_mut_path = manifest_dir.join("../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", bytes_mut_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let source = fs::read_to_string(&bytes_mut_path).expect("read actual bytes_mut.rs");
    let control_start = unique_index(&source, "// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE");
    let control_end = unique_index(&source, "// END EXACT SEQUENTIAL SHARED CONTROL GATE");
    let fragments = [
        Fragment { name: "sequential_shared_control", text: &source[control_start..control_end], start: control_start, end: control_end },
        extract_item(&source, "BytesMut", "pub struct BytesMut {", "pub struct BytesMut {"),
        extract_item(&source, "Shared", "struct Shared {", "struct Shared {"),
        extract_item(&source, "SharedBuffer", "struct SharedBuffer {", "struct SharedBuffer {"),
        extract_line(&source, "KIND_VEC", "const KIND_VEC: usize = 0b1;"),
        extract_line(&source, "KIND_MASK", "const KIND_MASK: usize = 0b1;"),
        extract_item(
            &source,
            "from_vec",
            "    #[inline]\n    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]",
            "    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {",
        ),
        extract_item(
            &source,
            "proof_unique_at_zero_owned",
            "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_owned",
            "    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {",
        ),
        extract_item(
            &source,
            "proof_unique_at_zero_valid",
            "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_valid",
            "    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {",
        ),
        extract_item(
            &source,
            "proof_unique_slot",
            "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_unique_slot",
            "    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {",
        ),
        extract_item(
            &source,
            "proof_release_unique_at_zero",
            "    // Explicit restricted proof path.",
            "    pub(crate) fn proof_release_unique_at_zero(mut self) {",
        ),
        extract_item(
            &source,
            "vptr_native",
            "#[inline]\n#[cfg(not(creusot))]\nfn vptr",
            "fn vptr(ptr: *mut u8) -> NonNull<u8> {",
        ),
        extract_item(
            &source,
            "vptr_creusot",
            "// Unadapted pointer updates carry no physical binding.",
            "fn vptr(ptr: *mut u8) -> crate::ownership_proof::raw_vec::BoundPtr {",
        ),
        extract_item(
            &source,
            "invalid_ptr",
            "#[inline]\n#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]",
            "fn invalid_ptr<T>(addr: usize) -> *mut T {",
        ),
    ];

    let mut generated = String::from(
        "// Generated from exact source fragments by build.rs; do not edit.\n\
         use alloc::{vec::Vec, boxed::Box};\n\
         use core::mem::{self, ManuallyDrop};\n\
         use core::ptr::NonNull;\n\
         use core::sync::atomic::AtomicUsize;\n\
         use creusot_std::prelude::*;\n\
         use crate::capacity_ops::original_capacity_to_repr;\n\n",
    );
    let is_method = |name: &str| matches!(name,
        "from_vec" | "proof_unique_at_zero_owned" | "proof_unique_at_zero_valid" | "proof_unique_slot" | "proof_release_unique_at_zero");
    for fragment in fragments.iter().filter(|fragment| !is_method(fragment.name)) {
        generated.push_str(fragment.text);
        generated.push_str("\n\n");
    }
    generated.push_str("impl BytesMut {\n");
    for fragment in fragments.iter().filter(|fragment| is_method(fragment.name)) {
        generated.push_str(fragment.text);
        generated.push_str("\n\n");
    }
    generated.push_str("}\n");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("actual_from_vec.rs"), generated).expect("write extracted source");

    let mut manifest = String::from("source: src/bytes_mut.rs\nmethod: exact fragments extracted at build time\nhash: FNV-1a-64 (audit identifier)\n");
    for fragment in &fragments {
        let start_line = source[..fragment.start].lines().count() + 1;
        let end_line = source[..fragment.end].lines().count() + 1;
        let hash = fnv1a64(fragment.text.as_bytes());
        manifest.push_str(&format!(
            "{}: lines {}-{}, bytes {}, fnv1a64 {:016x}\n",
            fragment.name,
            start_line,
            end_line,
            fragment.text.len(),
            hash
        ));
    }
    fs::write(out_dir.join("source-extraction-manifest.txt"), manifest)
        .expect("write extraction manifest");
}
