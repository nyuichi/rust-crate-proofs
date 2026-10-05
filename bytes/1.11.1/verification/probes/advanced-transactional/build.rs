use std::{env, fs, path::PathBuf};

fn brace_end(source: &str, open: usize) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    let mut state = 0u8; // code, line comment, block comment, string, char
    let mut block_depth = 0usize;
    let mut escaped = false;
    while i < bytes.len() {
        let b = bytes[i];
        match state {
            0 => match (b, bytes.get(i + 1).copied()) {
                (b'/', Some(b'/')) => { state = 1; i += 2; continue; }
                (b'/', Some(b'*')) => { state = 2; block_depth = 1; i += 2; continue; }
                (b'"', _) => state = 3,
                (b'\'', _) => state = 4,
                (b'{', _) => depth += 1,
                (b'}', _) => {
                    depth -= 1;
                    if depth == 0 { return i + 1; }
                }
                _ => {}
            },
            1 => if b == b'\n' { state = 0; },
            2 => match (b, bytes.get(i + 1).copied()) {
                (b'/', Some(b'*')) => { block_depth += 1; i += 2; continue; }
                (b'*', Some(b'/')) => { block_depth -= 1; i += 2; if block_depth == 0 { state = 0; } continue; }
                _ => {}
            },
            3 | 4 => {
                if escaped { escaped = false; }
                else if b == b'\\' { escaped = true; }
                else if (state == 3 && b == b'"') || (state == 4 && b == b'\'') { state = 0; }
            }
            _ => unreachable!(),
        }
        i += 1;
    }
    panic!("unterminated source item at byte {open}");
}

fn item(source: &str, start_marker: &str, body_marker: &str) -> String {
    let start = source.find(start_marker).unwrap_or_else(|| panic!("missing source marker: {start_marker}"));
    assert!(source[start + start_marker.len()..].find(start_marker).is_none(), "duplicate start marker: {start_marker}");
    let body = start + source[start..].find(body_marker).unwrap_or_else(|| panic!("missing body marker: {body_marker}"));
    let open = body + source[body..].find('{').expect("source item opening brace");
    source[start..brace_end(source, open)].to_owned()
}

