#!/usr/bin/env python3
"""Fail-closed native source/MIR correspondence audit for AQ slice views.

No Cargo, rustc, Why3 or solver is invoked here. This checks the captured
native source, selected ElaborateDrops MIR, source extraction, and input routes.
It does not prove MIR adequacy, Rust provenance, or memory-model semantics.
"""
from __future__ import annotations
import argparse, copy, hashlib, importlib.util, json, pathlib, re, sys, tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parent
CRATE_ROOT = ROOT.parents[2]
PROBES = ROOT.parent
AP_PROBE = PROBES / "original-shared-finite-owners-2026-10-09"
AP_CHECKER_PATH = AP_PROBE / "check_native.py"
AP_CHECKER_SHA256 = "5c6b0d399114bf3f0d7fd7149573f506faba14043b60c5acb5018f75a16524cc"
AI_PROBE = PROBES / "original-shared-scoped-client-2026-10-09"
AI_CHECKER_PATH = AI_PROBE / "check_correspondence.py"
AI_CHECKER_SHA256 = "decb502490fdd1c534e02b5ac0798b070eeb8ed7be4d6aa1779e05cee5e08bb7"
AI_HELPER_FIXTURE_PATH = AI_PROBE / "fixtures/expected-shadow-bodies.json"
AI_HELPER_FIXTURE_SHA256 = "c4903a139f0924fc26a62c8319617db1f34cfbb876e7e775abba0f70130d26e1"
MIR_DIR = ROOT / "native-mir"
CAPTURE_PATH = MIR_DIR / "capture.json"
EXPECTED_CAPTURE_SHA256 = "877f54fcf8638dfde66ae21ef65fdb1d7f4be81429eb3fa2b0a70aab50a72569"
EXPECTED_RUSTC = (
    "rustc 1.98.0-nightly (91fe22da8 2026-06-21)\n"
    "binary: rustc\ncommit-hash: 91fe22da8084a1c9e993d78d4a56f22ab8396236\n"
    "commit-date: 2026-06-21\nhost: x86_64-unknown-linux-gnu\n"
    "release: 1.98.0-nightly\nLLVM version: 22.1.7\n")
EXPECTED_CARGO = "cargo 1.98.0-nightly (a595d0da2 2026-06-20)\n"
EXPECTED_MANIFEST = {
    "package":{"name":"bytes-shared-slice-views-native","version":"0.0.0","edition":"2021"},
    "workspace":{},
    "lib":{"name":"bytes_shared_slice_views_native","path":"../native.rs"},
    "dependencies":{"bytes":{"path":"../../../../"}},
}
EXPECTED_NATIVE_SOURCE = r"""use bytes::Bytes;

pub fn nested_slice_scope(input: Box<[u8]>, a: usize, b: usize, c: usize, d: usize) -> Vec<u8> {
    let selected = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        let first = owner.slice(a..b);
        first.slice(c..d)
    };
    let observed = AsRef::<[u8]>::as_ref(&selected).to_vec();
    observed
}
"""
EXPECTED_NATIVE_TEST = r"""use bytes_shared_slice_views_native::nested_slice_scope;

#[test]
fn nested_ranges_preserve_exact_contents() {
    for len in [1usize, 2, 7, 31] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        for (a,b) in [(0,len), (len/2,len), (0,0), (len,len), (len/2,len/2)] {
            let n=b-a;
            for (c,d) in [(0,n),(n/2,n),(0,0),(n,n),(n/2,n/2)] {
                assert_eq!(nested_slice_scope(expected.clone().into_boxed_slice(),a,b,c,d),expected[a+c..a+d]);
            }
        }
    }
}"""
EXPECTED_SLICE = r"""pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        use core::ops::Bound;

        let len = self.len();

        let begin = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n.checked_add(1).expect("out of range"),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        assert!(
            begin <= end,
            "range start must not be greater than end: {:?} <= {:?}",
            begin,
            end,
        );
        assert!(
            end <= len,
            "range end out of bounds: {:?} <= {:?}",
            end,
            len,
        );

        if end == begin {
            return Bytes::new_empty_with_ptr(self.ptr.wrapping_add(begin));
        }

        let mut ret = self.clone();

        ret.len = end - begin;
        ret.ptr = unsafe { ret.ptr.add(begin) };

        ret
    }"""
EXPECTED_NEW_EMPTY = r"""fn new_empty_with_ptr(ptr: *const u8) -> Self {
        debug_assert!(!ptr.is_null());

        // Detach this pointer's provenance from whichever allocation it came from, and reattach it
        // to the provenance of the fake ZST [u8;0] at the same address.
        let ptr = without_provenance(ptr as usize);

        Bytes {

            #[cfg(all(creusot, bytes_original_freeze_gate))]

            original_frozen: None,
            ptr,
            len: 0,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }"""
