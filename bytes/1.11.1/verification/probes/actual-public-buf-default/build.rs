use std::{env, fs, path::PathBuf};

fn item(source: &str, marker: &str) -> String {
    let start = source.find(marker).unwrap_or_else(|| panic!("missing source item: {marker}"));
    let open = start + source[start..].find('{').unwrap();
    let mut depth = 0;
    for (offset, ch) in source[open..].char_indices() {
        if ch == '{' { depth += 1; }
        if ch == '}' {
            depth -= 1;
            if depth == 0 { return source[start..open + offset + 1].to_owned(); }
        }
    }
    panic!("unterminated source item: {marker}")
}

fn declaration(source: &str, marker: &str) -> String {
    let start = source.find(marker).unwrap_or_else(|| panic!("missing declaration: {marker}"));
    let end = source[start..].find(';').map(|offset| start + offset + 1)
        .unwrap_or_else(|| panic!("unterminated declaration: {marker}"));
    source[start..end].to_owned()
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../..");
    let buf_path = root.join("src/buf/buf_impl.rs");
    let lib_path = root.join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", buf_path.display());
    println!("cargo:rerun-if-changed={}", lib_path.display());

    let source = fs::read_to_string(&buf_path).unwrap();
    let public_trait = item(&source, "pub trait Buf {");
    let public_impl = item(&source, "impl Buf for &[u8]");
    let remaining = declaration(&public_trait, "fn remaining(&self) -> usize");
    let chunk = declaration(&public_trait, "fn chunk(&self) -> &[u8]");
    let advance = declaration(&public_trait, "fn advance(&mut self, cnt: usize)");
    let try_get_u8 = item(&public_trait, "fn try_get_u8(&mut self) -> Result<u8, TryGetError>");
    let slice_remaining = item(&public_impl, "fn remaining(&self) -> usize");
    let slice_chunk = item(&public_impl, "fn chunk(&self) -> &[u8]");
    let slice_advance = item(&public_impl, "fn advance(&mut self, cnt: usize)");

    // Round-trip guard: the default read and selected implementation bodies
    // below must stay byte-for-byte identical to the current public source.
    for (extracted, marker) in [
        (&remaining, "fn remaining(&self) -> usize"),
        (&chunk, "fn chunk(&self) -> &[u8]"),
        (&advance, "fn advance(&mut self, cnt: usize)"),
        (&try_get_u8, "fn try_get_u8(&mut self) -> Result<u8, TryGetError>"),
    ] {
        let actual = if marker.contains("try_get_u8") {
            item(&public_trait, marker)
        } else {
            declaration(&public_trait, marker)
        };
        assert_eq!(actual, *extracted);
    }
    for (extracted, marker) in [
        (&slice_remaining, "fn remaining(&self) -> usize"),
        (&slice_chunk, "fn chunk(&self) -> &[u8]"),
        (&slice_advance, "fn advance(&mut self, cnt: usize)"),
    ] {
        assert_eq!(item(&public_impl, marker), *extracted);
    }

    let lib_source = fs::read_to_string(&lib_path).unwrap();
    let error = item(&lib_source, "pub struct TryGetError");
    let panic_advance = item(&lib_source, "fn panic_advance(");
    let out = format!(r#"
use creusot_std::prelude::*;

#[derive(core::fmt::Debug)]
{error}

#[requires(false)]
{panic_advance}

/// A source-sliced copy of the actual public `Buf` trait surface needed by the
/// default `try_get_u8`. The runtime method signatures and bodies are extracted
/// exactly from `src/buf/buf_impl.rs`; `unread` and the contracts are proof-only
/// additions for evaluating the minimum public trait law model.
pub trait Buf {{
    #[logic]
    fn unread(&self) -> Seq<u8>;

    #[ensures(result@ == self.unread().len())]
    {remaining}

    #[ensures(result@.len() <= self.unread().len())]
    #[ensures(result@ == self.unread().subsequence(0, result@.len()))]
    #[ensures(self.unread().len() > 0 ==> result@.len() > 0)]
    {chunk}

    #[requires(cnt@ <= self.unread().len())]
    #[ensures((^self).unread() == self.unread().subsequence(cnt@, self.unread().len()))]
    {advance}

    #[ensures(match result {{
        Ok(value) => self.unread().len() > 0
            && value == self.unread()[0]
            && (^self).unread() == self.unread().subsequence(1, self.unread().len()),
        Err(error) => self.unread().len() == 0
            && error.requested == 1usize
            && error.available@ == 0
            && (^self).unread() == self.unread(),
    }})]
    {try_get_u8}
}}

impl<'a> Buf for &'a [u8] {{
    #[logic(open)]
    fn unread(&self) -> Seq<u8> {{ pearlite! {{ (*self)@ }} }}

    #[ensures(result@ == self.unread().len())]
    {slice_remaining}

    #[ensures(result@.len() <= self.unread().len())]
    #[ensures(result@ == self.unread().subsequence(0, result@.len()))]
    #[ensures(self.unread().len() > 0 ==> result@.len() > 0)]
    {slice_chunk}

    #[requires(cnt@ <= self.unread().len())]
    #[ensures((^self).unread() == self.unread().subsequence(cnt@, self.unread().len()))]
    {slice_advance}
}}

/// Generic dispatch through the extracted public-trait default.
#[ensures(match result {{
    Ok(value) => input.unread().len() > 0
        && value == input.unread()[0]
        && (^input).unread() == input.unread().subsequence(1, input.unread().len()),
    Err(error) => input.unread().len() == 0
        && error.requested == 1usize && error.available@ == 0
        && (^input).unread() == input.unread(),
}})]
pub fn generic_try_get_u8<B: Buf + ?Sized>(input: &mut B) -> Result<u8, TryGetError> {{
    input.try_get_u8()
}}

/// Concrete call through the selected actual `&[u8]` implementation.
#[ensures(match result {{
    Ok(value) => input.unread().len() > 0
        && value == input.unread()[0]
        && (^input).unread() == input.unread().subsequence(1, input.unread().len()),
    Err(error) => input.unread().len() == 0
        && error.requested == 1usize && error.available@ == 0
        && (^input).unread() == input.unread(),
}})]
pub fn slice_try_get_u8(input: &mut &[u8]) -> Result<u8, TryGetError> {{
    generic_try_get_u8(input)
}}
"#);

    let source_fragments = format!(
        "// Exact public trait and selected implementation methods from bytes 1.11.1.\n\n{remaining}\n\n{chunk}\n\n{advance}\n\n{try_get_u8}\n\n{slice_remaining}\n\n{slice_chunk}\n\n{slice_advance}\n"
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("actual_public_buf.rs"), &out).unwrap();
    fs::write(out_dir.join("source_fragments.txt"), &source_fragments).unwrap();

    // Keep an inspectable source correspondence beside this probe as well as
    // under Cargo's disposable OUT_DIR.
    let extraction_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("extraction");
    fs::create_dir_all(&extraction_dir).unwrap();
    fs::write(extraction_dir.join("actual_public_buf.rs"), out).unwrap();
    fs::write(extraction_dir.join("source_fragments.txt"), source_fragments).unwrap();
}
