#!/usr/bin/env python3
"""Emit AY source/native indexes without generating or changing Rust bodies.

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
AX=ROOT.parent/'original-raw-suffix-drop-2026-10-09'
PREFIX_SHA='9440ac43f7a116e4bd36b5affb449334be8872e34f9e9cb7a9dc0c923311d414'
RECEIPT_SHA='9499ae885e6339eaca9897bce69ea46c3fd133c9ef28357ffc77ca63cf9a54fd'
SELECTED_SOURCE_SHA='21f1b7acb2267f5e38938691003d81fdc25dea93a2016be3b74178b98c87a6ed'
MIRROR_SHA='118144396db1a92e6dd3e9e6ed1b313d807599dacbeab0936a37ff2af87ba61e'
NATIVE_CHECKER_SHA='d0afa9123d6187b463c827118645a145cbd84b5e7eb9edfb4bd58ffd3f690372'
OUTER_SHA='05cca18b78627f28340e01f86abc52c7e75669734f1143389b832d7a1db5e8a5'

def sha(raw):return hashlib.sha256(raw).hexdigest()

def build(feature=''):
    assert feature=='','AY supports only the applicable positive configuration'
    prefix=(AX/'src/promotion.rs').read_bytes()
    assert sha(prefix)==PREFIX_SHA,'published AX prefix changed'
    source=(ROOT/'src/promotion.rs').read_bytes()
    assert source.startswith(prefix) and len(source)>len(prefix),'AY requires append-only new bodies'
    assert sha(source)==SELECTED_SOURCE_SHA and sha((ROOT/'src/root_phase_extension.rs').read_bytes())==MIRROR_SHA, 'frozen AY source changed'
    assert b'\n'+(ROOT/'src/root_phase_extension.rs').read_bytes()==source[len(prefix):], 'AY appendix mirror differs'
    inherited=json.loads((ROOT/'inherited-targets.json').read_bytes())
    receipt_raw=(AX/'evidence/AX_FULL_PROOF_RUN.json').read_bytes()
    assert sha(receipt_raw)==RECEIPT_SHA,'published AX full positive receipt changed'
    receipt=json.loads(receipt_raw)
    assert receipt['statistics']==dict(files=177,prover=1798,null=0,structural=0)
    assert receipt['features']==[] and receipt['excluded']=={} and receipt['prover_process_exit']==0
    package='verif/bytes_original_raw_suffix_drop_rlib/'
    relative=sorted(r['path'][len(package):] for r in receipt['proof_file_identity'] if r['path'].endswith('.coma'))
    assert len(relative)==len(set(relative))==177 and relative==inherited['relative_targets']
    assert inherited['ancestor_full_proof_receipt_sha256']==RECEIPT_SHA and inherited['ancestor_outer_manifest_sha256']==OUTER_SHA
    scaffold=json.loads((ROOT/'AY_SCAFFOLD.json').read_bytes())
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
        assert sha((ROOT/'check_native.py').read_bytes())==NATIVE_CHECKER_SHA, 'frozen AY native checker changed'
        spec=importlib.util.spec_from_file_location('ay_native_mapping',ROOT/'check_native.py')
        assert spec and spec.loader
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        native=module.audit_bundle(module.load_bundle())
        assert native['status']=='pass','fresh AY native audit failed'
    mapping=dict(schema='ay-root-phase-view-mapping-v1',feature=feature,full_original_admitted=False,source_prefix_sha256=PREFIX_SHA,source_inventory=source_rows,selected_source_sha256=sha(source),appendix_sha256=sha(source[len(prefix):]),new_functions=new_functions,inherited_target_count=177,inherited_targets=relative,ancestor_outer_manifest_sha256=OUTER_SHA,ancestor_full_proof_receipt_sha256=RECEIPT_SHA,native_report=native,correspondence_status='not_run; hashes and native facts alone do not establish shadow operation joins')
    generated=ROOT/'generated';generated.mkdir(exist_ok=True)
    (generated/'mapping.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n')
    return dict(status='metadata_emitted',selected_source_sha256=sha(source),new_function_names=new_functions,inherited_targets=177,native_audit_present=native is not None,full_original_admitted=False)

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--feature',default='');args=parser.parse_args()
    print(json.dumps(build(args.feature),indent=2))

if __name__=='__main__':main()