EXPECTED_STATIC_CLONE = r"""unsafe fn static_clone(_: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let slice = slice::from_raw_parts(ptr, len);
    Bytes::from_static(slice)
}"""
EXPECTED_STATIC_DROP = r"""unsafe fn static_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) {
    // nothing to drop for &'static [u8]
}"""
EXPECTED_WITHOUT_PROVENANCE = r"""fn without_provenance(ptr: usize) -> *const u8 {
    core::ptr::null::<u8>().wrapping_add(ptr)
}"""
EXPECTED_STATIC_VTABLE = r"""const STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    into_vec: static_to_vec,
    into_mut: static_to_mut,
    is_unique: static_is_unique,
    drop: static_drop,
}"""
EXPECTED_AS_REF_IMPL = r"""impl AsRef<[u8]> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[inline]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}"""
EXPECTED_SOURCE_SHA = {
    "native.rs":"308599fb0344a907d52ca9dac8a9b28666cbe95241328d7b7482147d8eb050fd",
    "native_test":"37c8a87b55783027f91ae4503c8b9fc88948adb6a11fc58c735a51765b9cc6ba",
    "capture_script":"15bf9322fee65a2418055452568475466026e2b93e354743fd8ef60b16fe81f1",
    "extractor":"6ba81b23f0b8edfc37d442f6617925d29d7a87d74a0aaf6e5bb999b5320b2414",
    "native_view_bindings":"dfc6305f4b8ceebf4a0afa74429ae24f968d7b5da541ce4aff88cb92cefc2e00",
    "source_map":"db9999dc8a2ddec2411df6bf817eafc89ec7383e617035963da52aa395166d22",
    "reviewed_manifest":"e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306",
    "native_manifest":"3007e4f4013a61fb489eb3cee721d57e7cdaff9ae80445962411635c2ac00198",
    "native_lock":"aaae41852444f4f1dd9b4132dda7500afa6fffc854fb0d7b91d551901daf579f",
    "native_log":"0371750aad8e2798acc8fa1c67185b68b21844627ca1274c0b5c1d1472dd0023",
}
MIR_EXPECTED = {'client': ('bytes_shared_slice_views_native.nested_slice_scope.2-2-004.ElaborateDrops.after.mir', '14e4b98674e4fa32d7bf6daadd6754549d010e4f80b63c43f1ce5984ca06b161'), 'from_box': ('bytes.bytes-{impl#43}-from.2-2-004.ElaborateDrops.after.mir', '0c41ac37dc2f03f706c2c8791576540fc9627851d4b14fea8a757d383865250c'), 'clone_impl': ('bytes.bytes-{impl#4}-clone.2-2-004.ElaborateDrops.after.mir', 'fb9d64b8c5b602e08210dc337f843477344498f7eb505e9e5baf41274661c4e6'), 'cleanup': ('bytes.bytes-{impl#0}-cleanup.2-2-004.ElaborateDrops.after.mir', 'd0ac909028307408b85eecf4c9bcf93c3bb37fa471a5074dd6ae3dd001f9c31e'), 'bytes_drop': ('bytes.bytes-{impl#3}-drop.2-2-004.ElaborateDrops.after.mir', '50f3f09f68d2c58edb8200370d82b140e78f06827112f4f86fbf1eb10a7c3594'), 'as_ref': ('bytes.bytes-{impl#7}-as_ref.2-2-004.ElaborateDrops.after.mir', '37f244a44caaef9c601db6c8a0361d670eb5b4a54bef61e4f189178e233a06dc'), 'as_slice': ('bytes.bytes-{impl#0}-as_slice.2-2-004.ElaborateDrops.after.mir', 'e087d5506ef2e39da6545bd7f463754d25e631068a84bde906f1d09925a92b0d'), 'promotable_even_clone': ('bytes.bytes-promotable_even_clone.2-2-004.ElaborateDrops.after.mir', '8c8096ff055712a1952ea9af792df5887ef59078ea86bae0694f9be7e2ff6227'), 'promotable_odd_clone': ('bytes.bytes-promotable_odd_clone.2-2-004.ElaborateDrops.after.mir', '19ef9766bcb4e947b6110dbd599ad8d7f20c35f692103d434b8c33a0903bc203'), 'shallow_clone_vec': ('bytes.bytes-shallow_clone_vec.2-2-004.ElaborateDrops.after.mir', '1162f36670fee9e69839849a7d5aa1ff38e3d62fa4c9d1070bf01c84dc8cfc58'), 'shallow_clone_arc': ('bytes.bytes-shallow_clone_arc.2-2-004.ElaborateDrops.after.mir', 'b7c65e93f45f64c5c5796b162c8626cc3e8fa684d668aa1ffecac187af21c074'), 'shared_clone': ('bytes.bytes-shared_clone.2-2-004.ElaborateDrops.after.mir', 'bba53488a14c0bb04e1cf3e39f00ed7db3a6d5a3cd7b80cf5b3c4347d9be261c'), 'slice': ('bytes.bytes-{impl#0}-slice.2-2-004.ElaborateDrops.after.mir', '0278bb73404aa84da0ecad4aa6c6e39f6a4a5d739f0e7e22169638b2370b245f'), 'new_empty_with_ptr': ('bytes.bytes-{impl#0}-new_empty_with_ptr.2-2-004.ElaborateDrops.after.mir', 'd9e5b431dc1f20a7db7b7a4991328632f26e0832acdf45bd0b2faff087429c99'), 'static_clone': ('bytes.bytes-static_clone.2-2-004.ElaborateDrops.after.mir', '6eb2fedbc9a5d6e7f073068002e0fdfb1aab354aa816fb54c85a1c6951dfa6b3'), 'static_drop': ('bytes.bytes-static_drop.2-2-004.ElaborateDrops.after.mir', '25573f03210e9377e6fb1131013c75342b13388ef27a8958eb5f7a654bda68b8'), 'without_provenance': ('bytes.bytes-without_provenance.2-2-004.ElaborateDrops.after.mir', '054eaa11b62398130479f25f315f2b8b2d5e26da5b665dbd023d2b1626c08b9d'), 'ref_count_increment': ('bytes.ref_count_ops-increment.2-2-004.ElaborateDrops.after.mir', 'da43b1e17f5cd62d494c6993ea67f1766731576a53bb667858e5883398bc1487'), 'promotable_even_drop': ('bytes.bytes-promotable_even_drop.2-2-004.ElaborateDrops.after.mir', 'df5b8232c6e577d0c62bf890e2036ad7b46a0b32d5c9aacb418fd71f94e91a50'), 'promotable_odd_drop': ('bytes.bytes-promotable_odd_drop.2-2-004.ElaborateDrops.after.mir', 'c86fbbd98e95d7a29a1544d1efb9a6512596a022515c523320916711dbcd39bb'), 'shared_drop': ('bytes.bytes-shared_drop.2-2-004.ElaborateDrops.after.mir', 'fe67a58c872b33e16ea77a15a3f1aacedf50d9f81a33a7ff4b28ca378827f398'), 'release_shared': ('bytes.bytes-release_shared.2-2-004.ElaborateDrops.after.mir', '7c512f07e67b255306da9f320097cf70a9595ea769606f69b8ae8b8d8ae2aaab'), 'free_shared': ('bytes.bytes-free_shared.2-2-004.ElaborateDrops.after.mir', 'e53703381f9a08dccabfaa2f2a3a7a8dd5b144fff3fe04094e9371873ffe9226'), 'ptr_map': ('bytes.bytes-ptr_map.2-2-004.ElaborateDrops.after.mir', '3415ee18db30bc60b4ffe17aabdfd04d20a4fe691e7b73d85f171c58655c07f4'), 'atomic_with_mut': ('bytes.loom-sync-atomic-{impl#0}-with_mut.2-2-004.ElaborateDrops.after.mir', 'bb7d61c4e2129237fed09d974f0e791a6f1543d04204e122f503a1c2ab3bada8')}
EXPECTED_MIR_LABELS = set(MIR_EXPECTED)
EXPECTED_PRODUCTION_MANIFEST_SHA256 = "4c2a59a19d9d8fc05c8be9610d0687d99107498a5981756439e2b39eb876c24a"
EXPECTED_BYTES_SOURCE_SHA256 = "95789896965446187ecd5cc7bf52f447435fcdc40f4e19c0c49c5a59b22495fd"
EXPECTED_BYTES_RECORD_SHA256 = "cbf965c9da9d738a34c332ef9fdc09b39df9d21b15656c942a35aed81da1625b"
EXPECTED_NATIVE_VIEW_BINDINGS_SHA256 = "dfc6305f4b8ceebf4a0afa74429ae24f968d7b5da541ce4aff88cb92cefc2e00"
EXPECTED_SOURCE_MAP_SHA256 = "db9999dc8a2ddec2411df6bf817eafc89ec7383e617035963da52aa395166d22"

class AuditError(RuntimeError): pass
def require(ok: bool, message: str) -> None:
    if not ok: raise AuditError(message)
