use std::{env, fs, path::PathBuf};

fn matching_brace(source: &str, open: usize) -> usize {
    let bytes = source.as_bytes();
    assert_eq!(bytes[open], b'{');
    let mut depth = 0usize;
    let mut index = open;
    let mut state = 0u8; // code, line comment, block comment, string, char
    let mut block_depth = 0usize;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        match state {
            0 => match (byte, bytes.get(index + 1).copied()) {
                (b'/', Some(b'/')) => {
                    state = 1;
                    index += 2;
                    continue;
                }
                (b'/', Some(b'*')) => {
                    state = 2;
                    block_depth = 1;
                    index += 2;
                    continue;
                }
                (b'"', _) => state = 3,
                (b'\'', _) => state = 4,
                (b'{', _) => depth += 1,
                (b'}', _) => {
                    depth -= 1;
                    if depth == 0 {
                        return index + 1;
                    }
                }
                _ => {}
            },
            1 => {
                if byte == b'\n' {
                    state = 0;
                }
            }
            2 => match (byte, bytes.get(index + 1).copied()) {
                (b'/', Some(b'*')) => {
                    block_depth += 1;
                    index += 2;
                    continue;
                }
                (b'*', Some(b'/')) => {
                    block_depth -= 1;
                    index += 2;
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
        index += 1;
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

fn extract_item<'a>(source: &'a str, start_marker: &str, body_marker: &str) -> &'a str {
    let start = unique_index(source, start_marker);
    let body = unique_index(source, body_marker);
    assert!(start <= body, "item prefix follows its body marker");
    let open = body + source[body..].find('{').expect("item body opening brace");
    let end = matching_brace(source, open);
    &source[start..end]
}

fn extract_impl_method<'a>(source: &'a str, impl_marker: &str, body_marker: &str) -> &'a str {
    let start = unique_index(source, impl_marker);
    let body = start + source[start..].find(body_marker).expect("impl method body marker");
    let open = body + source[body..].find('{').expect("method body opening brace");
    let end = matching_brace(source, open);
    &source[start..end]
}

fn extract_block<'a>(source: &'a str, start_marker: &str) -> &'a str {
    let start = unique_index(source, start_marker);
    let open = start + source[start..].find('{').expect("block opening brace");
    let end = matching_brace(source, open);
    &source[start..end]
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source_path = manifest.join("../../../src/buf/uninit_slice.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    let source = fs::read_to_string(&source_path).unwrap();

    let struct_start = unique_index(&source, "#[repr(transparent)]\npub struct UninitSlice(");
    let struct_end = struct_start + source[struct_start..].find(';').unwrap() + 1;
    let struct_item = &source[struct_start..struct_end];
    let view_impl = extract_block(&source, "impl View for UninitSlice {");
    let methods = [
        extract_item(
            &source,
            "    /// Creates a `&mut UninitSlice` wrapping a slice of initialised memory.",
            "    pub fn new(slice: &mut [u8]) -> &mut UninitSlice {",
        ),
        extract_item(
            &source,
            "    /// Creates a `&mut UninitSlice` wrapping a slice of uninitialised memory.",
            "    pub fn uninit(slice: &mut [MaybeUninit<u8>]) -> &mut UninitSlice {",
        ),
        extract_item(
            &source,
            "    /// Write a single byte at the specified offset.",
            "    pub fn write_byte(&mut self, index: usize, byte: u8) {",
        ),
        extract_item(
            &source,
            "    /// Copies bytes from `src` into `self`.",
            "    pub fn copy_from_slice(&mut self, src: &[u8]) {",
        ),
        extract_item(
            &source,
            "    /// Returns the number of bytes in the slice.",
            "    pub fn len(&self) -> usize {",
        ),
    ];
    let from_initialized = extract_impl_method(
        &source,
        "impl<'a> From<&'a mut [u8]> for &'a mut UninitSlice {",
        "    fn from(slice: &'a mut [u8]) -> Self {",
    );
    let from_maybe_uninit = extract_impl_method(
        &source,
        "impl<'a> From<&'a mut [MaybeUninit<u8>]> for &'a mut UninitSlice {",
        "    fn from(slice: &'a mut [MaybeUninit<u8>]) -> Self {",
    );

    let mut extracted = String::from(
        "use core::mem::MaybeUninit;\nuse creusot_std::prelude::*;\n",
    );
    extracted.push_str(struct_item);
    extracted.push_str("\n");
    extracted.push_str(view_impl);
    extracted.push_str("\n");
    extracted.push_str("\nimpl UninitSlice {\n");
    for method in methods {
        extracted.push_str(method);
        extracted.push('\n');
    }
    extracted.push_str("}\n");
    extracted.push_str(from_initialized);
    extracted.push_str("\n}\n");
    extracted.push_str(from_maybe_uninit);
    extracted.push_str("\n}\n");

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_uninit_slice.rs"), extracted).unwrap();
}
