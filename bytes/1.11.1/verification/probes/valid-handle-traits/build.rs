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
    let source_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../../src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    let caller_source_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", caller_source_path.display());
    for cfg in ["bytes_proof_probe", "bytes_proof_valid_handle"] {
        println!("cargo:rustc-cfg={cfg}");
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    let source = fs::read_to_string(&source_path).unwrap();
    let caller_source = fs::read_to_string(&caller_source_path).unwrap();
    let is_empty_caller = extract_item(
        &caller_source,
        "probe_unique_is_empty_caller",
        "#[cfg(any())]\nfn proof_observe_is_empty",
        "fn proof_observe_is_empty(owner: &actual::BytesMut) {",
    );
    let is_empty_native_caller = extract_item(
        &caller_source,
        "probe_unique_is_empty_native_caller",
        "#[cfg(not(creusot))]\npub fn unique_is_empty(input: Vec<u8>) -> bool {",
        "pub fn unique_is_empty(input: Vec<u8>) -> bool {",
    );
    let mut items = vec![
        extract_item(&source, "release_unique_storage", "// BEGIN EXACT RELEASE_UNIQUE_STORAGE", "unsafe fn release_unique_storage("),
        extract_item(&source, "BytesMut", "pub struct BytesMut {", "pub struct BytesMut {"),
        extract_item(&source, "Shared", "struct Shared {", "struct Shared {"),
        extract_item(&source, "SharedBuffer", "struct SharedBuffer {", "struct SharedBuffer {"),
        extract_item(&source, "Invariant", "// BEGIN EXACT BYTESMUT INVARIANT", "impl creusot_std::invariant::Invariant for BytesMut {"),
        extract_item(&source, "AsRef", "impl AsRef<[u8]> for BytesMut {", "impl AsRef<[u8]> for BytesMut {"),
        extract_item(&source, "AsMut", "impl AsMut<[u8]> for BytesMut {", "impl AsMut<[u8]> for BytesMut {"),
        extract_item(&source, "invalid_ptr", "#[inline]\n#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]", "fn invalid_ptr<T>(addr: usize) -> *mut T {"),
    ];
    for (name, marker) in [("KIND_VEC", "const KIND_VEC: usize = 0b1;"), ("KIND_ARC", "const KIND_ARC: usize = 0b0;"), ("KIND_MASK", "const KIND_MASK: usize = 0b1;")] {
        items.push(extract_line(&source, name, marker));
    }
    let start = unique_index(&source, "// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE");
    let end = unique_index(&source, "// END EXACT SEQUENTIAL SHARED CONTROL GATE");
    items.push(Fragment { name: "control_types_and_helpers", text: &source[start..end], start, end });
    let methods = vec![
        extract_item(&source, "reject_unregistered_as_ref", "    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle, feature = \"negative_unregistered_as_ref\"))]", "    pub(crate) fn proof_reject_unregistered_as_ref(#[cfg_attr(creusot, creusot::open_inv)] owner: &Self) {"),
        extract_item(&source, "proof_unique_at_zero_owned", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_owned", "    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {"),
        extract_item(&source, "proof_unique_at_zero_valid", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    pub(crate) fn proof_unique_at_zero_valid", "    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {"),
        extract_item(&source, "proof_unique_owned", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_unique_owned", "    fn proof_unique_owned(self) -> bool {"),
        extract_item(&source, "proof_unique_slot", "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_unique_slot", "    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {"),
        extract_item(&source, "proof_view_slot", "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_view_slot", "    pub(crate) fn proof_view_slot(self, index: Int) -> Option<Option<u8>> {"),
        extract_item(&source, "proof_owned_slot", "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_owned_slot", "    pub(crate) fn proof_owned_slot(self, index: Int) -> Option<Option<u8>> {"),
        extract_item(&source, "proof_empty_valid", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_empty_valid", "    fn proof_empty_valid(self) -> bool {"),
        extract_item(&source, "proof_owned_valid", "    #[cfg(all(creusot, bytes_proof_valid_handle))]\n    #[logic(prophetic)]\n    fn proof_owned_valid", "    #[cfg(all(creusot, bytes_proof_valid_handle))]\n    #[logic(prophetic)]\n    fn proof_owned_valid(self) -> bool {"),
        extract_item(&source, "proof_initialized", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_initialized", "    fn proof_initialized(self) -> bool {"),
        extract_item(&source, "proof_registered_valid", "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_registered_valid", "    fn proof_registered_valid(self) -> bool {"),
        extract_item(&source, "from_vec", "    #[inline]\n    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]", "    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {"),
        extract_item(&source, "proof_release_unique_at_zero", "    // Explicit restricted proof path.", "    pub(crate) fn proof_release_unique_at_zero(mut self) {"),
        extract_item(&source, "as_slice", "    // BEGIN EXACT AS_SLICE\n", "    fn as_slice(&self) -> &[u8] {"),
        extract_item(&source, "as_slice_mut", "    // BEGIN EXACT AS_SLICE_MUT\n", "    fn as_slice_mut(&mut self) -> &mut [u8] {"),
        extract_item(&source, "spare_capacity_mut", "    // BEGIN EXACT SPARE_CAPACITY_MUT\n", "    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {"),
        extract_item(&source, "truncate", "    // BEGIN EXACT TRUNCATE", "    pub fn truncate(&mut self, len: usize) {"),
        extract_item(&source, "clear", "    // BEGIN EXACT CLEAR", "    pub fn clear(&mut self) {"),
        extract_item(&source, "set_len", "    // BEGIN EXACT SET_LEN", "    pub unsafe fn set_len(&mut self, len: usize) {"),
        extract_item(&source, "len", "    #[inline]\n    #[cfg_attr(creusot, ensures(result == self.len))]", "    pub fn len(&self) -> usize {"),
        extract_item(&source, "is_empty", "    // BEGIN EXACT IS_EMPTY\n", "    pub fn is_empty(&self) -> bool {"),
        extract_item(&source, "capacity", "    #[inline]\n    #[cfg_attr(creusot, ensures(result == self.cap))]", "    pub fn capacity(&self) -> usize {"),
        extract_item(&source, "kind", "    #[inline]\n    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]", "    fn kind(&self) -> usize {"),
        extract_item(&source, "proof_generic_as_ref", "    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]\n    fn proof_generic_as_ref<T: AsRef<[u8]>>(owner: &T) -> Option<u8>", "    fn proof_generic_as_ref<T: AsRef<[u8]>>(owner: &T) -> Option<u8> {"),
        extract_item(&source, "proof_generic_as_mut", "    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]\n    fn proof_generic_as_mut<T: AsMut<[u8]>>(owner: &mut T, value: u8)", "    fn proof_generic_as_mut<T: AsMut<[u8]>>(owner: &mut T, value: u8) {"),
        extract_item(&source, "proof_traits_unique", "    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]\n    pub(crate) fn proof_traits_unique(input: Vec<u8>, value: u8)", "    pub(crate) fn proof_traits_unique(input: Vec<u8>, value: u8) {"),
    ];
    let drop_buffer = extract_item(&source, "SharedBufferDrop", "impl Drop for SharedBuffer {", "impl Drop for SharedBuffer {");
    let mut generated = String::from("use alloc::{vec::Vec, boxed::Box};\nuse core::mem::{self, ManuallyDrop, MaybeUninit};\nuse core::ptr::{self, NonNull};\nuse core::cmp;\nuse core::sync::atomic::{AtomicUsize, Ordering};\nuse creusot_std::prelude::*;\nuse crate::capacity_ops::original_capacity_to_repr;\n");
    for f in &items { generated.push_str(f.text); generated.push('\n'); }
    generated.push_str("impl BytesMut {\n");
    for f in &methods {
        if f.name == "len" || f.name == "is_empty" {
            let method = match f.name {
                "len" => "    pub fn len(&self) -> usize {",
                "is_empty" => "    pub fn is_empty(&self) -> bool {",
                _ => unreachable!(),
            };
            let body = f.text.find(method).expect("exact observer source body");
            generated.push_str(&f.text[..body]);
            generated.push_str("    #[cfg_attr(creusot, check(ghost))]\n");
            generated.push_str(&f.text[body..]);
        } else {
            generated.push_str(f.text);
        }
        generated.push('\n');
    }
    generated.push_str("}\n#[cfg(not(creusot))]\n");
    generated.push_str(drop_buffer.text);
    let relocated_is_empty_caller = is_empty_caller
        .text
        .replace("#[cfg(any())]\n", "#[cfg(creusot)]\n")
        .replace("actual::BytesMut", "BytesMut");
    generated.push_str("\n");
    generated.push_str(&relocated_is_empty_caller);
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_traits.rs"), &generated).unwrap();
    let mut manifest = String::new();
    for f in items.iter().chain(methods.iter()).chain([&drop_buffer]) {
        manifest.push_str(&format!("{} {} {} {:016x}\n", f.name, f.start, f.end, fnv1a64(f.text.as_bytes())));
    }
    manifest.push_str(&format!("{} {} {} {:016x}\n", is_empty_caller.name, is_empty_caller.start, is_empty_caller.end, fnv1a64(is_empty_caller.text.as_bytes())));
    manifest.push_str(&format!("{} {} {} {:016x}\n", is_empty_native_caller.name, is_empty_native_caller.start, is_empty_native_caller.end, fnv1a64(is_empty_native_caller.text.as_bytes())));
    manifest.push_str("probe instrumentation: generated len and is_empty methods add cfg_attr(creusot, check(ghost)) to verify the exact source bodies; source hashes above exclude these check-only attributes\n");
    manifest.push_str("proof caller placement: exact caller template from src/lib.rs has cfg(any()) there and is relocated into the generated actual module under cfg(creusot), with only the `actual::` type qualifier removed so its calls share the actual BytesMut module's field contracts\n");
    manifest.push_str("scope: exact valid-handle invariant, unique constructor/access/cleanup, actual len/is_empty/capacity observers, core AsRef/AsMut and generic callers; promotion/splitting/automatic BytesMut Drop excluded\n");
    fs::write(out.join("source_fragments.txt"), &manifest).unwrap();
    let extraction = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("extraction");
    fs::create_dir_all(&extraction).unwrap();
    fs::write(extraction.join("actual_traits.rs"), &generated).unwrap();
    fs::write(extraction.join("source_fragments.txt"), &manifest).unwrap();
    fs::write(extraction.join("unique_is_empty_caller.rs"), is_empty_caller.text).unwrap();
    fs::write(extraction.join("relocated_unique_is_empty_caller.rs"), &relocated_is_empty_caller).unwrap();
    fs::write(extraction.join("unique_is_empty_native_caller.rs"), is_empty_native_caller.text).unwrap();
    let is_empty_method = methods.iter().find(|f| f.name == "is_empty").expect("is_empty method fragment");
    fs::write(extraction.join("source_is_empty.rs"), is_empty_method.text).unwrap();
}