def sha(data: bytes) -> str: return hashlib.sha256(data).hexdigest()

def load_ap_checker():
    require(AP_CHECKER_PATH.is_file() and sha(AP_CHECKER_PATH.read_bytes()) == AP_CHECKER_SHA256,
            "frozen AP native checker dependency changed")
    spec=importlib.util.spec_from_file_location("aq_frozen_ap_native", AP_CHECKER_PATH)
    require(spec is not None and spec.loader is not None,"cannot load AP native checker")
    mod=importlib.util.module_from_spec(spec);sys.modules[spec.name]=mod;spec.loader.exec_module(mod)
    return mod
AP=load_ap_checker(); AI=AP.AI
AP_BASE=AP.load_bundle()

def tokens(text: str) -> list[str]: return AI.rust_tokens(text)
def require_tokens(actual: str, expected: str, label: str) -> None:
    require(tokens(actual)==tokens(expected),f"{label} token structure changed")
def token_count(hay: list[str], needle: list[str]) -> int:
    return sum(hay[i:i+len(needle)]==needle for i in range(len(hay)-len(needle)+1))

def extract_item(source: str, prefix: str, label: str) -> list[str]:
    ts=tokens(source); pre=tokens(prefix)
    hits=[i for i in range(len(ts)-len(pre)+1) if ts[i:i+len(pre)]==pre]
    require(len(hits)==1,f"expected one `{label}` item, found {len(hits)}")
    start=hits[0]
    brace=start+len(pre)-1 if pre[-1]=="{" else next((i for i in range(start+len(pre),len(ts)) if ts[i]=="{"),None)
    require(brace is not None,f"{label} has no body")
    depth=0
    for i in range(brace,len(ts)):
        if ts[i]=="{": depth+=1
        elif ts[i]=="}":
            depth-=1
            if depth==0:return ts[start:i+1]
    raise AuditError(f"unclosed {label} item")

def item_by_prefix(source: str,prefix: str,expected: str,label: str) -> None:
    require(extract_item(source,prefix,label)==tokens(expected),f"{label} source item changed")

def reject_trust(source: str,prefix: str,label: str) -> None:
    masked=AI.mask_noncode(source);start=masked.find(prefix)
    require(start>=0 and masked.find(prefix,start+1)<0,f"cannot uniquely locate {label}")
    prev=source.rfind("}",0,start);attrs=source[prev+1:start]
    require(re.search(r"#\s*\[\s*(?:trusted|assume|axiom|extern_spec)\b",attrs) is None,
            f"{label} has an unreviewed trust attribute")

def blocks_of(mir: str,label: str) -> dict[tuple[int,bool],str]:
    lines=mir.splitlines();out={};i=0
    while i<len(lines):
        m=re.fullmatch(r"    bb(\d+)( \(cleanup\))?: \{",lines[i])
        if not m:i+=1;continue
        key=(int(m.group(1)),m.group(2) is not None)
        require(key not in out,f"{label} duplicates bb{key[0]}");i+=1;body=[]
        while i<len(lines) and lines[i]!="    }":body.append(lines[i]);i+=1
        require(i<len(lines),f"{label} has unterminated bb{key[0]}")
        out[key]="\n".join(body);i+=1
    require(out,f"{label} has no MIR blocks")
    return out

def significant(body: str) -> list[str]:
    return [line.strip() for line in body.splitlines() if line.strip() and
            not line.strip().startswith(("StorageLive(","StorageDead(","debug ","PlaceMention("))]

def assert_sig_blocks(mir: str, normal: dict[int,list[str]], cleanup: dict[int,list[str]],label:str) -> None:
    bs=blocks_of(mir,label)
    expect={(n,False) for n in normal}|{(n,True) for n in cleanup}
    require(set(bs)==expect,f"{label} block/cleanup set changed")
    for (n,clean),lines in [*((((n,False),v) for n,v in normal.items())),*(((n,True),v) for n,v in cleanup.items())]:
        got=significant(bs[(n,clean)])
        require([tokens(x) for x in got]==[tokens(x) for x in lines],
                f"{label} {'cleanup ' if clean else ''}bb{n} operations/edge changed: {got}")

def require_mir(mir:str,snippet:str,label:str,count:int=1) -> None:
    actual=token_count(tokens(mir),tokens(snippet))
    require(actual==count,f"MIR {label} expected {count} occurrence(s) of {snippet!r}, found {actual}")

def load_bundle() -> dict[str,Any]:
    cap=json.loads(CAPTURE_PATH.read_text())
    source=(ROOT/"native.rs").read_text();test=(ROOT/"native-test/tests/clone_witness.rs").read_text()
    crate_source=(CRATE_ROOT/"src/bytes.rs").read_text()
    inp={p.relative_to(CRATE_ROOT).as_posix():p.read_text() for p in (CRATE_ROOT/"src").rglob("*") if p.is_file()}
    mir={label:(MIR_DIR/rel).read_text() for label,(rel,_) in MIR_EXPECTED.items()}
    return {"capture":cap,"native_source":source,"native_test_source":test,
      "native_test_log":(ROOT/"native-test/native-run.log").read_text(),
      "native_manifest":(ROOT/"native-test/Cargo.toml").read_text(),
      "native_lock":(ROOT/"native-test/Cargo.lock").read_text(),
      "capture_script":(ROOT/"capture-native.sh").read_text(),
      "rustc_text":(MIR_DIR/"rustc-version.txt").read_text(),"cargo_text":(MIR_DIR/"cargo-version.txt").read_text(),
      "mir_sources":mir,"production_source":crate_source,"production_source_inputs":inp,
      "production_manifest":(CRATE_ROOT/"Cargo.toml").read_text(),"production_lock":(CRATE_ROOT/"Cargo.lock").read_text(),
      "shared_record_source":(CRATE_ROOT/"src/bytes/shared_record.rs").read_text(),
      "bytes_record_source":(CRATE_ROOT/"src/bytes/bytes_record.rs").read_text(),
      "ref_count_source":(CRATE_ROOT/"src/ref_count_ops.rs").read_text(),"loom_source":(CRATE_ROOT/"src/loom.rs").read_text(),
      "reviewed_production_manifest":(ROOT/"reviewed-production-inputs.json").read_text(),
      "native_field_profile_source":(ROOT/"native-field-profile.rs").read_text(),
      "native_field_profile_log":(ROOT/"native-field-profile.log").read_text(),
      "extractor":(ROOT/"extract_public.py").read_text(),
      "native_view_bindings":(ROOT/"generated/native_view_bindings.rs").read_text(),
      "source_map_text":(ROOT/"generated/source-map.json").read_text(),
      "mapping_text":(ROOT/"generated/mapping.json").read_text()}

