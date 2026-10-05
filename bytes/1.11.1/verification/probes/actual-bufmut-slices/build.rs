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
    panic!("unterminated item at byte {open}");
}

fn unique_index(source: &str, marker: &str) -> usize {
    let first = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing source marker: {marker}"));
    assert!(
        source[first + marker.len()..].find(marker).is_none(),
        "duplicate source marker: {marker}"
    );
    first
}

fn extract_item<'a>(source: &'a str, start_marker: &str, body_marker: &str) -> &'a str {
    let start = unique_index(source, start_marker);
    let body = unique_index(source, body_marker);
    assert!(start <= body, "item prefix follows body marker");
    let open = body + source[body..].find('{').expect("item body opening brace");
    &source[start..matching_brace(source, open)]
}

fn extract_scoped_method<'a>(source: &'a str, impl_marker: &str, method: &str) -> &'a str {
    let impl_start = unique_index(source, impl_marker);
    let method_start = impl_start
        + source[impl_start..]
            .find(method)
            .unwrap_or_else(|| panic!("missing method `{method}` after `{impl_marker}`"));
    let open = method_start + source[method_start..].find('{').expect("method body");
    &source[method_start..matching_brace(source, open)]
}

fn normalized_body(item: &str) -> String {
    let open = item.find('{').expect("function body");
    let end = matching_brace(item, open);
    item[open..end]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let crate_root = manifest.join("../../..");
    let buf_mut_path = crate_root.join("src/buf/buf_mut.rs");
    let uninit_path = crate_root.join("src/buf/uninit_slice.rs");
    let slice_ops_path = crate_root.join("src/slice_mut_ops.rs");
    let lib_path = crate_root.join("src/lib.rs");
    for path in [&buf_mut_path, &uninit_path, &slice_ops_path, &lib_path] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");

    let buf_mut = fs::read_to_string(&buf_mut_path).unwrap();
    let uninit = fs::read_to_string(&uninit_path).unwrap();
    let slice_ops = fs::read_to_string(&slice_ops_path).unwrap();
    let lib = fs::read_to_string(&lib_path).unwrap();

    let impl_u8 = "unsafe impl BufMut for &mut [u8] {";
    let impl_maybe = "unsafe impl BufMut for &mut [core::mem::MaybeUninit<u8>] {";
    let u8_remaining =
        extract_scoped_method(&buf_mut, impl_u8, "fn remaining_mut(&self) -> usize {");
    let u8_chunk = extract_scoped_method(
        &buf_mut,
        impl_u8,
        "fn chunk_mut(&mut self) -> &mut UninitSlice {",
    );
    let u8_advance = extract_scoped_method(
        &buf_mut,
        impl_u8,
        "unsafe fn advance_mut(&mut self, cnt: usize) {",
    );
    let maybe_remaining =
        extract_scoped_method(&buf_mut, impl_maybe, "fn remaining_mut(&self) -> usize {");
    let maybe_chunk = extract_scoped_method(
        &buf_mut,
        impl_maybe,
        "fn chunk_mut(&mut self) -> &mut UninitSlice {",
    );
    let maybe_advance = extract_scoped_method(
        &buf_mut,
        impl_maybe,
        "unsafe fn advance_mut(&mut self, cnt: usize) {",
    );

    let struct_start = unique_index(&uninit, "#[repr(transparent)]\npub struct UninitSlice(");
    let struct_end = struct_start + uninit[struct_start..].find(';').unwrap() + 1;
    let extracted_uninit_struct = &uninit[struct_start..struct_end];
    let view_start = unique_index(&uninit, "impl View for UninitSlice {");
    let view_open = view_start + uninit[view_start..].find('{').unwrap();
    let view_impl = &uninit[view_start..matching_brace(&uninit, view_open)];
    let uninit_new = extract_item(
        &uninit,
        "    /// Creates a `&mut UninitSlice` wrapping a slice of initialised memory.",
        "    pub fn new(slice: &mut [u8]) -> &mut UninitSlice {",
    );
    let uninit_uninit = extract_item(
        &uninit,
        "    /// Creates a `&mut UninitSlice` wrapping a slice of uninitialised memory.",
        "    pub fn uninit(slice: &mut [MaybeUninit<u8>]) -> &mut UninitSlice {",
    );
    let write_byte = extract_item(
        &uninit,
        "    /// Write a single byte at the specified offset.",
        "    pub fn write_byte(&mut self, index: usize, byte: u8) {",
    );
    let len = extract_item(
        &uninit,
        "    /// Returns the number of bytes in the slice.",
        "    pub fn len(&self) -> usize {",
    );

    let advance_helper = extract_item(
        &slice_ops,
        "/// Advances the mutable-slice cursor and returns the unchanged consumed prefix.",
        "pub(crate) fn advance_slice_mut<'a, T>(input: &mut &'a mut [T], count: usize) -> &'a mut [T] {",
    );
    let error_struct = extract_item(&lib, "pub struct TryGetError {", "pub struct TryGetError {");
    let panic_helper = extract_item(
        &lib,
        "#[cold]\nfn panic_advance(error_info: &TryGetError) -> ! {",
        "fn panic_advance(error_info: &TryGetError) -> ! {",
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let uninit_output = format!(
        "use core::mem::MaybeUninit;\nuse creusot_std::prelude::*;\n\n{extracted_uninit_struct}\n\n{view_impl}\n\nimpl UninitSlice {{\n{uninit_new}\n\n{uninit_uninit}\n\n{write_byte}\n\n{len}\n}}\n"
    );
    fs::write(out_dir.join("actual_uninit_slice.rs"), &uninit_output).unwrap();

    let impl_output = format!(
        r#"{error_struct}

#[requires(false)]
{panic_helper}

pub(crate) mod slice_mut_ops {{
    use creusot_std::prelude::*;
    {advance_helper}
}}

impl ClosedU8SliceBufMut for &mut [u8] {{
    #[logic(open)]
    fn byte_view(&self) -> Seq<u8> {{ pearlite! {{ (**self)@ }} }}

    #[ensures(result@ == self.byte_view().len())]
    {u8_remaining}

    #[ensures(result@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> result@[i] == Some(self.byte_view()[i]))]
    #[ensures((^result)@.len() == self.byte_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.byte_view().len() ==> (^result)@[i] == Some((^self).byte_view()[i]))]
    #[ensures((^self).byte_view().len() == self.byte_view().len())]
    {u8_chunk}

    #[requires(cnt@ <= self.byte_view().len())]
    #[ensures((^self).byte_view() == self.byte_view()[cnt@..])]
    {u8_advance}
}}

impl ClosedMaybeUninitSliceBufMut for &mut [MaybeUninit<u8>] {{
    #[logic(open)]
    fn option_view(&self) -> Seq<Option<u8>> {{
        pearlite! {{ Seq::create((**self)@.len(), |index: Int| (**self)@[index]@) }}
    }}

    #[ensures(result@ == self.option_view().len())]
    {maybe_remaining}

    #[ensures(result@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> result@[i] == self.option_view()[i])]
    #[ensures((^result)@.len() == self.option_view().len())]
    #[ensures(forall<i> 0 <= i && i < self.option_view().len() ==> (^result)@[i] == (^self).option_view()[i])]
    #[ensures((^self).option_view().len() == self.option_view().len())]
    {maybe_chunk}

    #[requires(cnt@ <= self.option_view().len())]
    #[ensures((^self).option_view() == self.option_view()[cnt@..])]
    {maybe_advance}
}}
"#
    );
    fs::write(out_dir.join("actual_bufmut_slice_methods.rs"), &impl_output).unwrap();

    // The generated method items must retain each selected runtime body byte for
    // byte modulo whitespace; the surrounding contracts are probe-only specs.
    let generated_methods =
        fs::read_to_string(out_dir.join("actual_bufmut_slice_methods.rs")).unwrap();
    let generated_uninit = fs::read_to_string(out_dir.join("actual_uninit_slice.rs")).unwrap();
    for (name, source_item) in [
        ("UninitSlice struct", extracted_uninit_struct),
        ("UninitSlice View", view_impl),
        ("UninitSlice::new", uninit_new),
        ("UninitSlice::uninit", uninit_uninit),
        ("UninitSlice::write_byte", write_byte),
        ("UninitSlice::len", len),
    ] {
        assert!(
            generated_uninit.contains(source_item),
            "generated extraction differs from source for {name}"
        );
    }
    for (name, source_item) in [
        ("TryGetError", error_struct),
        ("panic_advance", panic_helper),
        ("advance_slice_mut", advance_helper),
    ] {
        assert!(
            generated_methods.contains(source_item),
            "generated dependency differs from source for {name}"
        );
    }
    for (impl_marker, method, source_item) in [
        (
            "impl ClosedU8SliceBufMut for &mut [u8] {",
            "fn remaining_mut(&self) -> usize {",
            u8_remaining,
        ),
        (
            "impl ClosedU8SliceBufMut for &mut [u8] {",
            "fn chunk_mut(&mut self) -> &mut UninitSlice {",
            u8_chunk,
        ),
        (
            "impl ClosedU8SliceBufMut for &mut [u8] {",
            "unsafe fn advance_mut(&mut self, cnt: usize) {",
            u8_advance,
        ),
        (
            "impl ClosedMaybeUninitSliceBufMut for &mut [MaybeUninit<u8>] {",
            "fn remaining_mut(&self) -> usize {",
            maybe_remaining,
        ),
        (
            "impl ClosedMaybeUninitSliceBufMut for &mut [MaybeUninit<u8>] {",
            "fn chunk_mut(&mut self) -> &mut UninitSlice {",
            maybe_chunk,
        ),
        (
            "impl ClosedMaybeUninitSliceBufMut for &mut [MaybeUninit<u8>] {",
            "unsafe fn advance_mut(&mut self, cnt: usize) {",
            maybe_advance,
        ),
    ] {
        let generated_item = extract_scoped_method(&generated_methods, impl_marker, method);
        assert_eq!(
            normalized_body(source_item),
            normalized_body(generated_item),
            "generated body differs from source for {method} in {impl_marker}"
        );
    }

    fs::write(
        out_dir.join("source_fragments.txt"),
        format!(
            "// Exact selected bodies from bytes 1.11.1 source.\n\n// UninitSlice new:\n{uninit_new}\n\n// UninitSlice uninit:\n{uninit_uninit}\n\n// UninitSlice write_byte:\n{write_byte}\n\n// UninitSlice len:\n{len}\n\n// advance_slice_mut dependency:\n{advance_helper}\n\n// BufMut for &mut [u8], remaining_mut:\n{u8_remaining}\n\n// BufMut for &mut [u8], chunk_mut:\n{u8_chunk}\n\n// BufMut for &mut [u8], advance_mut:\n{u8_advance}\n\n// BufMut for &mut [MaybeUninit<u8>], remaining_mut:\n{maybe_remaining}\n\n// BufMut for &mut [MaybeUninit<u8>], chunk_mut:\n{maybe_chunk}\n\n// BufMut for &mut [MaybeUninit<u8>], advance_mut:\n{maybe_advance}\n"
        ),
    )
    .unwrap();
}
