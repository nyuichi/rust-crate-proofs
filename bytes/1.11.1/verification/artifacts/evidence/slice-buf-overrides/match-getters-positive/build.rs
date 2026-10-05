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
    let mut methods = vec![("try_get_u8".to_owned(), "u8".to_owned(), 1usize, "input@[0]@".to_owned())];
    for bits in [16, 32, 64, 128] {
        let width = bits / 8;
        for signed in [false, true] {
            for little in [false, true] {
                let ty = format!("{}{bits}", if signed { "i" } else { "u" });
                let suffix = if little { "_le" } else { "" };
                let mut terms = Vec::new();
                for index in 0..width {
                    let exponent = if little { index } else { width - 1 - index };
                    let factor = 256u128.pow(exponent as u32);
                    terms.push(if factor == 1 { format!("input@[{index}]@") } else { format!("input@[{index}]@ * {factor}") });
                }
                let unsigned = terms.join(" + ");
                let value = if signed {
                    format!("crate::{}::signed_u{bits}({unsigned})", if bits <= 32 { "endian_ops" } else { "signed_wide_ops" })
                } else { unsigned };
                methods.push((format!("try_get_{ty}{suffix}"), ty, width, value));
            }
        }
    }
    for (name, ty, width, value) in methods {
        let original = item(&implementation, &format!("fn {name}(&mut self)"));
        let adapted = original.replace("(&mut self)", "(input: &mut &[u8])").replace("self", "input");
        assert_eq!(adapted.replace("(input: &mut &[u8])", "(&mut self)").replace("input", "self"), original);
        assert!(original.contains(&format!("requested: {width},")));
        assert!(original.contains(&format!("-> Result<{ty}, TryGetError>")));
        out.push_str(&format!("#[ensures(match result {{ Ok(value) => input@.len() >= {width} && value@ == {value} && (^input)@ == input@[{width}..], Err(error) => input@.len() < {width} && error.requested == {width}usize && error.available@ == input@.len() && (^input)@ == input@ }})]\npub {adapted}\n"));
        exact.push_str(&format!("\n{original}\n"));
        let getter = name.replacen("try_", "", 1);
        let original_getter = item(&implementation, &format!("fn {getter}(&mut self)"));
        let adapted_getter = original_getter.replace("(&mut self)", "(input: &mut &[u8])").replace("self", "input")
            .replace(&format!("input.{name}()"), &format!("{name}(input)"));
        let inverse = adapted_getter.replace(&format!("{name}(input)"), &format!("input.{name}()"))
            .replace("(input: &mut &[u8])", "(&mut self)").replace("input", "self");
        assert_eq!(inverse, original_getter);
        out.push_str(&format!("#[requires(input@.len() >= {width})]\n#[ensures(result@ == {value} && (^input)@ == input@[{width}..])]\npub {adapted_getter}\n"));
        exact.push_str(&format!("\n{original_getter}\n"));
    }
    let lib_source = fs::read_to_string(&lib_path).unwrap();
    for name in ["panic_advance", "panic_does_not_fit"] {
        let original = item(&lib_source, &format!("fn {name}("));
        out.push_str(&format!("#[requires(false)]\n{original}\n"));
        exact.push_str(&format!("\n{original}\n"));
    }
    for (name, signature, contract) in [
        ("remaining", "fn remaining(input: &&[u8]) -> usize", "result@ == input@.len()"),
        ("chunk", "fn chunk<'a>(input: &'a &[u8]) -> &'a [u8]", "result@ == input@"),
    ] {
        let original = item(&implementation, &format!("fn {name}(&self)"));
        let open = original.find('{').unwrap();
        let adapted = format!("{signature} {}", original[open..].replace("self", "input"));
        assert_eq!(adapted[adapted.find('{').unwrap()..].replace("input", "self"), original[open..]);
        out.push_str(&format!("#[ensures({contract})]\npub {adapted}\n"));
        exact.push_str(&format!("\n{original}\n"));
    }
    for little in [false, true] {
        let suffix = if little { "_le" } else { "" };
        let name = format!("try_get_uint{suffix}");
        let original = item(&implementation, &format!("fn {name}(&mut self, nbytes: usize)"));
        let adapted = original.replace("(&mut self,", "(input: &mut &[u8],").replace("self", "input");
        assert_eq!(adapted.replace("(input: &mut &[u8],", "(&mut self,").replace("input", "self"), original);
        let value = format!("crate::variable_read_ops::{}_weight(input@, nbytes@)", if little { "le" } else { "be" });
        out.push_str(&format!("#[requires(nbytes <= 8usize)]\n#[ensures(match result {{ Ok(value) => input@.len() >= nbytes@ && value@ == {value} && (^input)@ == input@[nbytes@..], Err(error) => input@.len() < nbytes@ && error.requested == nbytes && error.available@ == input@.len() && (^input)@ == input@ }})]\npub {adapted}\n"));
        exact.push_str(&format!("\n{original}\n"));
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("actual_slice_overrides.rs"), out).unwrap();
    fs::write(out_dir.join("source_fragments.txt"), exact).unwrap();
}
