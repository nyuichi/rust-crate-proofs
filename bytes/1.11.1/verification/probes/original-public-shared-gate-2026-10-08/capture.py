#!/usr/bin/env python3
import argparse, hashlib, json, pathlib, tarfile
p=argparse.ArgumentParser(); p.add_argument('label'); p.add_argument('log'); a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent; evidence=root/'evidence'; evidence.mkdir(exist_ok=True)
out=evidence/(a.label+'.tar.gz'); assert not out.exists(), out
paths=[x for x in root.rglob('*') if x.is_file() and not any(k in x.relative_to(root).parts for k in ['evidence','target','.proof-config']) and (x.suffix in ['.rs','.toml','.lock','.json','.coma','.sh','.py'] or x.name==a.log)]
extra=[root/'../../../src/ref_count_limit.rs',pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std/src/std/sync/atomic.rs'),pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std/src/std/sync/committer.rs'),pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std/src/std/sync/view.rs'),pathlib.Path('/workspace/bytes-proof-tools/creusot-source/examples/logically_atomic_faa.rs')]
rows=[]
with tarfile.open(out,'w:gz') as t:
    for x in paths:
        name=str(x.relative_to(root)); t.add(x,arcname=name); rows.append({'path':name,'sha256':hashlib.sha256(x.read_bytes()).hexdigest()})
    for x in extra:
        name='inputs/'+x.name; t.add(x,arcname=name); rows.append({'path':name,'sha256':hashlib.sha256(x.read_bytes()).hexdigest(),'source':str(x.resolve())})
manifest={'archive':out.name,'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'members':rows}
(evidence/(a.label+'.json')).write_text(json.dumps(manifest,indent=2)+'\n')
print(manifest['archive'],manifest['sha256'],len(rows))
