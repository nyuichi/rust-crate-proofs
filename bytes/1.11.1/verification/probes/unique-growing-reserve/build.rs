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
    let source_path = manifest_dir.join("../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-cfg=bytes_proof_probe");
    println!("cargo:rustc-cfg=bytes_proof_unique_growth");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_unique_growth)");
    println!("cargo:rustc-cfg=bytes_proof_noalloc");
    println!("cargo:rustc-cfg=bytes_proof_unique_advance");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_unique_advance)");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_noalloc)");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_probe)");
    let source = fs::read_to_string(&source_path).unwrap();
    let mut items = vec![
        extract_item(&source, "release_unique_storage", "// BEGIN EXACT RELEASE_UNIQUE_STORAGE", "unsafe fn release_unique_storage("),
        extract_item(&source,"BytesMut","pub struct BytesMut {","pub struct BytesMut {"),
        extract_item(&source,"Shared","struct Shared {","struct Shared {"),
        extract_item(&source,"SharedBuffer","struct SharedBuffer {","struct SharedBuffer {"),
        extract_line(&source,"KIND_VEC","const KIND_VEC: usize = 0b1;"),
        extract_line(&source,"KIND_ARC","const KIND_ARC: usize = 0b0;"),
        extract_line(&source,"KIND_MASK","const KIND_MASK: usize = 0b1;"),
        extract_item(&source,"invalid_ptr","#[inline]\n#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]","fn invalid_ptr<T>(addr: usize) -> *mut T {"),
    ];
    let methods = vec![
        extract_item(&source,"reserve_unique_growing","    // BEGIN EXACT UNIQUE_GROWING_RESERVE","    fn reserve_unique_growing(&mut self, additional: usize) {"),
        extract_item(&source,"get_vec_pos","    // BEGIN EXACT GET_VEC_POS","    unsafe fn get_vec_pos(&self) -> usize {"),
        extract_item(&source,"set_vec_pos","    // BEGIN EXACT SET_VEC_POS","    unsafe fn set_vec_pos(&mut self, pos: usize) {"),
        extract_item(&source,"resize","    // BEGIN EXACT RESIZE","    pub fn resize("),
        extract_item(&source,"reserve","    // BEGIN EXACT RESERVE","    pub fn reserve("),
        extract_item(&source,"extend_from_slice","    // BEGIN EXACT EXTEND_FROM_SLICE","    pub fn extend_from_slice("),
        extract_item(&source,"proof_unique_at_zero_owned","    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_owned","    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {"),
        extract_item(&source,"proof_release_unique_at_zero","    // Explicit restricted proof path.","    pub(crate) fn proof_release_unique_at_zero(mut self) {"),
        extract_item(&source,"as_slice","    // BEGIN EXACT AS_SLICE\n","    fn as_slice(&self) -> &[u8] {"),
        extract_item(&source,"spare_capacity_mut","    // BEGIN EXACT SPARE_CAPACITY_MUT","    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {"),
        extract_item(&source,"truncate","    // BEGIN EXACT TRUNCATE","    pub fn truncate(&mut self, len: usize) {"),
        extract_item(&source,"clear","    // BEGIN EXACT CLEAR","    pub fn clear(&mut self) {"),
        extract_item(&source,"set_len","    // BEGIN EXACT SET_LEN","    pub unsafe fn set_len(&mut self, len: usize) {"),
        extract_item(&source,"capacity","    #[inline]\n    #[cfg_attr(creusot, ensures(result == self.cap))]","    pub fn capacity(&self) -> usize {"),
        extract_item(&source,"split_off","    // BEGIN EXACT SPLIT_OFF","    pub fn split_off(&mut self, at: usize) -> BytesMut {"),
        extract_item(&source,"as_slice_mut","    // BEGIN EXACT AS_SLICE_MUT","    fn as_slice_mut(&mut self) -> &mut [u8] {"),
        extract_item(&source,"from_vec","    #[inline]\n    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]","    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {"),
        extract_item(&source,"proof_unique_at_zero_valid","    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_valid","    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {"),
        extract_item(&source,"proof_unique_slot","    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_unique_slot","    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {"),
        extract_item(&source,"len","    #[inline]\n    #[cfg_attr(creusot, ensures(result == self.len))]","    pub fn len(&self) -> usize {"),
        extract_item(&source,"kind","    #[inline]\n    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]","    fn kind(&self) -> usize {"),
        extract_item(&source,"split_to","    // BEGIN EXACT SPLIT_TO","    pub fn split_to(&mut self, at: usize) -> BytesMut {"),
        extract_item(&source,"advance_unchecked","    // BEGIN EXACT ADVANCE_UNCHECKED","    pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {"),
        extract_item(&source,"promote_to_shared","    // BEGIN EXACT PROMOTE_TO_SHARED","    unsafe fn promote_to_shared(&mut self, ref_cnt: usize) {"),
        extract_item(&source,"shallow_clone","    // BEGIN EXACT SHALLOW_CLONE","    unsafe fn shallow_clone(&mut self) -> BytesMut {"),
        extract_item(&source,"pending_descriptor_copy","    // Copies metadata only.","    fn pending_descriptor_copy(&self) -> BytesMut {"),
    ];
    let start=unique_index(&source,"// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE");
    let end=unique_index(&source,"// END EXACT SEQUENTIAL SHARED CONTROL GATE");
    items.push(Fragment {name:"sequential_shared_control",text:&source[start..end],start,end});
    let start=unique_index(&source,"    // BEGIN EXACT BYTESMUT SEQUENTIAL SPLIT METHODS");
    let end=unique_index(&source,"    // END EXACT BYTESMUT SEQUENTIAL SPLIT METHODS");
    let helpers=Fragment{name:"split_proof_methods",text:&source[start..end],start,end};
    let drop_buffer=extract_item(&source,"SharedBuffer::drop","impl Drop for SharedBuffer {","impl Drop for SharedBuffer {");
    let mut generated=String::from("// Exact source fragments, recorded below.\nuse alloc::{vec::Vec, boxed::Box};\nuse core::mem::{self, ManuallyDrop, MaybeUninit};\nuse core::ptr::{self,NonNull};\nuse core::cmp;\nuse core::sync::atomic::{AtomicUsize,Ordering};\nuse creusot_std::prelude::*;\nuse crate::capacity_ops::{original_capacity_to_repr, MAX_VEC_POS};\n");
    for f in &items {generated.push_str(f.text);generated.push('\n');}
    generated.push_str("impl BytesMut {\n");
    for f in &methods {generated.push_str(f.text);generated.push('\n');}
    generated.push_str(helpers.text);
    generated.push_str("\n}\n#[cfg(not(creusot))]\n");
    generated.push_str(drop_buffer.text);
    let out=PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_split.rs"),generated).unwrap();
    let mut records=String::new();
    for f in items.iter().chain(methods.iter()).chain([&helpers,&drop_buffer]) {
        records.push_str(&format!("{} {} {} {:016x}\n",f.name,f.start,f.end,fnv1a64(f.text.as_bytes())));
    }
    records.push_str("adaptation: cfg(bytes_proof_probe) selects exact restricted representation/permission branches; actual SharedBuffer Drop is native-only; BytesMut automatic Drop is excluded\n");
    fs::write(out.join("source_fragments.txt"),records).unwrap();
}
