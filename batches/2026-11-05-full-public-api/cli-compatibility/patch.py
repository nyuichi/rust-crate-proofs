from pathlib import Path
p=Path('/workspace/cargo-proofs-local/src/scan.rs');s=p.read_text()
pos=s.index('struct Scanner<')
s=s[:pos]+'''#[derive(Clone)]
struct LocalTraitDefault {
    trait_path: String,
    method: syn::TraitItemFn,
    file: PathBuf,
}
#[derive(Clone)]
struct LocalTraitImpl {
    module: Vec<String>,
    context: CallableContext,
    overrides: BTreeSet<String>,
}
'''+s[pos:]
s=s.replace('    traits: BTreeSet<String>,','    traits: BTreeSet<String>,\n    trait_defaults: Vec<LocalTraitDefault>,\n    trait_impls: Vec<LocalTraitImpl>,',1)
s=s.replace('        traits: BTreeSet::new(),','        traits: BTreeSet::new(),\n        trait_defaults: vec![],\n        trait_impls: vec![],',1)
# All imports, traits and implementations must be scanned before resolving defaults.
s=s.replace('    s.file(&project.root_source, &[], true)?;','    s.file(&project.root_source, &[], true)?;\n    s.add_local_trait_defaults()?;')
needle='impl Scanner<\'_> {\n'
method='''    /// Discover actual inherited bodies from local traits, preserving their
    /// defining file/span while using the concrete implementation's identity.
    /// This does not synthesize bodies, infer foreign defaults, or replace an
    /// explicit implementation method. Generic trait arguments remain exactly
    /// the arguments written on the implementation; no body is rewritten.
    fn add_local_trait_defaults(&mut self) -> Result<()> {
        let public_visibility = syn::Visibility::Public(Default::default());
        for implementation in self.trait_impls.clone() {
            let CallableContext::TraitImpl { trait_path, .. } = &implementation.context else {
                continue;
            };
            let mut bare_trait = trait_path.clone();
            for segment in &mut bare_trait.segments {
                segment.arguments = syn::PathArguments::None;
            }
            let resolved = self.resolve(&segments(&bare_trait)?, &implementation.module, 0)?;
            let defaults: Vec<_> = self.trait_defaults.iter()
                .filter(|default| default.trait_path == resolved)
                .cloned().collect();
            for default in defaults {
                if implementation.overrides.contains(&default.method.sig.ident.to_string()) {
                    continue;
                }
                self.function(
                    &default.method.attrs, &default.method.sig, &public_visibility,
                    default.method.span(), &default.file, &implementation.module,
                    implementation.context.clone(),
                )?;
            }
        }
        Ok(())
    }
'''
assert s.count(needle)==1;s=s.replace(needle,needle+method)
old='''                        self.traits.insert(path.clone());
                        if matches!(i.vis, syn::Visibility::Public(_)) {
                            self.public_items.insert(path);
                        }
'''
new='''                        self.traits.insert(path.clone());
                        for item in &i.items {
                            if let syn::TraitItem::Fn(method) = item {
                                if method.default.is_some() {
                                    self.trait_defaults.push(LocalTraitDefault {
                                        trait_path: path.clone(), method: method.clone(), file: file.into(),
                                    });
                                }
                            }
                        }
                        if matches!(i.vis, syn::Visibility::Public(_)) {
                            self.public_items.insert(path);
                        }
'''
assert s.count(old)==1;s=s.replace(old,new)
needle='''                    for method in &i.items {
                        if let syn::ImplItem::Fn(f) = method {
'''
insert='''                    if matches!(context, CallableContext::TraitImpl { .. }) {
                        let mut overrides = BTreeSet::new();
                        for item in &i.items {
                            if let syn::ImplItem::Fn(method) = item {
                                if attrs(&method.attrs, self.project)?.is_some() {
                                    overrides.insert(method.sig.ident.to_string());
                                }
                            }
                        }
                        self.trait_impls.push(LocalTraitImpl {
                            module: module.to_vec(), context: context.clone(), overrides,
                        });
                    }
'''
assert s.count(needle)==1;s=s.replace(needle,insert+needle)
pos=s.index('    #[test]\n    fn foreign_self_trait_with_public_owned_argument()')
tests='''    #[test]
    fn local_trait_default_bodies_follow_real_impls_and_overrides() {
        let source = "pub trait Order {\\nfn write(&self, x: u16) -> u16 { x }\\nfn required(&self);\\n}\\npub struct A; pub struct B;\\nimpl Order for A { fn required(&self) {} }\\nimpl Order for B { fn required(&self) {} fn write(&self, x: u16) -> u16 { x + 1 } }\\n";
        let c = creusot_contracts(source, CreusotTarget::All);
        assert_eq!(c.len(), 4);
        let a = c.iter().find(|c| c.api_paths.contains(&"<demo::A as demo::Order>::write".into())).unwrap();
        let b = c.iter().find(|c| c.api_paths.contains(&"<demo::B as demo::Order>::write".into())).unwrap();
        assert_eq!(a.first_line, 2, "inherited body points to trait declaration");
        assert_eq!(b.first_line, 7, "override retains its own defining body");
        assert!(!c.iter().any(|c| c.api_paths.iter().any(|p| p.starts_with("Order::"))));
    }

    #[test]
    fn local_generic_trait_defaults_preserve_each_impl_argument() {
        let c = creusot_contracts(r#"
pub trait Convert<T> { fn value(&self, x: T) -> T { x } }
pub struct S;
impl Convert<u8> for S {}
impl Convert<u16> for S {}
"#, CreusotTarget::All);
        assert_eq!(c.len(), 2);
        assert!(c.iter().any(|c| c.api_paths.contains(&"<demo::S as demo::Convert<u8>>::value".into())));
        assert!(c.iter().any(|c| c.api_paths.contains(&"<demo::S as demo::Convert<u16>>::value".into())));
    }

    #[test]
    fn local_trait_defaults_obey_reexports_visibility_and_cfg() {
        let c = creusot_contracts(r#"
mod hidden {
    pub trait T { fn yes(&self) {} #[cfg(feature="absent")] fn disabled(&self) {} }
    trait Private { fn no(&self) {} }
    pub struct S;
    impl T for S {}
    impl Private for S {}
}
pub use hidden::{T as Trait, S as Public};
"#, CreusotTarget::All);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].api_paths, ["<demo::Public as demo::Trait>::yes"]);
    }

    #[test]
    fn local_trait_defaults_do_not_invent_required_or_foreign_methods() {
        let c = creusot_contracts(r#"
pub trait Required { fn needed(&self); }
pub struct S;
impl Required for S { fn needed(&self) {} }
impl Clone for S { fn clone(&self) -> Self { S } }
"#, CreusotTarget::All);
        assert_eq!(c.len(), 2);
        assert!(!c.iter().any(|c| c.api_paths.iter().any(|p| p.ends_with("clone_from"))));
        assert!(c.iter().any(|c| c.api_paths.contains(&"<demo::S as demo::Required>::needed".into())));
    }

'''
s=s[:pos]+tests+s[pos:];p.write_text(s)
