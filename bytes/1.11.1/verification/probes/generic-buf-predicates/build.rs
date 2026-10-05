use std::{env, fs, path::PathBuf};

fn item(source: &str, marker: &str) -> String {
    let start = source.find(marker).unwrap_or_else(|| panic!("source item missing: {marker}"));
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

fn main() {
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../..");
    let buf_path = crate_root.join("src/buf/buf_impl.rs");
    let buf_mut_path = crate_root.join("src/buf/buf_mut.rs");
    println!("cargo:rerun-if-changed={}", buf_path.display());
    println!("cargo:rerun-if-changed={}", buf_mut_path.display());

    let buf_source = fs::read_to_string(&buf_path).unwrap();
    let buf_trait = item(&buf_source, "pub trait Buf {");
    let has_remaining = item(&buf_trait, "fn has_remaining(&self) -> bool");
    let slice_impl = item(&buf_source, "impl Buf for &[u8]");
    let remaining = item(&slice_impl, "fn remaining(&self) -> usize");

    let buf_mut_source = fs::read_to_string(&buf_mut_path).unwrap();
    let buf_mut_trait = item(&buf_mut_source, "pub unsafe trait BufMut {");
    let has_remaining_mut = item(&buf_mut_trait, "fn has_remaining_mut(&self) -> bool");
    let slice_mut_impl = item(&buf_mut_source, "unsafe impl BufMut for &mut [u8]");
    let remaining_mut = item(&slice_mut_impl, "fn remaining_mut(&self) -> usize");

    // Round-trip source check. These bodies are inserted unchanged below.
    assert_eq!(item(&has_remaining, "fn has_remaining(&self) -> bool"), has_remaining);
    assert_eq!(item(&has_remaining_mut, "fn has_remaining_mut(&self) -> bool"), has_remaining_mut);
    assert_eq!(item(&remaining, "fn remaining(&self) -> usize"), remaining);
    assert_eq!(item(&remaining_mut, "fn remaining_mut(&self) -> usize"), remaining_mut);

    let wrong = env::var_os("CARGO_FEATURE_WRONG_PREDICATE").is_some();
    let buf_post = if wrong {
        "result == (self.remaining_metadata() == 0)"
    } else {
        "result == (self.remaining_metadata() > 0)"
    };
    let out = format!(r#"
use creusot_std::prelude::*;

/// Deliberately narrow proof interface. This is not the public bytes::Buf trait.
pub trait BufPredicateSurface {{
    #[logic]
    fn remaining_metadata(&self) -> Int;

    #[ensures(result@ == self.remaining_metadata())]
    fn remaining(&self) -> usize;

    #[ensures({buf_post})]
    {has_remaining}
}}

/// Deliberately narrow proof interface. This is not the public bytes::BufMut trait.
pub trait BufMutPredicateSurface {{
    #[logic]
    fn remaining_mut_metadata(&self) -> Int;

    #[ensures(result@ == self.remaining_mut_metadata())]
    fn remaining_mut(&self) -> usize;

    #[ensures(result == (self.remaining_mut_metadata() > 0))]
    {has_remaining_mut}
}}

impl<'a> BufPredicateSurface for &'a [u8] {{
    #[logic(open)]
    fn remaining_metadata(&self) -> Int {{ pearlite! {{ (*self)@.len() }} }}

    #[ensures(result@ == self.remaining_metadata())]
    {remaining}
}}

impl<'a> BufMutPredicateSurface for &'a mut [u8] {{
    #[logic(open)]
    fn remaining_mut_metadata(&self) -> Int {{ pearlite! {{ (*self)@.len() }} }}

    #[ensures(result@ == self.remaining_mut_metadata())]
    {remaining_mut}
}}

/// A generic caller uses the default method through the probe trait.
#[ensures(result == (buf.remaining_metadata() > 0))]
pub fn generic_buf_has_remaining<B: BufPredicateSurface + ?Sized>(buf: &B) -> bool {{
    buf.has_remaining()
}}

/// Concrete slice caller for the exact #[path]-extracted `&[u8]` implementation.
#[ensures(result == (input@.len() > 0))]
pub fn slice_has_remaining(input: &[u8]) -> bool {{
    input.has_remaining()
}}

/// A generic caller uses the default method through the probe trait.
#[ensures(result == (buf.remaining_mut_metadata() > 0))]
pub fn generic_bufmut_has_remaining<B: BufMutPredicateSurface + ?Sized>(buf: &B) -> bool {{
    buf.has_remaining_mut()
}}

/// Concrete mutable-slice caller for the exact #[path]-extracted implementation.
#[ensures(result == (input@.len() > 0))]
pub fn slice_mut_has_remaining(input: &mut [u8]) -> bool {{
    input.has_remaining_mut()
}}
"#);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("actual_predicates.rs"), out).unwrap();
    fs::write(out_dir.join("source_fragments.txt"), format!(
        "// Exact source methods extracted from bytes 1.11.1.\n\n{has_remaining}\n\n{has_remaining_mut}\n\n{remaining}\n\n{remaining_mut}\n"
    )).unwrap();
}
