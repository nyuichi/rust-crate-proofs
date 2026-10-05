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
    let body = start
        + source[start..]
            .find(body_marker)
            .expect("impl method body marker");
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

fn function_body(source: &str, signature: &str) -> String {
    let start = unique_index(source, signature);
    let open = start
        + source[start..]
            .find('{')
            .expect("function body opening brace");
    let end = matching_brace(source, open);
    source[open..end]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source_path = manifest.join("../../../src/buf/uninit_slice.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_REMAINING_APIS");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_FULL_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_BOUNDS_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_FROM_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_TO_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_TO_INCLUSIVE_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RANGE_INCLUSIVE_INDEX");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BOUND_RAW");
    println!("cargo:rustc-check-cfg=cfg(bytes_proof_bound_uninit_raw)");
    let source = fs::read_to_string(&source_path).unwrap();
    let remaining_apis = env::var_os("CARGO_FEATURE_REMAINING_APIS").is_some();
    let range_full_index = env::var_os("CARGO_FEATURE_RANGE_FULL_INDEX").is_some();
    let range_bounds_index = env::var_os("CARGO_FEATURE_RANGE_BOUNDS_INDEX").is_some();
    let range_from_index = env::var_os("CARGO_FEATURE_RANGE_FROM_INDEX").is_some();
    let range_to_index = env::var_os("CARGO_FEATURE_RANGE_TO_INDEX").is_some();
    let range_to_inclusive_index =
        env::var_os("CARGO_FEATURE_RANGE_TO_INCLUSIVE_INDEX").is_some();
    let range_inclusive_index = env::var_os("CARGO_FEATURE_RANGE_INCLUSIVE_INDEX").is_some();
    let bound_raw = env::var_os("CARGO_FEATURE_BOUND_RAW").is_some();
    if bound_raw {
        println!("cargo:rustc-cfg=bytes_proof_bound_uninit_raw");
    }
    let range_family_count = [
        range_from_index,
        range_to_index,
        range_to_inclusive_index,
        range_inclusive_index,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    assert!(range_family_count <= 1, "select only one range family at a time");
    let range_family_index = range_family_count == 1;

    let struct_start = unique_index(&source, "#[repr(transparent)]\npub struct UninitSlice(");
    let struct_end = struct_start + source[struct_start..].find(';').unwrap() + 1;
    let struct_item = &source[struct_start..struct_end];
    let view_impl = extract_block(&source, "impl View for UninitSlice {");
    let mut methods = vec![
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
    if remaining_apis || range_full_index || range_bounds_index || range_family_index {
        methods.push(extract_item(
            &source,
            "    #[trusted]\n    #[cfg_attr(creusot, check(ghost))]\n    #[ensures(result@.len() == slice@.len())]\n    #[ensures(result@ == Seq::create(slice@.len(), |index: Int| slice@[index]@))]\n    #[ensures(forall<i> 0 <= i && i < slice@.len() ==> result@[i] == slice@[i]@)]\n    fn uninit_ref(slice: &[MaybeUninit<u8>]) -> &UninitSlice {",
            "    fn uninit_ref(slice: &[MaybeUninit<u8>]) -> &UninitSlice {",
        ));
    }
    if remaining_apis {
        methods.push(extract_item(
            &source,
            "    /// Return a raw pointer to the slice's buffer.",
            "    pub fn as_mut_ptr(&mut self) -> *mut u8 {",
        ));
        methods.push(extract_item(
            &source,
            "    /// Return a `&mut [MaybeUninit<u8>]` to this slice's buffer.",
            "    pub unsafe fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<u8>] {",
        ));
    }
    if bound_raw {
        methods.push(extract_item(
            &source,
            "    #[cfg(bytes_proof_bound_uninit_raw)]\n    #[inline]\n    #[requires(ptr == bound.raw_pointer())]",
            "    pub unsafe fn from_raw_parts_mut<'a>(\n        ptr: *mut u8,\n",
        ));
    }
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

    let mut extracted = String::from("use core::mem::MaybeUninit;\nuse creusot_std::prelude::*;\n");
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
    if range_full_index || range_bounds_index || range_family_index {
        let (index_signature, index_mut_signature, expansion) = if range_full_index {
            (
                "fn index(&self, index: RangeFull) -> &UninitSlice {",
                "fn index_mut(&mut self, index: RangeFull) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, RangeFull};\nimpl_index!(@range_full);\n",
            )
        } else if range_bounds_index {
            (
                "fn index(&self, index: Range<usize>) -> &UninitSlice {",
                "fn index_mut(&mut self, index: Range<usize>) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, Range};\nimpl_index!(@range_bounds);\n",
            )
        } else if range_from_index {
            (
                "fn index(&self, index: $index_ty) -> &UninitSlice {",
                "fn index_mut(&mut self, index: $index_ty) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, RangeFrom};\nimpl_index!(@range_family RangeFrom<usize>);\n",
            )
        } else if range_to_index {
            (
                "fn index(&self, index: $index_ty) -> &UninitSlice {",
                "fn index_mut(&mut self, index: $index_ty) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, RangeTo};\nimpl_index!(@range_family RangeTo<usize>);\n",
            )
        } else if range_to_inclusive_index {
            (
                "fn index(&self, index: $index_ty) -> &UninitSlice {",
                "fn index_mut(&mut self, index: $index_ty) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, RangeToInclusive};\nimpl_index!(@range_family RangeToInclusive<usize>);\n",
            )
        } else {
            (
                "fn index(&self, index: $index_ty) -> &UninitSlice {",
                "fn index_mut(&mut self, index: $index_ty) -> &mut UninitSlice {",
                "\nuse core::ops::{Index, IndexMut, RangeInclusive};\nimpl_index!(@range_family RangeInclusive<usize>);\n",
            )
        };
        assert_eq!(
            function_body(&source, "fn index(&self, index: $t) -> &UninitSlice {",),
            function_body(&source, index_signature),
            "proof Index body must match the runtime macro body",
        );
        assert_eq!(
            function_body(
                &source,
                "fn index_mut(&mut self, index: $t) -> &mut UninitSlice {",
            ),
            function_body(
                &source,
                index_mut_signature,
            ),
            "proof IndexMut body must match the runtime macro body",
        );
        if range_family_index {
            extracted.push_str(&extract_block(
                &source,
                "#[cfg(creusot)]\nmod uninit_index_model {",
            ));
            extracted.push_str("\n#[cfg(creusot)]\nuse uninit_index_model::UninitSliceIndexModel;\n");
        }
        extracted.push_str(&extract_block(&source, "macro_rules! impl_index {"));
        extracted.push_str(expansion);
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_uninit_slice.rs"), extracted).unwrap();
}
