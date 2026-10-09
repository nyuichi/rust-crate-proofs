#!/usr/bin/env python3
"""Immutable portable capture of the runtime-variable finite-owner development experiment."""
import argparse,hashlib,json,pathlib,tarfile
R=pathlib.Path(__file__).resolve().parent
T=pathlib.Path('/workspace/bytes-proof-tools')
def sha(b): return hashlib.sha256(b).hexdigest()
def stats(proofs):
    out=dict(files=len(proofs),prover=0,null=0,structural=0)
    def visit(t):
        if t is None:out['null']+=1
        elif 'children' in t:
            if not t['children']:out['structural']+=1
            for x in t['children']:visit(x)
        elif 'prover' in t:out['prover']+=1
        else:raise ValueError(t)
    for p in proofs:
        for t in p['proofs']['Coma'].values():visit(t)
    return out

def capture(label,log,status):
    folder=R/'evidence';folder.mkdir(exist_ok=True)
    archive=folder/(label+'.tar.gz');manifest=folder/(label+'.json')
    assert not archive.exists() and not manifest.exists(),'immutable label exists'
    files={}
    def tree(base,prefix,skip=()):
        for p in base.rglob('*'):
            if p.is_file() and not any(x in skip for x in p.relative_to(base).parts):
                files[prefix+'/'+p.relative_to(base).as_posix()]=p
    tree(R,'probe',('evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(T/'bytes-proof-std','inputs/private-std',('target','.git','__pycache__'))
    for n,p in [('installation-manifest.json',T/'installation-manifest.json'),
                ('activate.sh',T/'activate.sh'),('creusot_why3.conf',T/'creusot-data/creusot_why3.conf'),
                ('why3-main.conf',T/'config/creusot/why3.conf')]:files['inputs/tools/'+n]=p
    files['inputs/tools/cargo-config.toml']=T/'cargo/config.toml'
    files['inputs/tools/base-activate.sh']=pathlib.Path('/workspace/proof-tools/activate.sh')
    files['run.log']=pathlib.Path(log).resolve()
    repository=R.parents[4]
    for name in ['Cargo.toml','Cargo.lock','Cargo.toml.orig']:
        path=repository/'bytes/1.11.1'/name
        if path.exists(): files['inputs/repository/bytes/1.11.1/'+name]=path
    tree(repository/'bytes/1.11.1/src','inputs/repository/bytes/1.11.1/src')
    for name in ['shared-physical-lifecycle-2026-10-08','original-public-shared-gate-2026-10-08','scoped-issuance-cursor-2026-10-09','original-shared-lifecycle-2026-10-08','guarded-shared-protocol-2026-10-08']:
        base=repository/'bytes/1.11.1/verification/probes'/name/'src'
        tree(base,'inputs/repository/bytes/1.11.1/verification/probes/'+name+'/src')
    tree(repository/'bytes/1.11.1/verification/probes/original-shared-scoped-client-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-shared-scoped-client-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-public-constructor-gate-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-public-constructor-gate-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-boxed-automatic-drop-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-boxed-automatic-drop-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-promotable-first-clone-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-promotable-first-clone-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-shared-automatic-drop-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-shared-automatic-drop-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-promotable-automatic-drop-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-promotable-automatic-drop-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-promotable-reclone-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-promotable-reclone-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(repository/'bytes/1.11.1/verification/probes/original-promotable-surviving-child-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-promotable-surviving-child-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    policy=json.loads((R/'generated/proof-targets.json').read_text())
    rows=[];proofs=[]
    for c in policy['included']:
        p=pathlib.Path(c).with_suffix('')/'proof.json'
        row=dict(coma='probe/'+c,coma_sha256=sha((R/c).read_bytes()),proof='probe/'+p.as_posix())
        if (R/p).exists():
            row['proof_sha256']=sha((R/p).read_bytes());proofs.append(json.loads((R/p).read_text()))
        else:row['proof_missing']=True
        rows.append(row)
    summary=stats(proofs)
    if status=='proved':
        assert summary['null']==0 and len(proofs)==len(rows) and not policy['excluded']
        assert policy['correspondence_exit_status']==0 and not policy['features']
        assert len(rows)==130 and sorted(policy['included'])==sorted(p.relative_to(R).as_posix() for p in (R/'verif').rglob('*.coma')) and not policy.get("terminal_feature") and not policy.get('diagnostic',False) and not policy.get('source_control','')
    members=[]
    with tarfile.open(archive,'w:gz') as tar:
        for name,path in sorted(files.items()):
            tar.add(path,arcname=name,recursive=False);members.append(dict(path=name,sha256=sha(path.read_bytes())))
    receipt=dict(archive=archive.name,archive_sha256=sha(archive.read_bytes()),status=status,
                 statistics=summary,targets=rows,target_policy=policy,members=members,
                 scope='Runtime-variable finite Shared child cloning after original retirement with actual Vec push/pop inventory and peer normal Drops; surviving child read, saved return, final child and empty Vec normal Drop, nonempty Box both parities; no CAS loser/concurrency/unwind/full-crate admission.')
    manifest.write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:receipt[k] for k in ['archive','archive_sha256','status','statistics']}))

def audit(label):
    r=json.loads((R/'evidence'/(label+'.json')).read_text());a=R/'evidence'/r['archive']
    assert sha(a.read_bytes())==r['archive_sha256']
    proofs=[]
    with tarfile.open(a) as tar:
        names=tar.getnames();assert len(names)==len(set(names))==len(r['members'])
        assert set(names)=={x['path'] for x in r['members']}
        for x in r['members']:assert sha(tar.extractfile(x['path']).read())==x['sha256'],x['path']
        for x in r['targets']:
            assert sha(tar.extractfile(x['coma']).read())==x['coma_sha256']
            if x.get('proof_missing'):continue
            b=tar.extractfile(x['proof']).read();assert sha(b)==x['proof_sha256'];proofs.append(json.loads(b))
        assert stats(proofs)==r['statistics']
        assert json.load(tar.extractfile('probe/generated/proof-targets.json'))==r['target_policy']
    if r['status']=='proved':
        assert r['statistics']['null']==0 and not r['target_policy']['excluded']
        assert len(proofs)==len(r['targets']) and r['target_policy']['correspondence_exit_status']==0
        assert not r['target_policy']['features']
    report=dict(archive_sha256=r['archive_sha256'],members_verified=len(names),statistics=r['statistics'],
                status=r['status'],self_contained_toolchain=False,scope=r['scope'],
                correspondence='Stored receipt hashed; use independent checker reconstruction for adequacy evidence.')
    (R/'evidence'/(label+'-audit.json')).write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();s=p.add_subparsers(dest='action',required=True)
    c=s.add_parser('capture');c.add_argument('label');c.add_argument('log');c.add_argument('--status',choices=['proved','failed','diagnostic'],required=True)
    a=s.add_parser('audit');a.add_argument('label');q=p.parse_args()
    if q.action=='capture':capture(q.label,q.log,q.status)
    else:audit(q.label)
