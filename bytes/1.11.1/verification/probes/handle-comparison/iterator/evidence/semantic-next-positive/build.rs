use std::{env, path::PathBuf};
#[path = "../build_iter.rs"] mod build_iter;
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
    println!("cargo:rerun-if-changed=../build_iter.rs");
    let root=PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../../..");
    build_iter::generate_from(root);
}
