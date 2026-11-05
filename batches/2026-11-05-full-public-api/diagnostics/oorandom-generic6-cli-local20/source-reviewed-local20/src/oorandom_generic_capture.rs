//! Compiler-owned source capture for the finite six-family gate; validated locally.
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeSet,fs,path::{Path,PathBuf},process::Command};

fn digest(bytes:impl AsRef<[u8]>)->String { format!("{:x}",Sha256::digest(bytes.as_ref())) }
fn id(v:&Value)->Result<String> { if let Some(n)=v.as_u64(){Ok(n.to_string())}else{v.as_str().map(str::to_owned).context("Invalid compiler item ID")} }
fn item<'a>(doc:&'a Value,i:&Value)->Result<&'a Value> {doc["index"].get(id(i)?).context("Missing compiler item")}
fn path_id(doc:&Value,parts:&[&str],kind:&str)->Result<Value> {
    let found:Vec<_>=doc["paths"].as_object().context("Missing compiler paths")?.iter().filter(|(_,v)|v["path"]==json!(parts)&&v["kind"]==kind).collect();
    ensure!(found.len()==1,"Compiler path must resolve exactly once: {parts:?}");
    let (key,path)=found[0]; let external=&doc["external_crates"][id(&path["crate_id"])?];
    ensure!(parts[0]!="core" || external["name"]=="core","Foreign trait path is not original core");
    Ok(json!(key.parse::<u64>()?))
}
fn args(ty:Value)->Value {json!({"angle_bracketed":{"args":[{"type":ty}],"constraints":[]}})}
fn tyvar(name:&str)->Value {json!({"generic":name})}
fn trait_ref(doc:&Value,name:&str,arg:Value)->Result<Value> {Ok(json!({"path":name,"id":path_id(doc,&["core","convert",name],"trait")?,"args":args(arg)}))}
fn source_params()->Value {json!([{"name":"T","kind":{"type":{"bounds":[],"default":null,"is_synthetic":false}}},{"name":"U","kind":{"type":{"bounds":[],"default":null,"is_synthetic":false}}}])}