fn line(source: &str, marker: &str) -> String {
    let start = source.find(marker).unwrap_or_else(|| panic!("missing source line: {marker}"));
    let end = start + source[start..].find('\n').unwrap_or(source.len() - start);
    source[start..end].to_owned()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in bytes { h = (h ^ u64::from(*b)).wrapping_mul(0x100000001b3); }
    h
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../../");
    let path = root.join("src/bytes_mut.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-changed=build.rs");
    for cfg in ["bytes_proof_probe", "bytes_proof_valid_handle"] {
        println!("cargo:rustc-cfg={cfg}");
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    let src = fs::read_to_string(path).unwrap();

    let fields = vec![
        item(&src, "pub struct BytesMut {", "pub struct BytesMut {"),
        item(&src, "struct Shared {", "struct Shared {"),
        item(&src, "struct SharedBuffer {", "struct SharedBuffer {"),
        line(&src, "const KIND_ARC: usize = 0b0;"),
        line(&src, "const KIND_VEC: usize = 0b1;"),
        line(&src, "const KIND_MASK: usize = 0b1;"),
        item(&src, "#[inline]\n#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]\nfn invalid_ptr<T>(addr: usize) -> *mut T {", "fn invalid_ptr<T>(addr: usize) -> *mut T {"),
    ];
    let mut methods = vec![
        item(&src, "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_view_slot", "pub(crate) fn proof_view_slot(self, index: Int) -> Option<Option<u8>> {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_owned_slot", "pub(crate) fn proof_owned_slot(self, index: Int) -> Option<Option<u8>> {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic]\n    pub(crate) fn proof_unique_slot", "pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_empty_valid", "fn proof_empty_valid(self) -> bool {"),
        item(&src, "    #[cfg(all(creusot, bytes_proof_valid_handle))]\n    #[logic(prophetic)]\n    fn proof_owned_valid", "fn proof_owned_valid(self) -> bool {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_initialized", "fn proof_initialized(self) -> bool {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_shared_allocation_registered", "fn proof_shared_allocation_registered(self) -> bool {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_registered_valid", "fn proof_registered_valid(self) -> bool {"),
        item(&src, "    #[cfg(creusot)]\n    #[logic(prophetic)]\n    fn proof_unique_owned", "fn proof_unique_owned(self) -> bool {"),
    ];
    let source_advance = item(&src, "    // BEGIN EXACT ADVANCE_UNCHECKED", "pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {");
    let source_set_position = item(&src, "    // BEGIN EXACT SET_VEC_POS", "unsafe fn set_vec_pos(&mut self, pos: usize) {");
    let source_get_position = item(&src, "    // BEGIN EXACT GET_VEC_POS", "unsafe fn get_vec_pos(&self) -> usize {");
    let source_kind = item(&src,
        "    #[inline]\n    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]",
        "fn kind(&self) -> usize {");
    let source_invariant = item(&src, "// BEGIN EXACT BYTESMUT INVARIANT",
        "impl creusot_std::invariant::Invariant for BytesMut {");
    let raw_start = "// BEGIN EXACT UNIQUE ADVANCE RAW TRANSITION";
    let raw_end = "// END EXACT UNIQUE ADVANCE RAW TRANSITION";
    let raw_start_index = src.find(raw_start).expect("raw transition start marker");
    let raw_end_index = src.find(raw_end).expect("raw transition end marker");
    assert!(raw_end_index > raw_start_index, "raw transition marker order");
    let source_raw_transition = &src[raw_start_index..raw_end_index + raw_end.len()];
    let control_start = src.find("// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE").unwrap();
    let control_end = src.find("// END EXACT SEQUENTIAL SHARED CONTROL GATE").unwrap();
    let control = &src[control_start..control_end + "// END EXACT SEQUENTIAL SHARED CONTROL GATE".len()];

    let mut generated = String::from(
        "use alloc::{boxed::Box, vec::Vec};\nuse core::mem;\nuse core::ptr::NonNull;\nuse core::sync::atomic::{AtomicUsize, Ordering};\nuse creusot_std::prelude::*;\nuse crate::capacity_ops::{original_capacity_to_repr, MAX_VEC_POS};\n"
    );
    for f in &fields { generated.push_str(f); generated.push('\n'); }
    generated.push_str(&source_invariant);
    let unique_owned_source = methods[8].clone();
    let initialized_source = methods[5].clone();
    let shared_registered_source = methods[6].clone();
    let empty_source = methods[3].clone();
    generated.push_str("impl BytesMut {\n");
    for f in methods.drain(..) { generated.push_str(&f); generated.push('\n'); }
    generated.push_str(&source_kind);
    generated.push('\n');
    generated.push_str(&source_get_position);
    generated.push('\n');
    generated.push_str(&source_set_position);
    generated.push('\n');
    generated.push_str(&source_advance);
    generated.push_str("\n}\n");
    generated.push_str(source_raw_transition);
    generated.push('\n');
    generated.push_str(control);
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_advanced.rs"), generated).unwrap();
    let mut fragments = String::new();
    for (name, text) in [
        ("BytesMut fields", fields[0].as_str()),
        ("proof_unique_owned", unique_owned_source.as_str()),
        ("proof_initialized", initialized_source.as_str()),
        ("proof_shared_allocation_registered", shared_registered_source.as_str()),
        ("proof_empty_valid", empty_source.as_str()),
        ("actual invariant", source_invariant.as_str()),
        ("actual advance_unchecked", source_advance.as_str()),
        ("actual kind", source_kind.as_str()),
        ("actual get_vec_pos", source_get_position.as_str()),
        ("actual set_vec_pos", source_set_position.as_str()),
        ("actual raw transition", source_raw_transition),
    ] { fragments.push_str(&format!("{name}: {:016x}\n", fnv1a64(text.as_bytes()))); }
    fs::write(out.join("source_fragments.txt"), fragments).unwrap();

    let extraction = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("extraction");
    fs::create_dir_all(&extraction).unwrap();
    fs::write(extraction.join("actual_advanced.rs"), fs::read(out.join("actual_advanced.rs")).unwrap()).unwrap();
    fs::write(extraction.join("source_fragments.txt"), fs::read(out.join("source_fragments.txt")).unwrap()).unwrap();
    fs::write(extraction.join("source_advance_unchecked.rs"), source_advance).unwrap();
    fs::write(extraction.join("source_get_vec_pos.rs"), source_get_position).unwrap();
    fs::write(extraction.join("source_set_vec_pos.rs"), source_set_position).unwrap();
    fs::write(extraction.join("source_raw_transition.rs"), source_raw_transition).unwrap();
    fs::write(extraction.join("source_invariant.rs"), source_invariant).unwrap();
}
