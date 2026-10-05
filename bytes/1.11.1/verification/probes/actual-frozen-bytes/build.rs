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
                (b'\'', _) => {
                    // Lifetimes such as `'a` do not have a closing quote.
                    // Enter char-literal state only for `'{` / `'x'` or an
                    // escaped char like `-'\\n'`.
                    if bytes.get(i + 1) == Some(&b'\\') || bytes.get(i + 2) == Some(&b'\'') {
                        state = 4;
                    }
                }
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
        .unwrap_or_else(|| panic!("missing source marker: {marker}"));
    assert_eq!(
        source[first + marker.len()..].find(marker),
        None,
        "duplicate source marker: {marker}"
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
    assert!(start <= body, "item prefix follows body marker: {name}");
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
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let path = manifest.join("../../../src/bytes.rs");
    let source = fs::read_to_string(&path).unwrap();
    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-changed=fixture.rs");
    println!("cargo:rustc-cfg=bytes_proof_frozen");
    println!("cargo:rustc-cfg=bytes_proof_probe");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_frozen)");
    let fragments = [
        extract_item(&source, "Bytes", "pub struct Bytes {", "pub struct Bytes {"),
        extract_item(&source, "Vtable", "pub(crate) struct Vtable {", "pub(crate) struct Vtable {"),
        extract_item(&source, "constructor", "    // BEGIN EXACT FROZEN BYTES CONSTRUCTOR", "    pub(crate) unsafe fn with_vtable("),
        extract_item(&source, "read", "    // BEGIN EXACT FROZEN BYTES READ", "    fn as_slice(&self) -> &[u8]"),
        extract_item(&source, "share", "    // BEGIN EXACT FROZEN BYTES SHARE", "    pub(crate) fn proof_share_frozen("),
        extract_item(&source, "close", "    // BEGIN EXACT FROZEN BYTES CLOSE", "    pub(crate) fn proof_return_frozen_ticket("),
    ];
    let mut out = String::from("use creusot_std::prelude::*;\nuse core::sync::atomic::AtomicPtr;\nuse alloc::vec::Vec;\nuse alloc::boxed::Box;\nuse core::mem::{self,ManuallyDrop};\nuse core::ptr::NonNull;\nuse core::sync::atomic::AtomicUsize;\nuse crate::capacity_ops::original_capacity_to_repr;\n");
    let mut_path = manifest.join("../../../src/bytes_mut.rs");
    let mutable=fs::read_to_string(&mut_path).unwrap();
    println!("cargo:rerun-if-changed={}",mut_path.display());
    let cs=unique_index(&mutable,"// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE");
    let ce=unique_index(&mutable,"// END EXACT SEQUENTIAL SHARED CONTROL GATE");
    out.push_str(&mutable[cs..ce]);
    let mut_fragments=[
        extract_item(&mutable,"BytesMut","pub struct BytesMut {","pub struct BytesMut {"),
        extract_item(&mutable,"Shared","struct Shared {","struct Shared {"),
        extract_item(&mutable,"SharedBuffer","struct SharedBuffer {","struct SharedBuffer {"),
        extract_line(&mutable,"KIND_VEC","const KIND_VEC: usize = 0b1;"),
        extract_line(&mutable,"KIND_MASK","const KIND_MASK: usize = 0b1;"),
        extract_item(&mutable,"invalid_ptr","#[inline]\n#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]","fn invalid_ptr<T>(addr: usize) -> *mut T {"),
    ];
    for f in &mut_fragments {out.push_str(f.text);out.push('\n');}
    out.push_str("impl BytesMut {\n");
    let mut_methods=[
        extract_item(&mutable,"from_vec","    #[inline]\n    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]","    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {"),
        extract_item(&mutable,"owned","    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_owned","    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {"),
        extract_item(&mutable,"valid","    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_valid","    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {"),
        extract_item(&mutable,"slot","    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_unique_slot","    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {"),
        extract_item(&mutable,"freeze","    // BEGIN EXACT FROZEN BYTESMUT FREEZE","    pub fn freeze(self,"),
    ];
    for f in &mut_methods {out.push_str(f.text);out.push('\n');}
    out.push_str("}\n");
    out.push_str(fragments[0].text); out.push('\n');
    out.push_str(fragments[1].text); out.push_str("\nimpl Bytes {\n");
    for f in &fragments[2..] { out.push_str(f.text); out.push('\n'); }
    out.push_str("}\n");
    if env::var_os("CARGO_FEATURE_NEGATIVE_ACTUAL_CLONE").is_some() {
        let clone = extract_item(&source, "Clone", "impl Clone for Bytes {", "impl Clone for Bytes {");
        let old = "(&self.data, self.ptr, self.len, self.vtable)";
        let new = "(&self.data, self.ptr.as_ptr() as *const u8, self.len, self.vtable)";
        assert_eq!(clone.text.matches(old).count(), 1);
        let adapted = clone.text.replace(old, new);
        assert_eq!(adapted.replace(new, old), clone.text);
        out.push_str(&adapted);
        out.push('\n');
        fs::write(manifest.join("artifacts/clone-original.rs"), clone.text).unwrap();
        fs::write(manifest.join("artifacts/clone-adapted.rs"), adapted).unwrap();
    }
    out.push_str(&fs::read_to_string(manifest.join("fixture.rs")).unwrap());
    let mut correspondence = String::new();
    for f in &fragments {
        assert_eq!(f.text, &source[f.start..f.end]);
        correspondence.push_str(&format!("{} bytes {}..{} fnv1a64 {:016x}\n", f.name, f.start, f.end, fnv1a64(f.text.as_bytes())));
    }
    for f in mut_fragments.iter().chain(mut_methods.iter()) {
        assert_eq!(f.text,&mutable[f.start..f.end]);
        correspondence.push_str(&format!("bytes_mut {} bytes {}..{} fnv1a64 {:016x}\n",f.name,f.start,f.end,fnv1a64(f.text.as_bytes())));
    }
    let dest = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(dest.join("actual_frozen.rs"), &out).unwrap();
    fs::write(manifest.join("artifacts/extracted.rs"), out).unwrap();
    fs::write(manifest.join("artifacts/source-correspondence.txt"), correspondence).unwrap();
}