def code_slice(source:str, signature:str,label:str) -> str:
    masked=AI.mask_noncode(source); hits=[m.start() for m in re.finditer(re.escape(signature),masked)]
    require(len(hits)==1,f"expected exactly one executable {label} signature, found {len(hits)}")
    start=hits[0];open_brace=masked.find("{",start)
    require(open_brace>=0,f"{label} missing body")
    depth=0;state="code";escaped=False;i=open_brace
    while i<len(source):
        ch=source[i];two=source[i:i+2]
        if state=="line":
            if ch=="\n":state="code"
        elif state=="block":
            if two=="*/":state="code";i+=1
        elif state=="string":
            if escaped:escaped=False
            elif ch=="\\":escaped=True
            elif ch=='"':state="code"
        elif two=="//":state="line";i+=1
        elif two=="/*":state="block";i+=1
        elif ch=='"':state="string"
        elif ch=="{":depth+=1
        elif ch=="}":
            depth-=1
            if depth==0:return source[start:i+1]
        i+=1
    raise AuditError(f"unterminated {label}")

def audit_view_bindings(data:dict[str,Any]) -> dict[str,Any]:
    source=data["production_source"]
    sigs={"slice":"pub fn slice(&self, range: impl RangeBounds<usize>) -> Self",
      "new_empty_with_ptr":"fn new_empty_with_ptr(ptr: *const u8) -> Self",
      "static_clone":"unsafe fn static_clone(","static_drop":"unsafe fn static_drop(",
      "without_provenance":"fn without_provenance(ptr: usize) -> *const u8"}
    bodies={k:code_slice(source,v,k) for k,v in sigs.items()}
    expected={"slice":EXPECTED_SLICE,"new_empty_with_ptr":EXPECTED_NEW_EMPTY,
      "static_clone":EXPECTED_STATIC_CLONE,"static_drop":EXPECTED_STATIC_DROP,
      "without_provenance":EXPECTED_WITHOUT_PROVENANCE}
    for k in sigs: require_tokens(bodies[k],expected[k],f"production view body {k}")
    for k,v in sigs.items():reject_trust(source,v,k)
    vtable=r"""const STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    into_vec: static_to_vec,
    into_mut: static_to_mut,
    is_unique: static_is_unique,
    drop: static_drop,
};"""
    item_by_prefix(source,"const STATIC_VTABLE",EXPECTED_STATIC_VTABLE,"STATIC_VTABLE")
    item_by_prefix(source,"impl AsRef<[u8]> for Bytes",EXPECTED_AS_REF_IMPL,"AsRef<[u8]> route")
    expected_file="\n\n".join(bodies.values())+"\n"
    require(data["native_view_bindings"]==expected_file,
      "generated/native_view_bindings.rs is not the exact selected production body extraction")
    source_map=json.loads(data["source_map_text"])
    body_hashes={k:sha(v.encode()) for k,v in bodies.items()}
    require(source_map.get("view_bodies")==body_hashes,"source-map view-body identities differ from extracted production bodies")
    require(sha(data["native_view_bindings"].encode())==EXPECTED_NATIVE_VIEW_BINDINGS_SHA256,
      "generated native view bindings identity changed")
    require(sha(data["source_map_text"].encode())==EXPECTED_SOURCE_MAP_SHA256,
      "generated source-map identity changed")
    require(sha(data["extractor"].encode())==EXPECTED_SOURCE_SHA["extractor"],
      "native view extractor identity changed")
    generated={"path":"generated/native_view_bindings.rs",
      "sha256":sha(data["native_view_bindings"].encode()),"source_exact":True}
    smap={"path":"generated/source-map.json","sha256":sha(data["source_map_text"].encode()),
      "view_bodies_exact":True}
    return {"source_body_names":list(sigs),"source_body_hashes":body_hashes,
      "generated_native_view_bindings":generated,
      "source_map":smap,
      "view_bindings":{"generated_reconstructed":True,"source_map_reconstructed":True,
        "generated":generated,"source_map":smap},"extractor_sha256":sha(data["extractor"].encode()),
      "static_vtable_entries":{"clone":"static_clone","drop":"static_drop",
        "constructor_selects":"STATIC_VTABLE","all_exact":True},
      "as_ref_routes_to_as_slice":True}

def audit_source(data:dict[str,Any]) -> dict[str,Any]:
    require_tokens(data["native_source"],EXPECTED_NATIVE_SOURCE,"native nested slice client")
    require_tokens(data["native_test_source"],EXPECTED_NATIVE_TEST,"native range smoke test")
    require(sha(data["native_source"].encode())==EXPECTED_SOURCE_SHA["native.rs"],"native client source hash changed")
    require(sha(data["native_test_source"].encode())==EXPECTED_SOURCE_SHA["native_test"],"native test source hash changed")
    item_by_prefix(data["production_source"],"pub fn slice",EXPECTED_SLICE,"Bytes::slice")
    item_by_prefix(data["production_source"],"fn new_empty_with_ptr",EXPECTED_NEW_EMPTY,"Bytes::new_empty_with_ptr")
    item_by_prefix(data["production_source"],"unsafe fn static_clone",EXPECTED_STATIC_CLONE,"static_clone")
    item_by_prefix(data["production_source"],"unsafe fn static_drop",EXPECTED_STATIC_DROP,"static_drop")
    item_by_prefix(data["production_source"],"fn without_provenance",EXPECTED_WITHOUT_PROVENANCE,"without_provenance")
    # The inherited native bytes/Shared callback chain and no-drop field
    # profile are independently audited by the frozen AP checker.
    src=AP.audit_production_sources(data)
    fields=AP.audit_terminal_field_profile(data)
    return {"native_client_source_closed":True,"native_test_source_closed":True,
      "test_case_count":4*5*5,"test_lengths":[1,2,7,31],
      "outer_ranges":["full","interior-to-end","empty-at-zero","empty-at-end","empty-interior"],
      "inner_ranges":["full","interior-to-end","empty-at-zero","empty-at-end","empty-interior"],
      "execution_is_corroboration_only":True,
      "actual_builtin_range_only":True,"arbitrary_RangeBounds_implementations_excluded":True,
      "slice_source_body_checked":True,"new_empty_source_body_checked":True,
      "without_provenance_source_body_checked":True,"static_table_dispatch_checked":True,
      "as_ref_as_slice_source_route_checked":True,"inherited_callback_source_audit":src,
      "terminal_field_profile":fields}

