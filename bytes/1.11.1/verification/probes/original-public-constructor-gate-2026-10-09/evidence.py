#!/usr/bin/env python3
"""Immutable, portable source/task capture and independent archive audit."""
from __future__ import annotations
import argparse, hashlib, json, pathlib, tarfile

R = pathlib.Path(__file__).resolve().parent
C = R.parents[2]
T = pathlib.Path('/workspace/bytes-proof-tools')
def sha(b): return hashlib.sha256(b).hexdigest()
def proof_stats(proofs):
    stats = dict(files=len(proofs), prover=0, null=0, structural=0)
    def visit(tree):
        if tree is None: stats['null'] += 1
        elif 'children' in tree:
            if not tree['children']: stats['structural'] += 1
            for child in tree['children']: visit(child)
        elif 'prover' in tree: stats['prover'] += 1
        else: raise ValueError(tree)
    for proof in proofs:
        for tree in proof['proofs']['Coma'].values(): visit(tree)
    return stats

def collect(label, log, status):
    evidence = R / 'evidence'; evidence.mkdir(exist_ok=True)
    archive = evidence / (label + '.tar.gz')
    manifest = evidence / (label + '.json')
    assert not archive.exists() and not manifest.exists(), 'immutable label already exists'
    files = {}
    def tree(base, prefix, skip=()):
        for path in base.rglob('*'):
            if path.is_file() and not any(p in skip for p in path.relative_to(base).parts):
                files[prefix + '/' + path.relative_to(base).as_posix()] = path
    tree(R, 'probe', ('evidence', 'target', '.proof-config', '.git', '__pycache__'))
    tree(C / 'src', 'inputs/production/src')
    for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
        files['inputs/production/' + name] = C / name
    for name in ['original-public-shared-gate-2026-10-08', 'guarded-shared-protocol-2026-10-08',
                 'shared-physical-lifecycle-2026-10-08', 'original-shared-lifecycle-2026-10-08']:
        tree(R.parent / name / 'src', 'inputs/' + name + '/src')
    tree(T / 'bytes-proof-std', 'inputs/private-std', ('target', '.git'))
    for name, path in [('installation-manifest.json', T/'installation-manifest.json'),
                       ('activate.sh', T/'activate.sh'),
                       ('creusot_why3.conf', T/'creusot-data/creusot_why3.conf'),
                       ('why3-main.conf', T/'config/creusot/why3.conf')]:
        files['inputs/tools/' + name] = path
    files['run.log'] = pathlib.Path(log).resolve()
    targets = json.loads((R/'generated/proof-targets.json').read_text())
    target_rows = []
    data = []
    for coma in targets['included']:
        proof = pathlib.Path(coma).with_suffix('') / 'proof.json'
        cp = R / coma; pp = R / proof
        assert cp.is_file()
        row = dict(coma='probe/'+coma, coma_sha256=sha(cp.read_bytes()), proof='probe/'+proof.as_posix())
        if pp.is_file():
            row['proof_sha256'] = sha(pp.read_bytes()); data.append(json.loads(pp.read_text()))
        else: row['proof_missing'] = True
        target_rows.append(row)
    stats = proof_stats(data)
    if status == 'proved':
        assert stats['null'] == 0 and len(data) == len(target_rows)
        assert not targets['excluded']
        assert len([x for x in targets['included'] if x.endswith('/from__refines.coma')]) == 2
    rows=[]
    with tarfile.open(archive, 'w:gz') as tar:
        for name,path in sorted(files.items()):
            tar.add(path, arcname=name, recursive=False)
            rows.append(dict(path=name, sha256=sha(path.read_bytes())))
    report=dict(archive=archive.name, archive_sha256=sha(archive.read_bytes()), status=status,
                statistics=stats, targets=target_rows, target_policy=targets,
                members=rows, scope='Constructor/read source gate; never whole crate or final-recovery completion.')
    manifest.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ['archive','archive_sha256','status','statistics']}))

def audit(label):
    report=json.loads((R/'evidence'/(label+'.json')).read_text())
    archive=R/'evidence'/report['archive']
    assert sha(archive.read_bytes()) == report['archive_sha256']
    proofs=[]
    with tarfile.open(archive) as tar:
        members={m.name:m for m in tar.getmembers()}
        assert len(members) == len(report['members'])
        assert set(members) == {r['path'] for r in report['members']}
        for row in report['members']:
            assert sha(tar.extractfile(row['path']).read()) == row['sha256'], row['path']
        for row in report['targets']:
            assert sha(tar.extractfile(row['coma']).read()) == row['coma_sha256']
            if row.get('proof_missing'): continue
            p=tar.extractfile(row['proof']).read()
            assert sha(p) == row['proof_sha256']
            proofs.append(json.loads(p))
        assert proof_stats(proofs) == report['statistics']
        targets=json.load(tar.extractfile('probe/generated/proof-targets.json'))
        assert targets == report['target_policy']
        source_map=json.load(tar.extractfile('probe/generated/source-map.json'))
        production=tar.extractfile('inputs/production/src/bytes.rs').read().decode()
        for name,begin,end in [
            ('bytes_from_vec_impl','ORIGINAL_SHARED_BEGIN','ORIGINAL_SHARED_END'),
            ('bytes_from_box_impl','ORIGINAL_CONSTRUCTOR_BEGIN','ORIGINAL_CONSTRUCTOR_END'),
            ('bytes_new','ORIGINAL_CONSTRUCTOR_BEGIN','ORIGINAL_CONSTRUCTOR_END'),
            ('bytes_from_static','ORIGINAL_CONSTRUCTOR_BEGIN','ORIGINAL_CONSTRUCTOR_END'),
            ('bytes_as_slice','ORIGINAL_SHARED_BEGIN','ORIGINAL_SHARED_END'),
            ('bytes_as_ref_impl','ORIGINAL_SHARED_BEGIN','ORIGINAL_SHARED_END')]:
            bmark=f'// {begin} {name}\n'; emark=f'// {end} {name}'
            assert production.count(bmark)==1 and production.count(emark)==1
            start=production.index(bmark); stop=production.index(emark,start)+len(emark)
            assert sha(production[start:stop].encode())==source_map[name]['sha256'], name
            assert production[:start].count('\n')+1==source_map[name]['line'], name
        assert source_map['from_refinements']['constructor_gate_preconditions']==[]
        for name,row in source_map.items():
            if name.startswith('generated/'):
                assert sha(tar.extractfile('probe/'+name).read()) == row['sha256'], name
    if report['status']=='proved':
        assert report['statistics']['null']==0 and not targets['excluded']
        assert len(proofs)==len(report['targets'])
    result=dict(archive_sha256=report['archive_sha256'], members_verified=len(members),
                status=report['status'],statistics=report['statistics'],
                excluded=targets['excluded'],portable=True,scope=report['scope'])
    (R/'evidence'/(label+'-audit.json')).write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(); commands=parser.add_subparsers(dest='command',required=True)
    c=commands.add_parser('capture');c.add_argument('label');c.add_argument('log');c.add_argument('--status',choices=['proved','failed','diagnostic'],required=True)
    a=commands.add_parser('audit');a.add_argument('label')
    args=parser.parse_args()
    if args.command=='capture': collect(args.label,args.log,args.status)
    else:audit(args.label)