/// Fresh rustdoc membership validation. Owner + impl + method correspondence is
/// checked, so shared original method IDs across the two owners are not collapsed.
pub(super) fn validate_document(doc:&Value,expected_core_metadata:&Path)->Result<Vec<Value>> {
    ensure!(doc["crate_version"]=="11.1.5","Wrong original crate version");
    let root=item(doc,&doc["root"])?;ensure!(root["name"]=="oorandom","Wrong original crate root");
    let core_id=path_id(doc,&["core","convert","Into"],"trait")?;
    let crateid=&doc["paths"][id(&core_id)?]["crate_id"];
    let metadata=PathBuf::from(doc["external_crates"][id(crateid)?]["path"].as_str().context("Missing compiler core metadata")?).canonicalize()?;
    ensure!(metadata==expected_core_metadata.canonicalize()?,"Compiler core metadata outside the pinned sysroot identity");
    let mut observed=Vec::new();let mut keys=BTreeSet::new();
    for owner_name in ["Rand32","Rand64"] {
        let owners:Vec<_>=doc["index"].as_object().context("Missing compiler index")?.iter().filter(|(_,v)|v["crate_id"]==0&&v["name"]==owner_name&&v["visibility"]=="public"&&v["inner"].get("struct").is_some()).collect();
        ensure!(owners.len()==1,"Expected one original public owner");let (owner_id,owner)=owners[0];
        ensure!(doc["paths"][owner_id]["path"]==json!(["oorandom",owner_name]),"Owner public path mismatch");
        let oid=json!(owner_id.parse::<u64>()?);
        ensure!(root["inner"]["module"]["items"].as_array().context("Missing root exports")?.contains(&oid),"Owner is not an actual root declaration");
        for (tr,method,callback) in [("Into","into","From"),("TryInto","try_into","TryFrom"),("TryFrom","try_from","Into")] {
            let trait_id=path_id(doc,&["core","convert",tr],"trait")?;
            let candidates:Vec<_>=owner["inner"]["struct"]["impls"].as_array().context("Missing owner impls")?.iter().filter_map(|i|item(doc,i).ok().map(|v|(i,v))).filter(|(_,v)|v["inner"]["impl"]["trait"]["id"]==trait_id).collect();
            ensure!(candidates.len()==1,"Conditional family membership is missing or ambiguous");let (impl_id,entry)=candidates[0];let imp=&entry["inner"]["impl"];
            ensure!(imp["is_unsafe"]==false&&imp["is_negative"]==false&&imp["is_synthetic"]==false&&imp["provided_trait_methods"]==json!([]),"Unexpected implementation origin/header");
            ensure!(imp["for"]==json!({"resolved_path":{"path":owner_name,"id":oid,"args":null}})&&imp["blanket_impl"]==tyvar("T"),"Owner/source-T blanket substitution mismatch");
            ensure!(imp["trait"]==trait_ref(doc,tr,tyvar("U"))?,"Original trait argument mismatch");
            let predicate=json!({"bound_predicate":{"type":tyvar("U"),"bounds":[{"trait_bound":{"trait":trait_ref(doc,callback,tyvar("T"))?,"generic_params":[],"modifier":"none"}}],"generic_params":[]}});
            ensure!(imp["generics"]==json!({"params":source_params(),"where_predicates":[predicate]}),"Original complete T/U predicate set changed");
            let items=imp["items"].as_array().context("Missing original impl items")?;
            let methods:Vec<_>=items.iter().filter_map(|i|item(doc,i).ok().map(|v|(i,v))).filter(|(_,v)|v["name"]==method&&v["inner"].get("function").is_some()).collect();
            ensure!(methods.len()==1,"Original method is not unique within owner implementation");let (method_id,met)=methods[0];let f=&met["inner"]["function"];
            ensure!(f["has_body"]==true&&f["generics"]==json!({"params":[],"where_predicates":[]})&&f["header"]==json!({"is_const":false,"is_unsafe":false,"is_async":false,"abi":"Rust"}),"Original method body/header/generics changed");
            let error=if tr=="TryFrom" {json!({"primitive":"never"})}else{json!({"qualified_path":{"name":"Error","args":null,"self_type":tyvar("U"),"trait":trait_ref(doc,"TryFrom",tyvar("T"))?}})};
            let output=if tr=="Into" {tyvar("U")}else{json!({"resolved_path":{"path":"Result","id":path_id(doc,&["core","result","Result"],"enum")?,"args":{"angle_bracketed":{"args":[{"type":tyvar(if tr=="TryFrom"{"T"}else{"U"})},{"type":error}],"constraints":[]}}}})};
            ensure!(f["sig"]==json!({"inputs":[[if tr=="TryFrom"{"value"}else{"self"},tyvar(if tr=="TryFrom"{"U"}else{"Self"})]],"output":output,"is_c_variadic":false}),"Original signature or associated Error substitution changed");
            if tr!="Into" {
                let associated:Vec<_>=items.iter().filter_map(|i|item(doc,i).ok()).filter(|v|v["name"]=="Error"&&v["inner"].get("assoc_type").is_some()).collect();
                ensure!(associated.len()==1&&associated[0]["inner"]["assoc_type"]["type"]==error,"Associated Error is not the source projection/never type");
                ensure!(items.len()==2,"Unexpected extra impl item");
            }else{ensure!(items.len()==1,"Unexpected Into impl item");}
            let key=format!("<oorandom::{owner_name} as core::convert::{tr}<U>>::{method}");ensure!(keys.insert(key.clone()),"Duplicate owner-family observation");
            observed.push(json!({"api":key,"owner_id_ephemeral":oid,"impl_id_ephemeral":impl_id,"method_id_ephemeral":method_id,"source_T_substitution":owner_name,"source_U":"caller.U#0","source_bound":predicate,"associated_error":if tr=="Into"{Value::Null}else{error},"local_declaration":false,"conditional":true}));
        }
    }
    ensure!(observed.len()==6&&keys.len()==6,"Expected six memberships from three core templates and two owners");Ok(observed)
}

