use std::{env, fs, path::PathBuf};
use crate::item;

pub fn generate() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../..");
    generate_from(root);
}

pub fn generate_from(root: PathBuf) {
    for name in ["src/buf/iter.rs", "src/buf/buf_impl.rs", "src/lib.rs"] {
        println!("cargo:rerun-if-changed={}", root.join(name).display());
    }
    let iterator = fs::read_to_string(root.join("src/buf/iter.rs")).unwrap();
    let buf = fs::read_to_string(root.join("src/buf/buf_impl.rs")).unwrap();
    let lib = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let slice = item(&buf, "impl Buf for &[u8]");
    let mut output = String::from("use creusot_std::prelude::*;\nuse creusot_std::std::iter::{IteratorSpec, ExactSizeIteratorSpec};\n");
    let mut exact = String::new();
    for (name, signature, contract) in [
        ("remaining", "fn remaining(input: &&[u8]) -> usize", "#[ensures(result@ == input@.len())]"),
        ("chunk", "fn chunk<'a>(input: &'a &[u8]) -> &'a [u8]", "#[ensures(result@ == input@)]"),
        ("advance", "fn advance(input: &mut &[u8], cnt: usize)", "#[requires(cnt@ <= input@.len())]\n#[ensures((^input)@ == input@[cnt@..])]"),
    ] {
        let original = item(&slice, &format!("fn {name}("));
        let body = &original[original.find('{').unwrap()..];
        let adapted = body.replace("self", "input");
        assert_eq!(adapted.replace("input", "self"), body);
        output.push_str(&format!("{contract}\n{signature} {adapted}\n"));
        exact.push_str(&format!("{original}\n"));
    }
    let defaults = &buf[..buf.find("macro_rules! deref_forward_buf").unwrap()];
    let original = item(defaults, "fn has_remaining(&self)");
    let body = &original[original.find('{').unwrap()..];
    let adapted = body.replace("self.remaining()", "remaining(input)");
    assert_eq!(adapted.replace("remaining(input)", "self.remaining()"), body);
    output.push_str(&format!("#[ensures(result == (input@.len() > 0))]\nfn has_remaining(input: &&[u8]) -> bool {adapted}\n"));
    exact.push_str(&format!("{original}\n"));
    output.push_str(&item(&lib, "pub struct TryGetError {"));
    output.push_str("\n#[requires(false)]\n");
    output.push_str(&item(&lib, "fn panic_advance("));
    output.push_str("\n");
    let declaration = item(&iterator, "pub struct IntoIter<T> {");
    output.push_str(&declaration);
    output.push_str("\nimpl<T> IntoIter<T> {\n");
    for (marker, contract) in [
        ("pub fn new(inner: T)", "#[ensures(result.inner == inner)]"),
        ("pub fn into_inner(self)", "#[ensures(result == self.inner)]"),
        ("pub fn get_ref(&self)", "#[ensures(*result == self.inner)]"),
        ("pub fn get_mut(&mut self)", "#[ensures(*result == self.inner && ^result == (^self).inner)]"),
    ] {
        let original = item(&iterator, marker);
        output.push_str(&format!("{contract}\n{original}\n"));
        exact.push_str(&format!("{original}\n"));
    }
    output.push_str("}\nimpl<'a> Iterator for IntoIter<&'a [u8]> {\ntype Item = u8;\n");
    for (marker, contracts) in [
        ("fn next(&mut self)", "#[ensures(match result { Some(byte) => self.inner@.len() > 0 && byte == self.inner@[0] && (^self).inner@ == self.inner@[1..], None => self.inner@.len() == 0 && (^self).inner@ == self.inner@ })]"),
        ("fn size_hint(&self)", "#[ensures(result.0@ == self.inner@.len() && result.1 == Some(result.0))]"),
    ] {
        let original = item(&iterator, marker);
        let pairs = [("self.inner.has_remaining()", "has_remaining(&self.inner)"),
            ("self.inner.chunk()", "chunk(&self.inner)"),
            ("self.inner.advance(1)", "advance(&mut self.inner, 1)"),
            ("self.inner.remaining()", "remaining(&self.inner)")];
        let mut adapted = original.clone();
        for (before, after) in pairs { adapted = adapted.replace(before, after); }
        let mut restored = adapted.clone();
        for (before, after) in pairs { restored = restored.replace(after, before); }
        assert_eq!(restored, original);
        output.push_str(&format!("{contracts}\n{adapted}\n"));
        exact.push_str(&format!("{original}\n"));
    }
    output.push_str(r#"
}
#[cfg(creusot)]
#[logic]
#[requires(bytes.len() > 0)]
#[ensures(bytes == Seq::singleton(bytes[0]).concat(bytes[1..]))]
fn sequence_head_tail(bytes: Seq<u8>) {}
#[cfg(creusot)]
impl<'a> IteratorSpec for IntoIter<&'a [u8]> {
    #[logic(prophetic)]
    fn produces(self, visited: Seq<u8>, other: Self) -> bool {
        let _ = sequence_head_tail;
        pearlite! { self.inner@ == visited.concat(other.inner@) }
    }
    #[logic(prophetic)]
    fn completed(&mut self) -> bool { pearlite! { self.inner@.len() == 0 } }
    #[logic(law)]
    #[ensures(self.produces(Seq::empty(), self))]
    fn produces_refl(self) { let _ = Seq::<u8>::concat_empty; }
    #[logic(law)]
    #[requires(a.produces(ab, b))]
    #[requires(b.produces(bc, c))]
    #[ensures(a.produces(ab.concat(bc), c))]
    fn produces_trans(a: Self, ab: Seq<u8>, b: Self, bc: Seq<u8>, c: Self) {
        let _ = Seq::<u8>::concat_assoc;
    }
}
impl<'a> ExactSizeIterator for IntoIter<&'a [u8]> {}
#[cfg(creusot)]
impl<'a> ExactSizeIteratorSpec for IntoIter<&'a [u8]> {
    #[logic(law)]
    #[requires(Self::size_hint.postcondition((self,), hint))]
    #[ensures(hint.1 == Some(hint.0))]
    fn size_hint_exact(&self, hint: (usize, Option<usize>)) {}
}
"#);
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("actual_iterator.rs"), output).unwrap();
    fs::write(out.join("iterator_exact_bodies.rs"), exact).unwrap();
}
