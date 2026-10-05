use std::{env,fs,path::PathBuf};
fn main(){
 let manifest=PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
 let root=manifest.join("../../../src").canonicalize().unwrap();
 let path=root.join("ownership_proof/raw_vec.rs");
 println!("cargo:rerun-if-changed={}",path.display());
 let source=fs::read_to_string(path).unwrap();
 let signature="pub(crate) unsafe fn borrow_bound<'a>(";
 assert_eq!(source.matches(signature).count(),1);
 // Production now carries the reviewed annotation. Preserve the exact body.
 let annotation=format!("#[cfg_attr(creusot, check(ghost))]\n{signature}");
 assert!(source.contains(&annotation));
 let changed=source;
 let relocated=changed.replace("#[path = \"../allocation_ops.rs\"]", &format!("#[path = {:?}]",root.join("allocation_ops.rs")));
 let out=PathBuf::from(env::var_os("OUT_DIR").unwrap());
 fs::write(out.join("readonly_raw_vec.rs"),relocated).unwrap();
 fs::write(out.join("raw_module.rs"),format!("#[path={:?}] mod raw_vec;",out.join("readonly_raw_vec.rs"))).unwrap();
}