fn clean_command(program:&str)->Command {
    let mut c=Command::new(program);
    for name in ["RUSTC","RUSTDOC","RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER","RUSTFLAGS","RUSTDOCFLAGS","CARGO_ENCODED_RUSTFLAGS","CARGO_ENCODED_RUSTDOCFLAGS","CARGO_BUILD_TARGET","CARGO_TARGET_DIR","CARGO_BUILD_RUSTFLAGS","CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"]{c.env_remove(name);}
    c.env("CARGO_NET_OFFLINE","true").env("CARGO_BUILD_JOBS","1").env("RUSTUP_TOOLCHAIN","nightly-2026-09-17");c
}
fn run(c:&mut Command)->Result<Vec<u8>> {let o=c.output()?;
    #[cfg(test)] if let Ok(path)=std::env::var("PROOFS_OOR_CAPTURE_COMMAND_LOG") {
        use std::io::Write;
        let mut file=fs::OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file,"{}",json!({"program":c.get_program().to_string_lossy(),"args":c.get_args().map(|s|s.to_string_lossy().into_owned()).collect::<Vec<_>>(),"cwd":c.get_current_dir().map(|p|p.to_string_lossy()),"exit":o.status.code(),"stdout_sha256":digest(&o.stdout),"stderr_sha256":digest(&o.stderr)}))?;
    }
    ensure!(o.status.success(),"Compiler-owned source capture failed: {}",String::from_utf8_lossy(&o.stderr));Ok(o.stdout)}

fn absent_config(path:&Path)->Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(()),
        Err(e)=>Err(e.into()),
        Ok(_)=>anyhow::bail!("Cargo configuration rejected: {}",path.display()),
    }
}
fn reject_cargo_configs(dir:&Path)->Result<()> {
    for ancestor in dir.canonicalize()?.ancestors() {
        for name in [".cargo/config", ".cargo/config.toml"] {
            absent_config(&ancestor.join(name))?;
        }
    }
    let home=match std::env::var_os("CARGO_HOME") {
        Some(p)=>PathBuf::from(p),
        None=>PathBuf::from(std::env::var_os("HOME").context("Missing effective Cargo home")?).join(".cargo"),
    };
    for name in ["config", "config.toml"] {
        absent_config(&home.join(name))?;
    }
    Ok(())
}
fn pinned_tool(name:&str, version_arg:&str, fixed:&Value)->Result<PathBuf> {
    let paths=std::env::var_os("PATH").context("Missing executable PATH")?;
    let candidate=std::env::split_paths(&paths).map(|p|p.join(name)).find(|p|p.is_file()).context("Missing pinned executable")?;
    let executable=candidate.canonicalize()?;
    let pin=&fixed["official_tools"][name];
    ensure!(digest(fs::read(&executable)?)==pin["sha256"].as_str().context("Missing official executable hash pin")?, "Official executable identity changed: {name}");
    let version=String::from_utf8(run(clean_command(executable.to_str().context("NonUTF8 executable")?).arg(version_arg))?)?;
    ensure!(version.trim()==pin["version"].as_str().context("Missing official executable version pin")?, "Official executable version changed: {name}");
    Ok(executable)
}

