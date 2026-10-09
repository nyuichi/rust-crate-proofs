#!/usr/bin/env python3
"""Emit AZ source/native indexes without generating or changing Rust bodies.

Mapping hashes are integrity indexes. Only the separate operational checker
establishes the bounded source/native correspondence.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT=Path(__file__).resolve().parent
AY=ROOT.parent/'original-root-phase-view-2026-10-09'
PREFIX_SHA='21f1b7acb2267f5e38938691003d81fdc25dea93a2016be3b74178b98c87a6ed'
RECEIPT_SHA='c10232a3db94caa520eeabf25b9be1387c71fb1e41cd17cab5aecf27d09b3dc4'
SELECTED_SOURCE_SHA='c85b8797cc13f98f2949e2d9e06d09b19ec46794e7da65e5c9e13bbc700e2486'
MIRROR_SHA='29b2bdf603c982f6c025ca0e54faa896b85c286142e960611bd9230f110bcf44'
NATIVE_CHECKER_SHA='eb6413cef39d685cd06b96878483f29c653170d3680a076608a16ae6ac257974'
OUTER_SHA='71ed49af75822b9281ec87b7d0009c13463cf4989ebbcf71f36705cc586139d8'

def sha(raw):return hashlib.sha256(raw).hexdigest()

def build(feature=''):
    assert feature=='','AZ supports only the applicable positive configuration'
    prefix=(AY/'src/promotion.rs').read_bytes()
    assert sha(prefix)==PREFIX_SHA,'published AY prefix changed'
    source=(ROOT/'src/promotion.rs').read_bytes()
    assert source.startswith(prefix) and len(source)>len(prefix),'AZ requires append-only new bodies'
    assert sha(source)==SELECTED_SOURCE_SHA and sha((ROOT/'src/root_clone_extension.rs').read_bytes())==MIRROR_SHA, 'frozen AZ source changed'
    assert b'\n'+(ROOT/'src/root_clone_extension.rs').read_bytes()==source[len(prefix):], 'AZ appendix mirror differs'
    inherited=json.loads((ROOT/'inherited-targets.json').read_bytes())
    receipt_raw=(AY/'evidence/AY_FULL_PROOF_RUN.json').read_bytes()
    assert sha(receipt_raw)==RECEIPT_SHA,'published AY full positive receipt changed'
    receipt=json.loads(receipt_raw)
    assert receipt['statistics']==dict(files=186,prover=1955,null=0,structural=0)
    assert receipt['features']==[] and receipt['excluded']=={} and receipt['prover_process_exit']==0
    package='verif/bytes_original_root_phase_view_rlib/'
    relative=sorted(r['path'][len(package):] for r in receipt['proof_file_identity'] if r['path'].endswith('.coma'))
    assert len(relative)==len(set(relative))==186 and relative==inherited['relative_targets']
    assert inherited['ancestor_full_proof_receipt_sha256']==RECEIPT_SHA and inherited['ancestor_outer_manifest_sha256']==OUTER_SHA
    scaffold=json.loads((ROOT/'AZ_SCAFFOLD.json').read_bytes())
    inherited_sources={r['path']:r for r in scaffold['source_inventory']}
    source_rows=[]
    for path in sorted((ROOT/'src').glob('*.rs')):
        raw=path.read_bytes();relative_path='src/'+path.name
        if relative_path in inherited_sources and path.name not in ('promotion.rs','lib.rs'):
            assert sha(raw)==inherited_sources[relative_path]['sha256'],f'inherited support changed: {relative_path}'
        source_rows.append(dict(path=relative_path,sha256=sha(raw),size=len(raw)))
    assert set(inherited_sources)<=set(r['path'] for r in source_rows),'inherited source module disappeared'
    new_functions=re.findall(r'\bfn\s+([A-Za-z_][A-Za-z0-9_]*)',source[len(prefix):].decode())
    assert len(new_functions)==len(set(new_functions)),'new function names are duplicated'
    native=None
    if (ROOT/'check_native.py').is_file():
        assert sha((ROOT/'check_native.py').read_bytes())==NATIVE_CHECKER_SHA, 'frozen AZ native checker changed'
        spec=importlib.util.spec_from_file_location('az_native_mapping',ROOT/'check_native.py')
        assert spec and spec.loader
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        native=module.audit_bundle(module.load_bundle())
        assert native['status']=='pass','fresh AZ native audit failed'
    mapping=dict(schema='ay-root-phase-view-mapping-v1',feature=feature,full_original_admitted=False,source_prefix_sha256=PREFIX_SHA,source_inventory=source_rows,selected_source_sha256=sha(source),appendix_sha256=sha(source[len(prefix):]),new_functions=new_functions,inherited_target_count=186,inherited_targets=relative,ancestor_outer_manifest_sha256=OUTER_SHA,ancestor_full_proof_receipt_sha256=RECEIPT_SHA,native_report=native,correspondence_status='not_run; hashes and native facts alone do not establish shadow operation joins')
    generated=ROOT/'generated';generated.mkdir(exist_ok=True)
    (generated/'mapping.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n')
    return dict(status='metadata_emitted',selected_source_sha256=sha(source),new_function_names=new_functions,inherited_targets=186,native_audit_present=native is not None,full_original_admitted=False)

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--feature',default='');args=parser.parse_args()
    print(json.dumps(build(args.feature),indent=2))

if __name__=='__main__':main()
