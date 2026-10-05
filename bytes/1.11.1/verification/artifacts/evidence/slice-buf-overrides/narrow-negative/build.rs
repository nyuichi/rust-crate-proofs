use std::{env, fs, path::PathBuf};
fn item(source: &str, marker: &str) -> String {
    let start = source.find(marker).expect("exact source item missing");
    let open = start + source[start..].find('{').unwrap();
    let mut depth = 0;
    for (offset, ch) in source[open..].char_indices() {
        if ch == '{' { depth += 1; }
        if ch == '}' { depth -= 1; if depth == 0 { return source[start..open+offset+1].to_owned(); } }
    }
    panic!("unterminated source item")
}
fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../..");
    let buf_path = root.join("src/buf/buf_impl.rs");
    let lib_path = root.join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", buf_path.display());
    println!("cargo:rerun-if-changed={}", lib_path.display());
    let source = fs::read_to_string(&buf_path).unwrap();
    let implementation = item(&source, "impl Buf for &[u8]");
    let declaration = item(&fs::read_to_string(&lib_path).unwrap(), "pub struct TryGetError");
    let mut out = format!("use creusot_std::prelude::*;\n#[derive(Debug)]\n{declaration}\n");
    let mut exact = format!("{declaration}\n");
    let widths = [("try_get_u8", "u8", 1, "input@[0]@"),
        ("try_get_u16", "u16", 2, "input@[0]@ * 256 + input@[1]@"),
        ("try_get_u32", "u32", 4, "input@[0]@ * 16777216 + input@[1]@ * 65536 + input@[2]@ * 256 + input@[3]@")];
    for (name, ty, width, value) in widths {
        let original = item(&implementation, &format!("fn {name}(&mut self)"));
        let adapted = original.replace("(&mut self)", "(input: &mut &[u8])").replace("self", "input");
        assert_eq!(adapted.replace("(input: &mut &[u8])", "(&mut self)").replace("input", "self"), original);
        assert!(original.contains(&format!("requested: {width},")));
        assert!(original.contains(&format!("-> Result<{ty}, TryGetError>")));
        out.push_str(&format!("#[ensures(match result {{ Ok(value) => input@.len() >= {width} && value@ == {value} && (^input)@ == input@[{width}..], Err(error) => input@.len() < {width} && error.requested == {width}usize && error.available@ == input@.len() && (^input)@ == input@ }})]\npub {adapted}\n"));
        exact.push_str(&format!("\n{original}\n"));
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("actual_slice_overrides.rs"), out).unwrap();
    fs::write(out_dir.join("source_fragments.txt"), exact).unwrap();
}