/// Creates raw observations itself: a supplied observation JSON is never used as authority.
pub(super) fn capture_rustdoc(original:&Path,out:&Path,fixed:&Value)->Result<(Value,Vec<Value>)> {
    fs::create_dir_all(out)?;
    reject_cargo_configs(original)?;
    let compiler=String::from_utf8(run(clean_command("rustc").arg("-vV"))?)?;
    ensure!(compiler.lines().any(|x|x=="commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8")&&compiler.lines().any(|x|x=="host: x86_64-unknown-linux-gnu"),"Unsupported compiler/host");
    let sysroot=PathBuf::from(String::from_utf8(run(clean_command("rustc").args(["--print","sysroot"]))?)?.trim()).canonicalize()?;
    let source=sysroot.join("lib/rustlib/src/rust/library/core/src/convert/mod.rs");
    ensure!(digest(fs::read(&source)?)==fixed["core_source_sha256"].as_str().context("Missing pinned core source")?,"Original core source changed");
    let metadata=sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/lib");
    let mut core_files=Vec::new();for e in fs::read_dir(&metadata)? {let p=e?.path();let name=p.file_name().unwrap().to_string_lossy();if name.starts_with("libcore-")&&name.ends_with(".rmeta")&&digest(fs::read(&p)?)==fixed["core_metadata_sha256"].as_str().context("Missing pinned core metadata")?{core_files.push(p);}}
    ensure!(core_files.len()==1,"Pinned original core compiler metadata is not unique");
    run(clean_command("cargo").current_dir(original).args(["rustdoc","--offline","--locked","--lib","--target","x86_64-unknown-linux-gnu","--target-dir"]).arg(out).args(["--","-Zunstable-options","--output-format","json"]))?;
    let jsonfile=out.join("x86_64-unknown-linux-gnu/doc/oorandom.json");let bytes=fs::read(jsonfile)?;let doc:Value=serde_json::from_slice(&bytes)?;let memberships=validate_document(&doc,&core_files[0])?;
    fs::write(out.join("raw-rustdoc.sha256"),digest(&bytes))?;Ok((doc,memberships))
}

fn checked_member(root:&Path,name:&str)->Result<PathBuf> {
    use std::path::Component;
    let rel=Path::new(name);ensure!(!rel.as_os_str().is_empty()&&rel.components().all(|c|matches!(c,Component::Normal(_))),"Nonrelative source closure member");
    let root=root.canonicalize()?;let mut p=root.clone();
    for part in rel.components(){p.push(part);ensure!(!fs::symlink_metadata(&p)?.file_type().is_symlink(),"Source closure symlink rejected");}
    let p=p.canonicalize()?;ensure!(p.starts_with(root),"Source closure escapes captured package");Ok(p)
}
pub(super) fn check_files(root:&Path,files:&Value)->Result<()> {
    for (name,hash) in files.as_object().context("Missing compiled finite file map")? {
        ensure!(digest(fs::read(checked_member(root,name)?)?)==hash.as_str().context("Invalid compiled digest")?,"Original/translation/proof closure changed: {name}");
    }Ok(())
}
fn rust_members(root:&Path,dir:&Path,out:&mut BTreeSet<String>)->Result<()> {
    for e in fs::read_dir(dir)? {let p=e?.path();let md=fs::symlink_metadata(&p)?;ensure!(!md.file_type().is_symlink(),"Original source tree symlink rejected");
        if md.is_dir(){rust_members(root,&p,out)?;}else if p.extension().and_then(|s|s.to_str())==Some("rs"){out.insert(p.strip_prefix(root)?.to_string_lossy().replace('\\',"/"));}}
    Ok(())
}
/// Copy only the archive-compared source closure to a fresh compiler-owned stage.
/// Capture never runs a script supplied by the artifact or uses its original target.
pub(super) fn stage_original(project:&crate::config::Project,work:&Path,fixed:&Value)->Result<PathBuf> {
    let package=project.manifest.parent().context("Missing original package")?.canonicalize()?;
    ensure!(project.root_source.canonicalize()?==checked_member(&package,"src/lib.rs")?,"Original library root mismatch");
    reject_cargo_configs(&package)?;
    reject_cargo_configs(work)?;
    ensure!(!package.join("build.rs").exists(),"Automatic build script rejected");
    check_files(&package,&fixed["source_files"])?;
    ensure!(digest(fs::read(checked_member(&package,"evidence/upstream.crate")?)?)==fixed["upstream_archive_sha256"].as_str().context("Missing original archive pin")?,"Original crate archive changed");
    let mut actual=BTreeSet::new();rust_members(&package,&package.join("src"),&mut actual)?;
    let expected:BTreeSet<String>=fixed["source_files"].as_object().unwrap().keys().filter(|p|p.starts_with("src/")&&p.ends_with(".rs")).cloned().collect();ensure!(actual==expected,"Original Rust source file set differs");
    let original=work.join("original");fs::create_dir_all(&original)?;
    for name in fixed["source_files"].as_object().unwrap().keys(){let dest=original.join(name);fs::create_dir_all(dest.parent().context("Missing stage parent")?)?;fs::copy(checked_member(&package,name)?,dest)?;}
    reject_cargo_configs(&original)?;
    check_files(&original,&fixed["source_files"])?;Ok(original)
}