def audit_reviewed_inputs(data:dict[str,Any]) -> dict[str,Any]:
    raw=data["reviewed_production_manifest"]
    require(sha(raw.encode())==EXPECTED_SOURCE_SHA["reviewed_manifest"],"reviewed production inputs manifest hash changed")
    manifest=json.loads(raw)
    require(manifest.get("schema")=="reviewed-original-production-inputs-v1" and
      manifest.get("base_commit")=="361c7cd261507ac0a705b3b836f73240070891c6" and manifest.get("crate")=="bytes/1.11.1",
      "reviewed production source anchor changed")
    files=manifest["files"];srcpaths={p for p in files if p.startswith("src/")}
    require(len(srcpaths)==61 and set(files)==srcpaths|{"Cargo.toml","Cargo.lock"},"reviewed production file inventory changed")
    require(set(data["production_source_inputs"])==srcpaths,"resolved production src tree differs from reviewed inventory")
    for p in srcpaths:require(sha(data["production_source_inputs"][p].encode())==files[p],f"reviewed production source changed: {p}")
    overrides={"Cargo.toml":data["production_manifest"],"Cargo.lock":data["production_lock"],
      "src/bytes.rs":data["production_source"],"src/bytes/bytes_record.rs":data["bytes_record_source"],
      "src/bytes/shared_record.rs":data["shared_record_source"],"src/ref_count_ops.rs":data["ref_count_source"],"src/loom.rs":data["loom_source"]}
    for p,v in overrides.items():require(sha(v.encode())==files[p],f"loaded production route differs from reviewed identity: {p}")
    return {"manifest_sha256":sha(raw.encode()),"base_commit":manifest["base_commit"],"source_files":61,
      "manifest_and_lock_pinned":True,"all_63_reviewed_inputs_match":True}

def audit_capture(data:dict[str,Any]) -> dict[str,Any]:
    c=data["capture"]
    require(sha(CAPTURE_PATH.read_bytes())==EXPECTED_CAPTURE_SHA256,"frozen native capture receipt changed")
    require(c.get("stage")=="2-2-004.ElaborateDrops.after.mir","native MIR stage changed")
    require(data["rustc_text"]==EXPECTED_RUSTC and c.get("rustc_version")==EXPECTED_RUSTC.rstrip("\n"),"rustc toolchain pin changed")
    require(data["cargo_text"]==EXPECTED_CARGO and c.get("cargo_version")==EXPECTED_CARGO.rstrip("\n"),"cargo toolchain pin changed")
    expected_paths={"native_source":("native.rs","native_source_sha256",data["native_source"]),
      "capture_script":("capture-native.sh","capture_script_sha256",data["capture_script"]),
      "native_manifest":("native-test/Cargo.toml","native_manifest_sha256",data["native_manifest"]),
      "native_lock":("native-test/Cargo.lock","native_lock_sha256",data["native_lock"]),
      "native_test_source":("native-test/tests/clone_witness.rs","native_test_source_sha256",data["native_test_source"]),
      "production_manifest":("../../../Cargo.toml","production_manifest_sha256",data["production_manifest"]),
      "production_source":("../../../src/bytes.rs","production_source_sha256",data["production_source"]),
      "native_test_log":("native-test/native-run.log","native_test_log_sha256",data["native_test_log"])}
    resolved={}
    for name,(rel,hkey,text) in expected_paths.items():
      require(c.get(name)==rel,f"native capture route changed: {name}")
      p=(ROOT/rel).resolve() if not rel.startswith("../../../") else (ROOT/rel).resolve()
      want=(CRATE_ROOT/rel.removeprefix("../../../")).resolve() if rel.startswith("../../../") else (ROOT/rel).resolve()
      require(p==want and p.is_file(),f"native capture path does not resolve as expected: {name}")
      require(c.get(hkey)==sha(text.encode()),f"native capture hash mismatch: {name}");resolved[name]=str(p)
    require(c.get("native_manifest_sha256")==EXPECTED_SOURCE_SHA["native_manifest"] and
      c.get("native_lock_sha256")==EXPECTED_SOURCE_SHA["native_lock"] and
      c.get("native_test_log_sha256")==EXPECTED_SOURCE_SHA["native_log"],"native test input hash differs from frozen values")
    require(sha(data["capture_script"].encode())==EXPECTED_SOURCE_SHA["capture_script"],"native capture procedure changed")
    require(sha(data["extractor"].encode())==EXPECTED_SOURCE_SHA["extractor"],"source extractor procedure changed")
    tm=tomllib.loads(data["native_manifest"]);require(tm==EXPECTED_MANIFEST,"native harness dependency or crate route changed")
    lock=tomllib.loads(data["native_lock"]);require(lock.get("version")==4,"native harness lockfile format changed")
    packages={(x.get("name"),x.get("version")) for x in lock.get("package",[])}
    require(("bytes","1.11.1") in packages and ("bytes-shared-slice-views-native","0.0.0") in packages,
      "native lockfile does not identify target crate and harness")
    require("test result: ok. 1 passed" in data["native_test_log"] and "test result: FAILED" not in data["native_test_log"],
      "captured native execution log does not show successful smoke test")
    for marker in ("-Zdump-mir=all","-Zmir-opt-level=0","-Zidentify-regions=yes","cargo test --locked --offline",
      "cargo rustc --locked --offline","../../../Cargo.toml"):
      require(marker in data["capture_script"],f"capture procedure missing {marker}")
    require("creusot" not in data["capture_script"].lower() and "why3" not in data["capture_script"].lower(),
      "native capture must not invoke proof tools")
    return {"resolved_capture_inputs":resolved,"rustc_pinned":True,"cargo_pinned":True,
      "native_manifest_exact":True,"native_lock_exact":True,"native_test_log_passed":True,
      "capture_script_sha256":sha(data["capture_script"].encode())}

