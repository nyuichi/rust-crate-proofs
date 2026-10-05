use std::{env, fs, path::PathBuf};
mod build_iter;
mod build_readonly;

mod valid_handle {
    include!("../valid-handle-traits/build.rs");
    pub fn extract() { main(); }
}

fn item(source: &str, marker: &str) -> String {
    let start = source.find(marker).expect(marker);
    assert!(!source[start + marker.len()..].contains(marker));
    let open = start + source[start..].find('{').unwrap();
    let mut depth = 0;
    for (offset, ch) in source[open..].char_indices() {
        if ch == '{' { depth += 1; }
        if ch == '}' {
            depth -= 1;
            if depth == 0 { return source[start..open + offset + 1].to_owned(); }
        }
    }
    panic!("unterminated {marker}")
}

fn main() {
    if env::var_os("CARGO_FEATURE_CONCRETE_ITERATOR").is_some() { build_iter::generate(); }
    valid_handle::extract();
    println!("cargo:rerun-if-changed=../valid-handle-traits/build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let path = out.join("actual_traits.rs");
    let mut generated = fs::read_to_string(&path).unwrap();
    let source = fs::read_to_string(PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../../src/bytes_mut.rs")).unwrap();
    // The common initialized predicate now accepts advanced unique handles;
    // retain its exact dependency while this gate keeps the stronger invariant.
    if !generated.contains("fn proof_unique_owned(self)") {
        let owned = item(&source, "fn proof_unique_owned(self) -> bool {");
        generated.push_str(&format!("impl BytesMut {{\n#[cfg(creusot)]\n#[logic(prophetic)]\n{owned}\n}}\n"));
    }
    if env::var_os("CARGO_FEATURE_READONLY_DEREF").is_some() {
        build_readonly::generate(&source, &mut generated);
    }
    // Additional contracts, with exact source bodies retained byte-for-byte.
    generated = generated.replace("    fn as_slice(&self) -> &[u8] {",
        "    #[cfg_attr(creusot, ensures(result.deep_model() == self.deep_model()))]\n    fn as_slice(&self) -> &[u8] {");
    generated = generated.replace("    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {",
        "    #[cfg_attr(creusot, ensures(result.deep_model() == vec.deep_model()))]\n    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {");
    let model = r#"
#[cfg(creusot)]
impl BytesMut {
    // Total outside the invariant too: Unknown/absent slots map to zero, but
    // this assigns no ownership or initialization. Valid handles prove Known.
    #[logic]
    pub fn proof_byte_model(self) -> Seq<Int> {
        pearlite! { Seq::create(self.len@, |index| match self.proof_view_slot(index) {
            Some(Some(byte)) => byte@,
            _ => 0int,
        }) }
    }
}
#[cfg(creusot)]
impl DeepModel for BytesMut {
    type DeepModelTy = Seq<Int>;
    #[logic]
    fn deep_model(self) -> Seq<Int> { self.proof_byte_model() }
}
"#;
    generated.push_str(model);
    let mut exact = String::new();
    generated.push_str("impl BytesMut {\n");
    for (name, rhs, equality) in [
        ("__creusot_eq_bytes_mut", "other.deep_model()", true),
        ("__creusot_cmp_bytes_mut", "other.deep_model()", false),
        ("__creusot_eq_slice", "other.deep_model()", true),
        ("__creusot_cmp_slice", "other.deep_model()", false),
    ] {
        let original = item(&source, &format!("pub fn {name}("));
        let condition = if equality { format!("result == (self.deep_model() == {rhs})") }
            else { format!("result == self.deep_model().cmp_log({rhs})") };
        generated.push_str("#[cfg_attr(creusot, requires(self.proof_initialized()))]\n");
        if name.ends_with("_bytes_mut") {
            generated.push_str("#[cfg_attr(creusot, requires(other.proof_initialized()))]\n");
        }
        generated.push_str(&format!("#[cfg_attr(creusot, ensures({condition}))]\n{original}\n"));
        exact.push_str(&format!("{original}\n"));
    }
    if env::var_os("CARGO_FEATURE_STR_ADAPTERS").is_some() {
        for (name, equality) in [("__creusot_eq_str", true), ("__creusot_cmp_str", false)] {
            let original = item(&source, &format!("pub fn {name}("));
            let rhs = "other@.to_bytes().map(|byte: u8| byte@)";
            let condition = if equality { format!("result == (self.deep_model() == {rhs})") }
                else { format!("result == self.deep_model().cmp_log({rhs})") };
            generated.push_str(&format!("#[cfg_attr(creusot, requires(self.proof_initialized()))]\n#[cfg_attr(creusot, ensures({condition}))]\n{original}\n"));
            exact.push_str(&format!("{original}\n"));
        }
    }
    generated.push_str("}\n");
    for marker in ["impl PartialEq for BytesMut {", "impl PartialOrd for BytesMut {",
        "impl Ord for BytesMut {", "impl Eq for BytesMut {}"] {
        let original = item(&source, marker);
        generated.push_str(&format!("#[cfg(feature = \"actual-traits\")]\n{original}\n"));
        exact.push_str(&format!("{original}\n"));
    }
    generated.push_str(r#"
#[cfg_attr(creusot, ensures(result.0 == (left.deep_model() == right.deep_model())))]
#[cfg_attr(creusot, ensures(result.1 == left.deep_model().cmp_log(right.deep_model())))]
pub fn compare_unique(left: Vec<u8>, right: Vec<u8>) -> (bool, cmp::Ordering) {
    let left = BytesMut::from_vec(left);
    let right = BytesMut::from_vec(right);
    #[cfg(not(feature = "actual-traits"))]
    let answer = (left.__creusot_eq_bytes_mut(&right), left.__creusot_cmp_bytes_mut(&right));
    #[cfg(feature = "actual-traits")]
    let answer = (left == right, left.cmp(&right));
    left.proof_release_unique_at_zero();
    right.proof_release_unique_at_zero();
    answer
}
#[cfg(all(creusot, feature = "negative_wrong_equality"))]
#[requires(left.deep_model() != right.deep_model())]
#[ensures(result)]
pub fn reject_wrong_equality(left: &BytesMut, right: &BytesMut) -> bool {
    left.__creusot_eq_bytes_mut(right)
}
#[cfg(feature = "str-adapters")]
#[cfg_attr(creusot, ensures(result.0 == (left.deep_model() == right@.to_bytes().map(|byte: u8| byte@))))]
#[cfg_attr(creusot, ensures(result.1 == left.deep_model().cmp_log(right@.to_bytes().map(|byte: u8| byte@))))]
pub fn compare_text_unique(left: Vec<u8>, right: &str) -> (bool, cmp::Ordering) {
    let left = BytesMut::from_vec(left);
    let result = (left.__creusot_eq_str(right), left.__creusot_cmp_str(right));
    left.proof_release_unique_at_zero();
    result
}
"#);
    fs::write(path, generated).unwrap();
    fs::write(out.join("comparison_exact_bodies.rs"), exact).unwrap();
    fs::write(out.join("comparison_model.rs"), model).unwrap();
    fs::write(out.join("bytes_mut_source_snapshot.rs"), source).unwrap();
}