/// The approved nine-line source has six owner witnesses and three generic roots.
/// The two extractions below are the approved generic3 and generic2 captures only.
const DRIVER:&str = "use oorandom::{Rand32, Rand64};\npub fn into32<U: From<Rand32>>(x: Rand32) -> U { x.into() }\npub fn into64<U: From<Rand64>>(x: Rand64) -> U { x.into() }\npub fn try_into32<U: TryFrom<Rand32>>(x: Rand32) -> Result<U, U::Error> { x.try_into() }\npub fn try_into64<U: TryFrom<Rand64>>(x: Rand64) -> Result<U, U::Error> { x.try_into() }\npub fn try_from32<U: Into<Rand32>>(x: U) -> Result<Rand32, core::convert::Infallible> { Rand32::try_from(x) }\npub fn try_from64<U: Into<Rand64>>(x: U) -> Result<Rand64, core::convert::Infallible> { Rand64::try_from(x) }\npub fn core_into<T, U: From<T>>(x:T) -> U { x.into() }\npub fn core_try_into<T, U: TryFrom<T>>(x:T) -> Result<U,U::Error> { x.try_into() }\npub fn core_try_from<T, U: Into<T>>(x:U) -> Result<T,core::convert::Infallible> { T::try_from(x) }\n";

fn llbc_name(value:&Value)->Option<String> {let parts=value.as_array()?;let names:Vec<_>=parts.iter().filter_map(|p|p["Ident"].as_array().and_then(|i|i[0].as_str())).collect();Some(names.join("::"))}
/// Structural origin/body guard, matching the already audited source-body guard.
/// This is not a new LLVM or operational-semantics verification system.
fn validate_core_bodies(raw:&Value,roles:&[&str])->Result<()> {
    ensure!(raw["has_errors"]==false,"Compiler LLBC has errors");let c=&raw["translated"];
    for (method,line) in [("into",779u64),("try_into",805),("try_from",821)] {
        if !roles.contains(&method){continue}
        let fs:Vec<_>=c["fun_decls"].as_array().context("Missing LLBC functions")?.iter().filter(|f|llbc_name(&f["item_meta"]["name"]).as_deref()==Some(&format!("core::convert::{method}"))).collect();
        ensure!(fs.len()==1,"Original core body missing or ambiguous");let f=fs[0];
        ensure!(f["item_meta"]["is_local"]==false&&f["item_meta"]["opacity"]=="Transparent"&&f["item_meta"]["attr_info"]["rename"].is_null(),"Core origin/transparent body changed");
        let span=&f["item_meta"]["span"]["Untagged"]["data"];ensure!(span["beg"]["line"]==line,"Original core method source line changed");let file=span["file_id"].as_u64().context("Missing core source file")? as usize;
        ensure!(c["files"][file]["name"]==json!({"Local":"/rustc/library/core/src/convert/mod.rs"})&&c["files"][file]["crate_name"]=="core","Actual body is not the pinned original core module");
        let generics=&f["generics"];let params=generics["types"].as_array().context("Missing source binders")?;let names:Vec<_>=params.iter().map(|p|p["name"].as_str()).collect();
        ensure!(names==if method=="try_into"{vec![Some("T"),Some("U"),Some("Clause0_Error")]}else{vec![Some("T"),Some("U")]},"Original source/associated-type binders differ");
        ensure!(generics["trait_clauses"].as_array().context("Missing original dictionary bound")?.len()==1,"Core source requires exactly one dictionary predicate");
        let statements=f["body"]["Structured"]["body"]["statements"].as_array().context("Original body was not extracted transparently")?;
        let shape:Vec<String>=statements.iter().map(|s|s["kind"].as_str().map(str::to_owned).or_else(||s["kind"].as_object().and_then(|o|o.keys().next()).cloned()).unwrap_or_default()).collect();
        let expected=if method=="try_from"{vec!["StorageLive","StorageLive","Call","Assign","StorageDead","StorageDead","Return"]}else{vec!["StorageLive","Call","StorageDead","Return"]};ensure!(shape==expected,"Original core body is not the approved one-call delegation/Ok wrapping");
        let calls:Vec<_>=statements.iter().filter_map(|s|s["kind"].get("Call")).collect();ensure!(calls.len()==1,"Original body call count differs");let call=&calls[0]["call"];
        ensure!(call["func"]["Regular"]["kind"]["Trait"].is_array()&&call["args"].as_array().map(Vec::len)==Some(1)&&call["args"][0]["Move"]["kind"]==json!({"Local":1})&&call["dest"]["kind"]==json!({"Local":if method=="try_from"{2}else{0}}),"Original callback argument/return changed");
    }Ok(())
}