def audit_headers(mirs:dict[str,str]) -> dict[str,Any]:
    require(set(mirs)==EXPECTED_MIR_LABELS,"captured MIR selected label set changed")
    norm=[]
    for label,source in mirs.items():
      headers=re.findall(r"(?m)^fn ([^\n]+)$",source)
      require(len(headers)==1,f"MIR {label} must contain one top-level function")
      h=headers[0]
      patterns={
        "client":r"nested_slice_scope\(_1: Box<\[u8\]>, _2: usize, _3: usize, _4: usize, _5: usize\) -> Vec<u8> \{",
        "from_box":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::from\(_1: Box<\[u8\]>\) -> bytes::Bytes \{",
        "clone_impl":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::clone\(_1: &bytes::Bytes\) -> bytes::Bytes \{",
        "cleanup":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::cleanup\(_1: bytes::Bytes\) -> \(\) \{",
        "bytes_drop":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::drop\(_1: &mut bytes::Bytes\) -> \(\) \{",
        "as_ref":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::as_ref\(_1: &bytes::Bytes\) -> &\[u8\] \{",
        "as_slice":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::as_slice\(_1: &bytes::Bytes\) -> &\[u8\] \{",
        "promotable_even_clone":r"promotable_even_clone\(_1: &Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> bytes::Bytes \{",
        "promotable_odd_clone":r"promotable_odd_clone\(_1: &Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> bytes::Bytes \{",
        "shallow_clone_vec":r"shallow_clone_vec\(_1: &Atomic<\*mut \(\)>, _2: \*const \(\), _3: \*mut u8, _4: \*const u8, _5: usize\) -> bytes::Bytes \{",
        "shallow_clone_arc":r"shallow_clone_arc\(_1: \*mut bytes::Shared, _2: \*const u8, _3: usize\) -> bytes::Bytes \{",
        "shared_clone":r"shared_clone\(_1: &Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> bytes::Bytes \{",
        "slice":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::slice\(_1: &bytes::Bytes, _2: impl RangeBounds<usize>\) -> bytes::Bytes \{",
        "new_empty_with_ptr":r"bytes::<impl at src/bytes\.rs:\d+:\d+: \d+:\d+>::new_empty_with_ptr\(_1: \*const u8\) -> bytes::Bytes \{",
        "static_clone":r"static_clone\(_1: &Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> bytes::Bytes \{",
        "static_drop":r"static_drop\(_1: &mut Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> \(\) \{",
        "without_provenance":r"bytes::without_provenance\(_1: usize\) -> \*const u8 \{",
        "ref_count_increment":r"increment\(_1: &Atomic<usize>\) -> \(\) \{",
        "promotable_even_drop":r"promotable_even_drop\(_1: &mut Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> \(\) \{",
        "promotable_odd_drop":r"promotable_odd_drop\(_1: &mut Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> \(\) \{",
        "shared_drop":r"shared_drop\(_1: &mut Atomic<\*mut \(\)>, _2: \*const u8, _3: usize\) -> \(\) \{",
        "release_shared":r"bytes::release_shared\(_1: \*mut bytes::Shared\) -> \(\) \{",
        "free_shared":r"free_shared\(_1: \*mut bytes::Shared\) -> \(\) \{",
        "ptr_map":r"ptr_map\(_1: \*mut u8, _2: F\) -> \*mut u8 \{",
        "atomic_with_mut":r"loom::sync::atomic::<impl at src/loom\.rs:\d+:\d+: \d+:\d+>::with_mut\(_1: &mut Atomic<\*mut T>, _2: F\) -> R \{",
      }
      require(label in patterns and re.fullmatch(patterns[label],h) is not None,f"selected MIR {label} header changed: {h}")
      norm.append(h.split("(",1)[0])
    require(len(norm)==len(set(norm)),"selected MIR contains duplicate callable symbols")
    return {"selected_mir_count_including_client":len(mirs),"selected_production_mir_count":len(mirs)-1,
      "selected_headers_unique":True}

EXPECTED_CLIENT_NORMAL = {
  0:["_9 = move _1;","_8 = <bytes::Bytes as From<Box<[u8]>>>::from(move _9) -> [return: bb1, unwind: bb17];"],
  1:["_10 = &'_ _8;","_7 = <bytes::Bytes as Clone>::clone(move _10) -> [return: bb2, unwind: bb16];"],
  2:["drop(_8) -> [return: bb3, unwind: bb18];"],
  3:["_12 = &'_ _7;","_14 = copy _2;","_15 = copy _3;","_13 = std::ops::Range::<usize> { start: move _14, end: move _15 };","_11 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _12, move _13) -> [return: bb4, unwind: bb15];"],
  4:["_16 = &'_ _11;","_18 = copy _4;","_19 = copy _5;","_17 = std::ops::Range::<usize> { start: move _18, end: move _19 };","_6 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _16, move _17) -> [return: bb5, unwind: bb14];"],
  5:["drop(_11) -> [return: bb6, unwind: bb15];"],
  6:["drop(_7) -> [return: bb7, unwind: bb18];"],
  7:["_24 = &'_ _6;","_23 = &'_ (*_24);","_22 = <bytes::Bytes as AsRef<[u8]>>::as_ref(move _23) -> [return: bb8, unwind: bb13];"],
  8:["_21 = &'_ (*_22);","_20 = slice::<impl [u8]>::to_vec(move _21) -> [return: bb9, unwind: bb13];"],
  9:["_0 = move _20;","goto -> bb10;"],
  10:["drop(_6) -> [return: bb11, unwind: bb18];"],
  11:["goto -> bb12;"],12:["return;"]}
EXPECTED_CLIENT_CLEANUP = {
  13:["drop(_6) -> [return: bb18, unwind terminate(cleanup)];"],
  14:["drop(_11) -> [return: bb15, unwind terminate(cleanup)];"],
  15:["drop(_7) -> [return: bb18, unwind terminate(cleanup)];"],
  16:["drop(_8) -> [return: bb18, unwind terminate(cleanup)];"],
  17:["goto -> bb18;"],18:["goto -> bb19;"],19:["resume;"]}

def audit_client_mir(mir:str,data_mapping_text:str) -> dict[str,Any]:
  assert_sig_blocks(mir,EXPECTED_CLIENT_NORMAL,EXPECTED_CLIENT_CLEANUP,"nested_slice_scope")
  calls=[line.strip() for n in sorted(EXPECTED_CLIENT_NORMAL) for line in blocks_of(mir,"client")[(n,False)].splitlines()
    if "-> [return:" in line]
  expected_calls=[line for n in sorted(EXPECTED_CLIENT_NORMAL) for line in EXPECTED_CLIENT_NORMAL[n] if "-> [return:" in line]
  require(calls==expected_calls,"native client normal call/drop order changed")
  require(calls.count("_6 = bytes::Bytes::slice::<std::ops::Range<usize>>(move _16, move _17) -> [return: bb5, unwind: bb14];")==1,
    "selected nested Range<usize> slice route absent")
  # Derive normal Drop edges from pinned MIR statements. Owner/scope labels are
  # closed against the exact source shape and then compared to the generator's
  # mapping as an independent cross-check.
  edge_profile=[(2,"_8","original","bb3","bb18","scope"),
    (5,"_11","first","bb6","bb15","detached"),
    (6,"_7","owner","bb7","bb18","detached"),
    (10,"_6","selected","bb11","bb18","detached")]
  normal_edges=[];bs=blocks_of(mir,"client")
  for block,place,owner,expected_successor,expected_unwind,scope in edge_profile:
    drops=[line.strip() for line in bs[(block,False)].splitlines()
      if re.search(rf"\bdrop\({re.escape(place)}(?:: bytes::Bytes)?\)",line)]
    require(len(drops)==1,f"expected one normal Drop for {owner} at {place} in bb{block}")
    dm=re.fullmatch(rf"drop\({re.escape(place)}(?:: bytes::Bytes)?\) -> \[return: bb(\d+), unwind: bb(\d+)\];",drops[0])
    require(dm is not None,f"unrecognized normal Drop edge for {owner}: {drops[0]}")
    edge={"block":f"bb{block}","place":place,"owner":owner,
      "successor":f"bb{dm.group(1)}","unwind":f"bb{dm.group(2)}",
      "repeated":False,"scope":scope}
    require(edge["successor"]==expected_successor and edge["unwind"]==expected_unwind,
      f"native normal Drop edge changed for {owner}")
    normal_edges.append(edge)
  mapping=json.loads(data_mapping_text)
  require(mapping.get("normal_edges")==normal_edges,
    "independently parsed client Drop edges disagree with generated/mapping.json.normal_edges")
  return {"normal_cfg_exact":True,"normal_edges":normal_edges,"calls_and_drops":calls,
    "range_specialization":{"native_type":"std::ops::Range<usize>","outer_call_count":2,
      "arguments":"outer a..b; inner c..d","arbitrary_RangeBounds_call_not_selected":True},
    "local_roles":{"Original":"_8","Owner":"_7","First":"_11","Selected":"_6","Observed":"_20"},
    "normal_drop_order":[{"role":"Original","place":"_8","block":"bb2","before":"first slice"},
      {"role":"First","place":"_11","block":"bb5","before":"Owner"},
      {"role":"Owner","place":"_7","block":"bb6","before":"selected read"},
      {"role":"Selected","place":"_6","block":"bb10","after":"to_vec and result move to return place"}],
    "selected_read_and_saved_return":{"as_ref":"bb7","to_vec":"bb8","result_saved":"bb9","selected_drop":"bb10"},
    "normal_completion_only":True,"unwind_claim":False}

def audit_slice_mir(mir:str) -> dict[str,Any]:
  bs=blocks_of(mir,"Bytes::slice")
  require(set(bs)=={*((i,False) for i in range(36)),(36,True),(37,True),(38,True)},"Bytes::slice full basic-block/cleanup topology changed")
  for snippet in [
    "_6 = <impl RangeBounds<usize> as RangeBounds<usize>>::start_bound(move _7) -> [return: bb2, unwind: bb37];",
    "_16 = <impl RangeBounds<usize> as RangeBounds<usize>>::end_bound(move _17) -> [return: bb10, unwind: bb37];",
    "_11 = core::num::<impl usize>::checked_add(move _12, const 1_usize) -> [return: bb7, unwind: bb37];",
    "_20 = core::num::<impl usize>::checked_add(move _21, const 1_usize) -> [return: bb14, unwind: bb37];",
    "_5 = Option::<usize>::expect(move _11, move _13) -> [return: bb8, unwind: bb37];",
    "_15 = Option::<usize>::expect(move _20, move _22) -> [return: bb15, unwind: bb37];",
    "_0 = bytes::Bytes::new_empty_with_ptr(move _68) -> [return: bb30, unwind: bb37];",
    "_71 = <bytes::Bytes as Clone>::clone(move _72) -> [return: bb31, unwind: bb37];",
    "_75 = SubWithOverflow(copy _73, copy _74);",
    "_76 = core::ptr::const_ptr::<impl *const u8>::add(move _77, move _78) -> [return: bb33, unwind: bb36];",
  ]: require_mir(mir,snippet,"slice branch")
  checks=[
    (2,"switchInt(move _8) -> [0: bb6, 1: bb5, 2: bb4, otherwise: bb3];"),
    (10,"switchInt(move _18) -> [0: bb13, 1: bb12, 2: bb11, otherwise: bb3];"),
    (16,"switchInt(move _26) -> [0: bb18, otherwise: bb17];"),
    (17,"switchInt(move _45) -> [0: bb23, otherwise: bb22];"),
    (22,"switchInt(move _64) -> [0: bb28, otherwise: bb27];"),
  ]
  for b,s in checks:require(tokens(s) in [tokens(x) for x in significant(bs[(b,False)])],f"Bytes::slice bb{b} control edge changed")
  require("wrapping_add" in bs[(27,False)] and "new_empty_with_ptr" not in bs[(27,False)],"empty branch no longer computes wrapping address before constructor")
  require("Bytes::new_empty_with_ptr" in bs[(29,False)],"empty branch does not use actual empty constructor")
  require("Bytes as Clone" not in bs[(27,False)] and "Bytes as Clone" in bs[(28,False)],"empty/nonempty clone split changed")
  require("Bytes::new_empty_with_ptr" not in bs[(28,False)],"nonempty branch unexpectedly uses empty constructor")
  require("wrapping_add" not in bs[(28,False)] and "::add(" in bs[(32,False)],"nonempty branch no longer uses actual ptr.add")
  require("SubWithOverflow(copy _73, copy _74)" in bs[(31,False)] and "(_71.1: usize) = move (_75.0: usize)" in bs[(32,False)],
    "nonempty length no longer comes from checked end - begin")
  require("(_71.0: *const u8) = move _76" in bs[(33,False)],"nonempty shifted pointer is not stored in the clone")
  # Bounds predicates must reject reversed and over-length ranges before either constructor path.
  require("Le(move _27, move _28)" in bs[(16,False)] and "Le(move _46, move _47)" in bs[(17,False)],
    "begin <= end / end <= len assertions changed")
  return {"normal_cfg_exact":True,"source_rangebounds_match_order_checked":True,"start_bound":{"block":"bb2","included":"bb6","excluded":"bb5","unbounded":"bb4"},
    "end_bound":{"block":"bb10","included":"bb13","excluded":"bb12","unbounded":"bb11"},
    "excluded_start_and_included_end_checked_add_calls":2,"bounds_assertions":["begin <= end (bb16/bb17, failure bb18)","end <= len (bb17/bb22, failure bb23)"],
    "empty_branch":{"condition":"end == begin (bb22)","edge":"bb27","pointer_op":"wrapping_add(begin)","constructor":"new_empty_with_ptr","clone":False},
    "nonempty_branch":{"edge":"bb28","clone":"self.clone()","length":"end - begin","pointer_op":"ret.ptr.add(begin)","empty_constructor":False},
    "normal_cfg_and_cleanup_blocks_checked":True}

def audit_empty_mir(data:dict[str,Any]) -> dict[str,Any]:
  mir=data["mir_sources"]
  empty=blocks_of(mir["new_empty_with_ptr"],"new_empty_with_ptr")
  require(set(empty)=={(i,False) for i in range(10)},"new_empty_with_ptr block/cleanup topology changed")
  require("is_null" in empty[(1,False)] and "switchInt(move _5) -> [0: bb4, otherwise: bb3]" in empty[(2,False)],
    "empty constructor lost non-null debug assertion path")
  require("panic(const \"assertion failed: !ptr.is_null()\")" in empty[(3,False)],"empty constructor null rejection changed")
  require("PointerExposeProvenance" in empty[(6,False)] and "bytes::without_provenance(move _9)" in empty[(6,False)],
    "empty constructor does not detach to the exact no-provenance helper")
  require("Atomic::<*mut ()>::new(move _13)" in empty[(8,False)] and "null_mut::<()>" in empty[(7,False)],
    "empty constructor atomic data field is no longer null")
  require("new_empty_with_ptr::promoted[0]" in empty[(9,False)] and
    "len: const 0_usize" in empty[(9,False)] and "vtable: move _14" in empty[(9,False)],
    "empty constructor is not length zero with its selected static-vtable promoted value")
  wp=blocks_of(mir["without_provenance"],"without_provenance")
  assert_sig_blocks(mir["without_provenance"],{0:["_2 = null::<u8>() -> [return: bb1, unwind continue];"],
    1:["_3 = copy _1;","_0 = core::ptr::const_ptr::<impl *const u8>::wrapping_add(move _2, move _3) -> [return: bb2, unwind continue];"],
    2:["return;"]},{},"without_provenance")
  sc=blocks_of(mir["static_clone"],"static_clone")
  require("from_raw_parts" in sc[(0,False)] and "Bytes::from_static" in sc[(1,False)],"static clone callback route changed")
  sd=blocks_of(mir["static_drop"],"static_drop")
  assert_sig_blocks(mir["static_drop"],{0:["_0 = const ();","return;"]}, {},"static_drop")
  return {"constructor_nonnull_assertion":True,"pointer_address_exposed_then_without_provenance":True,
    "empty_native_fields":{"len":0,"atomic_data":"null_mut","vtable":"STATIC_VTABLE"},
    "without_provenance":{"body":"null::<u8>().wrapping_add(address)","MIR_checked":True,"same_numeric_address_only":True,"allocation_provenance_claim":False},
    "static_vtable":{"clone":"static_clone","drop":"static_drop"},"static_clone_body_and_MIR_checked":True,
    "static_drop_no_effect_body_and_MIR_checked":True,"empty_has_no_ticket_or_allocation_authority_claim":True}

def audit_mirs(data:dict[str,Any]) -> dict[str,Any]:
  mir=data["mir_sources"];headers=audit_headers(mir)
  client=audit_client_mir(mir["client"],data["mapping_text"]);slice_facts=audit_slice_mir(mir["slice"]);empty=audit_empty_mir(data)
  # Re-run the frozen AP production callback MIR audit against AQ's selected
  # production MIR; retain AP's client only for that inherited finite-owner suite.
  inherited=dict(AP_BASE["mir_sources"])
  for label in set(inherited)-{"client"}:
    require(label in mir,f"AQ MIR omits inherited callback {label}")
    inherited[label]=mir[label]
  old=AP.audit_native_mir({"mir_sources":inherited})
  return {**headers,"client":client,"slice_mir":slice_facts,"empty_mir":empty,
    "inherited_native_callback_mir_audit":old,
    "selected_MIR_sha256":{k:sha(v.encode()) for k,v in mir.items()},
    "MIR_capture_header_sha256":EXPECTED_CAPTURE_SHA256}

def audit_capture_mir_hashes(data:dict[str,Any]) -> dict[str,Any]:
  c=data["capture"];rows=c.get("selected",[])
  require(len(rows)==25 and {x.get("label") for x in rows}==EXPECTED_MIR_LABELS,"capture receipt does not select exact 25 MIR bodies")
  by={x["label"]:x for x in rows};resolved={}
  for label,(rel,want) in MIR_EXPECTED.items():
    row=by[label];require(row.get("path")=="native-mir/"+rel,f"MIR path changed: {label}")
    require(row.get("sha256")==want and sha(data["mir_sources"][label].encode())==want,f"MIR identity changed: {label}")
    p=(ROOT/row["path"]).resolve();require(p== (MIR_DIR/rel).resolve() and p.is_file(),f"MIR input path changed: {label}")
    resolved[label]=str(p)
  return {"selected":resolved,"all_25_mir_hashes_match_frozen_pins":True,"production_mir_count":24}

def audit_bundle(data:dict[str,Any]|None=None) -> dict[str,Any]:
  if data is None:data=load_bundle()
  source=audit_source(data);views=audit_view_bindings(data);inputs=audit_reviewed_inputs(data)
  cap=audit_capture(data);client= audit_client_mir(data["mir_sources"]["client"],data["mapping_text"])
  slice_facts=audit_slice_mir(data["mir_sources"]["slice"]);empty=audit_empty_mir(data)
  callbacks=AP.audit_production_sources(data)
  fields=AP.audit_terminal_field_profile(data)
  mir=audit_mirs(data);mir_paths=audit_capture_mir_hashes(data)
  facts={"status":"pass","checker_scope":"AQ built-in Range<usize> nested slice source/MIR mapping plus closed production callback provenance; no Bytes proof or MIR adequacy claim",
    "paths_and_capture":cap,"reviewed_production_inputs":inputs,"source":source,"view_bindings":views,
    "client":client,"slice_mir":slice_facts,"empty_mir":empty,"production_callbacks":callbacks,
    "terminal_field_profile":fields,"native_mir":mir,"mir_paths":mir_paths,
    "native_audit":{"source_body_names":views["source_body_names"],"source_body_hashes":views["source_body_hashes"],
      "generated_view_bindings":views["generated_native_view_bindings"],"source_map":views["source_map"],
      "view_bindings":views["view_bindings"],
      "trait_routing":{"AsRef":"Bytes::as_slice","nonempty_clone":"Bytes::Clone -> stored vtable.clone -> promotable/shared callback","empty_constructor":"Bytes::new_empty_with_ptr -> without_provenance -> STATIC_VTABLE","shared_clone":"Relaxed data load -> shallow_clone_arc","static_drop":"static_drop no-op"},
      "range_specialization":{"selected_builtin_type":"Range<usize>",**client["range_specialization"]},
      "slice_mir":slice_facts,"slice_branches":slice_facts,
      "empty_view":empty,"normal_drop_order":client["normal_drop_order"],
      "normal_read_return":client["selected_read_and_saved_return"],"selected_mir_count":25,
      "production_mir_count":24,"native_test":{"case_count":source["test_case_count"],"lengths":source["test_lengths"],
        "outer_ranges":source["outer_ranges"],"inner_ranges":source["inner_ranges"],"execution_only":True}},
    "limits":["Only the actual built-in Range<usize> path is selected; arbitrary RangeBounds implementations are excluded.",
      "Native execution cases are corroboration, not a proof and do not cover all runtime endpoints.",
      "Invalid ranges/panic completion and unwind are excluded; only valid-range normal completion is checked by the client mapping.",
      "Null wrapping_add address reconstruction, Rust pointer provenance and MIR adequacy remain external language/compiler TCB.",
      "The empty handle has no owner ticket or allocation authority; STATIC_VTABLE drop is a no-op.",
      "Native Vec/output allocation and generic Std behavior are outside this source/MIR gate."]}
  return facts

def main() -> int:
  p=argparse.ArgumentParser();p.add_argument("--output",type=pathlib.Path);a=p.parse_args()
  try: result=audit_bundle()
  except AuditError as e:
    result={"status":"reject","checker_scope":"AQ native source/MIR correspondence","reason":str(e)}
  rendered=json.dumps(result,indent=2,ensure_ascii=False)+"\n"
  if a.output:a.output.write_text(rendered)
  print(rendered,end="")
  return 0 if result.get("status")=="pass" else 1
if __name__=="__main__":raise SystemExit(main())