/// Re-extract original core twice and compare the exact approved generated proof
/// bodies/metadata; the artifact's regenerate.py or other scripts are never run.
pub(super) fn capture_generation(original:&Path,work:&Path,package:&Path,fixed:&Value)->Result<()> {
    check_files(package,&fixed["proof_files"])?;check_files(package,&fixed["translation_files"])?;
    let driver=work.join("driver");fs::create_dir_all(driver.join("src"))?;
    reject_cargo_configs(&driver)?;
    let charon_tool=pinned_tool("charon","version",fixed)?;
    let aeneas_tool=pinned_tool("aeneas","-version",fixed)?;
    let dependency=serde_json::to_string(original.to_str().context("NonUTF8 original stage")?)?;
    fs::write(driver.join("Cargo.toml"),format!("[package]\nname=\"oorandom-generic-convert-roots\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\noorandom={{path={dependency}}}\n"))?;
    fs::write(driver.join("src/lib.rs"),DRIVER)?;
    fs::copy(checked_member(package,fixed["driver_lock_path"].as_str().context("Missing fixed driver lock")?)?,driver.join("Cargo.lock"))?;
    let archive=checked_member(package,fixed["custom_archive_path"].as_str().context("Missing custom tool archive")?)?;
    let binary=run(Command::new("gzip").args(["-dc","--"]).arg(archive))?;ensure!(digest(&binary)==fixed["custom_binary_sha256"].as_str().context("Missing binary pin")?,"Unknown custom translator binary");
    let tool=work.join("original-core-export-aeneas");fs::write(&tool,binary)?;
    #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(&tool,fs::Permissions::from_mode(0o700))?;}
    for (filename,roles,custom,expected) in [("CoreConversions.llbc",vec!["core_into","core_try_into","core_try_from"],false,"official"),("CoreDelegations.llbc",vec!["core_into","core_try_into"],true,"custom")] {
        let mut charon=clean_command(charon_tool.to_str().context("NonUTF8 Charon path")?);charon.current_dir(&driver).args(["cargo","--preset=aeneas","--sysroot","default","--dest-file",filename]);
        for root in roles{charon.args(["--start-from",&format!("crate::{root}")]);}charon.args(["--include","core::convert"]);run(&mut charon)?;
        let llbc:Value=serde_json::from_slice(&fs::read(driver.join(filename))?)?;validate_core_bodies(&llbc,if custom{&["into","try_into"]}else{&["into","try_into","try_from"]})?;
        let dest=work.join(expected);let mut aeneas=if custom{Command::new(&tool)}else{clean_command(aeneas_tool.to_str().context("NonUTF8 Aeneas path")?)};aeneas.current_dir(&driver).args(["-backend","lean","-emit-json","-abort-on-error","-dest"]).arg(&dest);
        if custom{aeneas.args(["-namespace","oorandom_core_delegations","-emit-original-core-conversion-bodies"]);}aeneas.arg(filename);run(&mut aeneas)?;
        for (file,hash) in fixed["generated_outputs"][expected].as_object().context("Missing fixed generated outputs")? {ensure!(digest(fs::read(dest.join(file))?)==hash.as_str().context("Invalid generated hash")?,"Original core generated body/metadata differs: {expected}/{file}");}
    }
    Ok(())
}

#[cfg(test)]
mod focused_tests {
    use super::*;
    #[test]
    #[ignore = "explicit compiler-owned capture; requires fixture env and pinned toolchain"]
    fn capture_evidence_and_five_semantic_negatives()->Result<()> {
        let fixture=PathBuf::from(std::env::var("PROOFS_OOR_GENERIC_FIXTURE")?);
        let project=crate::config::Project::load(&crate::ProjectArgs{manifest_path:Some(fixture.join("Cargo.toml")),..Default::default()})?;
        let fixed:Value=serde_json::from_str(super::super::DATA)?;
        let work=tempfile::tempdir()?.keep();
        println!("CAPTURE_EVIDENCE={}",work.display());
        std::env::set_var("PROOFS_OOR_CAPTURE_COMMAND_LOG",work.join("commands.jsonl"));
        fs::write(work.join("scope.json"),super::super::DATA)?;
        let original=stage_original(&project,&work,&fixed)?;
        let (doc,observations)=capture_rustdoc(&original,&work.join("rustdoc"),&fixed)?;
        fs::write(work.join("observations.json"),serde_json::to_vec_pretty(&observations)?)?;
        let core_trait=path_id(&doc,&["core","convert","Into"],"trait")?;
        let core_crate=id(&doc["paths"][id(&core_trait)?]["crate_id"])?;
        let core=PathBuf::from(doc["external_crates"][&core_crate]["path"].as_str().unwrap());
        let owner=doc["index"].as_object().unwrap().iter().find(|(_,v)|v["name"]=="Rand32"&&v["inner"].get("struct").is_some()).unwrap().0.clone();
        let imp=observations[0]["impl_id_ephemeral"].as_str().map(str::to_owned).unwrap_or_else(||observations[0]["impl_id_ephemeral"].to_string());
        let method=id(&observations[0]["method_id_ephemeral"])?;
        let try_imp=id(&observations[1]["impl_id_ephemeral"])?;
        let mut results=Vec::new();
        for kind in ["bounds","associated-error","trait-core-identity","owner-membership","method-signature"] {
            let mut altered=doc.clone();
            match kind {
                "bounds"=>altered["index"][&imp]["inner"]["impl"]["generics"]["where_predicates"]=json!([]),
                "associated-error"=>{let items=altered["index"][&try_imp]["inner"]["impl"]["items"].as_array().unwrap().clone();let a=items.iter().find(|i|item(&doc,i).unwrap()["name"]=="Error").unwrap();altered["index"][id(a)?]["inner"]["assoc_type"]["type"]=json!({"primitive":"u8"});},
                "trait-core-identity"=>altered["external_crates"][&core_crate]["name"]=json!("other_core"),
                "owner-membership"=>altered["index"][&owner]["inner"]["struct"]["impls"]=json!([]),
                _=>altered["index"][&method]["inner"]["function"]["header"]["is_unsafe"]=json!(true),
            }
            let error=validate_document(&altered,&core).expect_err("semantic mutation accepted");
            results.push(json!({"case":kind,"error":error.to_string(),"validator":"validate_document directly; no outer file hash"}));
        }
        ensure!(validate_document(&doc,&core)?.len()==6,"Positive document changed");
        fs::write(work.join("semantic-negatives.json"),serde_json::to_vec_pretty(&results)?)?;
        capture_generation(&original,&work,&fixture,&fixed)?;
        fn hashes(root:&Path,dir:&Path,out:&mut serde_json::Map<String,Value>)->Result<()> {for e in fs::read_dir(dir)? {let p=e?.path();if p.is_dir(){hashes(root,&p,out)?;}else{out.insert(p.strip_prefix(root)?.to_string_lossy().into_owned(),json!(digest(fs::read(p)?)));}}Ok(())}
        let mut map=serde_json::Map::new();hashes(&work,&work,&mut map)?;
        fs::write(work.join("FILE-HASHES.json"),serde_json::to_vec_pretty(&map)?)?;
        std::env::remove_var("PROOFS_OOR_CAPTURE_COMMAND_LOG");
        Ok(())
    }
}
