#!/usr/bin/env python3
"""Capture AU admission metadata and its complete proof-reuse ancestry."""
import argparse,hashlib,io,json,pathlib,tarfile
R=pathlib.Path(__file__).resolve().parent
T=pathlib.Path('/workspace/bytes-proof-tools')
AT_TARGETS=155
AT_PROVER_LEAVES=1417
AU_TARGETS=157
AU_PROVER_LEAVES=1447
AU_SCOPE=('Selected modular borrowed-input/owning-return client over the complete published AT positive source. '
          'The callee Clone then advances, returns an owner under a body-proved summary, and the caller uses its '
          'contract without inlining. This does not admit general Clone/From, arbitrary escaping/concurrent '
          'ownership, unwind or the full original crate.')
AU_DIAGNOSTIC_ARCHIVE_SHA='ffc00ffe50a408f971f2a56e1d58f7173d3d9856ebeb65e97c7a725d9e9393bd'
AU_DIAGNOSTIC_RECEIPT_SHA='6a57cba12daee897e79a4214e8627ba7ecc933133545abd8e742737a3c7956ee'
AU_DIAGNOSTIC_MEMBERS=1543
AT_ARCHIVE_SHA='958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d'
AT_RECEIPT_SHA='78e9694d4e0150ee6a566c62b96c698b7117bfe4302af53bd78115c65b6fab4e'
AT_AUDIT_CHECKER_SHA='54ccb9469212fcc5306ca564827f630069b5603c04a50d07fcb323f6ad102150'
AT_ARCHIVE_MEMBERS=1544
AT_PACKAGE_PATH='inputs/repository/bytes/1.11.1/verification/probes/original-owned-view-clone-2026-10-09'
AU_IDENTITY_PATH=pathlib.Path('/workspace/work/au-proof-reuse-identity.json')
AU_ORIGIN_RUN_LOG=pathlib.Path('/workspace/work/au-full-diagnostic-v1.log')
AU_ADMISSION_CORRESPONDENCE_LOG=pathlib.Path('/workspace/work/au-full-correspondence.log')
AU_COMPILED_CAPTURE_LOG=pathlib.Path('/workspace/work/au-compiled-capture.log')
AU_INDEPENDENT_ORIGIN_AUDIT=pathlib.Path('/workspace/work/au-diagnostic-independent-audit.json')
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

def regular(path):
    assert path.is_file() and not path.is_symlink(),f'expected a regular non-symlink file: {path}'
    return path.read_bytes()

def add_file(files,key,path):
    assert key not in files, f'duplicate archive path: {key}'
    regular(path)
    files[key]=path

def verify_reuse_identity(identity,origin_receipt,origin_files):
    assert identity.get('status')=='pass'
    assert identity.get('origin_archive')=='au-full-diagnostic-v1.tar.gz'
    assert identity.get('origin_archive_sha256')==AU_DIAGNOSTIC_ARCHIVE_SHA
    assert identity.get('statistics')==dict(files=AU_TARGETS,prover=AU_PROVER_LEAVES,null=0,structural=0)
    assert identity.get('prover_reexecuted') is False
    rows=identity.get('all_target_files_sha256')
    assert isinstance(rows,list) and len(rows)==AU_TARGETS*2
    table={row.get('path'):row.get('sha256') for row in rows}
    assert len(table)==len(rows) and all(isinstance(p,str) and isinstance(h,str) for p,h in table.items())
    manifest=origin_receipt.get('targets',[])
    assert len(manifest)==AU_TARGETS
    expected={}
    for row in manifest:
        for field in ('coma','proof'):
            path=row[field].removeprefix('probe/')
            expected[path]=row[field+'_sha256']
    assert table==expected, 'reuse identity target table differs from immutable origin manifest'
    for path,digest in table.items():
        assert sha(origin_files['probe/'+path])==digest, path
    assert identity.get('Rust_active_sha256')==sha(origin_files['probe/generated/active.rs'])
    return table

def verify_target_snapshot(policy,receipt,identity_table,current_files):
    assert policy.get('included') and len(policy['included'])==AU_TARGETS
    assert policy.get('excluded')=={} and policy.get('features')==[]
    assert policy.get('scope')==AU_SCOPE
    assert policy.get('diagnostic') is True and policy.get('correspondence_exit_status')==2
    assert not policy.get('terminal_feature') and not policy.get('source_control')
    included=sorted(policy['included'])
    expected_comas=sorted(p for p in current_files if p.endswith('.coma'))
    assert included==expected_comas, 'admission target inventory is not the complete generated target set'
    pairs=[];proofs=[]
    for coma in included:
        proof=pathlib.Path(coma).with_suffix('')/'proof.json'
        proof_name=proof.as_posix()
        assert coma in current_files and proof_name in current_files, f'incomplete AU proof pair: {coma}'
        pairs.extend([(coma,sha(current_files[coma])),(proof_name,sha(current_files[proof_name]))])
        proofs.append(json.loads(current_files[proof_name]))
    assert len(pairs)==AU_TARGETS*2 and dict(pairs)==identity_table
    computed=stats(proofs)
    expected=dict(files=AU_TARGETS,prover=AU_PROVER_LEAVES,null=0,structural=0)
    assert computed==expected
    assert receipt.get('statistics')==expected
    return computed,pairs

def checked_tar_bytes(data,expected_sha,expected_members,label):
    assert sha(data)==expected_sha, f'{label} archive digest changed'
    result={}
    with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as tar:
        entries=tar.getmembers(); names=[x.name for x in entries]
        assert len(names)==len(set(names))==expected_members, f'{label} archive member set is not regular/unique'
        for entry in entries:
            path=pathlib.PurePosixPath(entry.name)
            assert entry.isfile() and not path.is_absolute() and '..' not in path.parts
            member=tar.extractfile(entry)
            assert member is not None
            result[entry.name]=member.read()
    return result

def capture(label,log,status,reuse_identity=None,preflight=False):
    folder=R/'evidence';folder.mkdir(exist_ok=True)
    archive=folder/(label+'.tar.gz');manifest=folder/(label+'.json')
    assert not archive.exists() and not manifest.exists(),'immutable label exists'
    assert status=='admitted_reuse','AU admission capture must explicitly record proof reuse, not a new proof run'
    policy=json.loads(regular(R/'generated/proof-targets.json'))
    correspondence_bytes=regular(R/'generated/correspondence.json')
    correspondence=json.loads(correspondence_bytes)
    capture_summary_bytes=regular(R/'generated/compiled-capture-summary.json')
    capture_summary=json.loads(capture_summary_bytes)
    control_bytes=regular(R/'generated/checker-controls-receipt.json')
    controls=json.loads(control_bytes)
    control_fixture=json.loads(regular(R/'fixtures/checker-controls.json'))
    assert control_fixture.get('schema')=='au_named_direct_call_control_v1'
    control_rows=control_fixture.get('controls',[])
    assert len(control_rows)==1
    expected_control=control_rows[0]
    assert expected_control.get('id')=='refreshed_native_callee_identity'
    assert expected_control.get('field')=='named_call_summaries[0].native_callee'
    assert expected_control.get('expected_rejection')=="named-call mapping field 'native_callee' differs from native source/MIR facts"
    identity_path=pathlib.Path(reuse_identity or AU_IDENTITY_PATH).resolve()
    identity_bytes=regular(identity_path)
    identity=json.loads(identity_bytes)
    origin_archive=regular(R/'evidence/au-full-diagnostic-v1.tar.gz')
    origin_receipt_bytes=regular(R/'evidence/au-full-diagnostic-v1.json')
    origin_receipt=json.loads(origin_receipt_bytes)
    origin_files=checked_tar_bytes(origin_archive,AU_DIAGNOSTIC_ARCHIVE_SHA,AU_DIAGNOSTIC_MEMBERS,'AU diagnostic proof origin')
    target_table=verify_reuse_identity(identity,origin_receipt,origin_files)
    assert sha(origin_receipt_bytes)==AU_DIAGNOSTIC_RECEIPT_SHA
    assert origin_receipt.get('archive_sha256')==AU_DIAGNOSTIC_ARCHIVE_SHA
    assert origin_receipt.get('status')=='diagnostic'
    origin_policy=origin_receipt.get('target_policy',{})
    assert origin_receipt.get('statistics')==dict(files=AU_TARGETS,prover=AU_PROVER_LEAVES,null=0,structural=0)
    assert origin_policy.get('diagnostic') is True and origin_policy.get('correspondence_exit_status')==2
    assert origin_policy.get('features')==[] and origin_policy.get('excluded')=={}
    origin_log=origin_files.get('run.log',b'')
    assert b'Proved (157 files)' in origin_log, 'origin log lacks the successful proof completion marker'
    assert correspondence.get('status')=='pass' and correspondence.get('full_original_admitted') is False
    assert correspondence.get('checker_scope')=='AU complete source/native/compiled capture audit; no live Cargo target'
    assert correspondence.get('AU_completed_proof_reuse',{}).get('origin_archive_sha256')==AU_DIAGNOSTIC_ARCHIVE_SHA
    assert correspondence.get('AU_completed_proof_reuse',{}).get('prover_rerun_claimed') is False
    compiled=correspondence.get('AU_compiled_production_input',{})
    assert compiled.get('status')=='pass' and compiled.get('captured_artifact_count')==4
    assert all(compiled.get(k) is True for k in ('fingerprint_inputs_exact','build_output_exact','OUT_DIR_root_output_join_exact','public_records_reconstructed'))
    assert capture_summary.get('status')=='pass'
    assert capture_summary.get('AU_compiled_production_input',{}).get('captured_artifact_count')==4
    assert capture_summary.get('AU_compiled_production_input',{}).get('snapshot_written') is True
    assert controls.get('status')=='pass' and controls.get('control_count')==1 and controls.get('rejected_as_expected')==1
    assert controls.get('accepted')==[] and controls.get('errors')==[]
    assert controls.get('control_id')==expected_control['id']
    assert controls.get('expected_rejection')==expected_control['expected_rejection']
    assert controls.get('checker_sha256')==sha(regular(R/'check_correspondence.py'))
    assert controls.get('fixture_sha256')==sha(regular(R/'fixtures/checker-controls.json'))
    current_files={}
    for coma in sorted((R/'verif').rglob('*.coma')):
        relative=coma.relative_to(R).as_posix()
        proof=coma.with_suffix('')/'proof.json'
        assert proof.is_file() and not proof.is_symlink(),f'missing AU proof pair: {relative}'
        current_files[relative]=regular(coma)
        current_files[proof.relative_to(R).as_posix()]=regular(proof)
    current_stats,current_rows=verify_target_snapshot(policy,origin_receipt,target_table,current_files)
    assert current_stats==identity.get('statistics')
    admission_log=pathlib.Path(log).resolve()
    admission_log_bytes=regular(admission_log)
    assert b'"status": "pass"' in admission_log_bytes or b'"status":"pass"' in admission_log_bytes
    assert sha(regular(AU_ORIGIN_RUN_LOG))==sha(origin_files['run.log'])
    assert b'Proved (157 files)' in regular(AU_ORIGIN_RUN_LOG)
    assert sha(regular(AU_ADMISSION_CORRESPONDENCE_LOG))==sha(admission_log_bytes)
    assert correspondence.get('AU_named_call_summary',{}).get('body_proof_target_present') is True
    assert correspondence.get('AU_named_call_summary',{}).get('caller_modular_use_proved') is True
    assert correspondence.get('AU_named_call_summary',{}).get('trusted_summary') is False
    assert correspondence.get('AU_completed_proof_reuse',{}).get('current_source_Cargo_and_all_target_pairs_byte_exact') is True
    assert correspondence.get('AU_completed_proof_reuse',{}).get('current_rust_and_coma_inputs_identical_to_completed_proof') is True
    assert correspondence.get('AU_completed_proof_reuse',{}).get('new_admission_depends_on_current_correspondence_and_receipt') is True
    at_probe=R.parent/'original-owned-view-clone-2026-10-09'
    at_archive=regular(at_probe/'evidence/at-positive-canonical-v2.tar.gz')
    at_receipt_bytes=regular(at_probe/'evidence/at-positive-canonical-v2.json')
    at_receipt=json.loads(at_receipt_bytes)
    at_files=checked_tar_bytes(at_archive,AT_ARCHIVE_SHA,AT_ARCHIVE_MEMBERS,'published AT canonical v2')
    assert sha(at_receipt_bytes)==AT_RECEIPT_SHA and at_receipt.get('archive_sha256')==AT_ARCHIVE_SHA
    assert at_receipt.get('status')=='proved' and at_receipt.get('statistics')==dict(files=AT_TARGETS,prover=AT_PROVER_LEAVES,null=0,structural=0)
    at_audit=json.loads(regular(at_probe/'evidence/AT_CANONICAL_AUDIT.json'))
    assert at_audit.get('result')=='pass' and at_audit.get('archive_sha256')==AT_ARCHIVE_SHA
    assert at_audit.get('archive_integrity',{}).get('all_coma_and_proof_hashes_match') is True
    assert at_audit.get('archive_integrity',{}).get('diagnostic') is False
    assert correspondence.get('published_AT_ancestry',{}).get('canonical_archive_replayed') is True
    assert at_files.get('probe/check_correspondence.py') is not None
    assert sha(at_files['probe/check_correspondence.py'])==AT_AUDIT_CHECKER_SHA
    assert correspondence.get('published_AT_ancestry',{}).get('canonical_archive',{}).get('sha256')==AT_ARCHIVE_SHA
    assert correspondence.get('published_AT_ancestry',{}).get('published_AT_checker_sha256')==AT_AUDIT_CHECKER_SHA
    files={}
    def tree(base,prefix,skip=()):
        assert base.is_dir() and not base.is_symlink(),f'missing/redirected archive tree: {base}'
        for p in base.rglob('*'):
            assert not p.is_symlink(),f'symlink in evidence input tree: {p}'
            if p.is_file() and not any(x in skip for x in p.relative_to(base).parts):
                add_file(files,prefix+'/'+p.relative_to(base).as_posix(),p)
    tree(R,'probe',('evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    tree(T/'bytes-proof-std','inputs/private-std',('target','.git','__pycache__'))
    for n,p in [('installation-manifest.json',T/'installation-manifest.json'),
                ('activate.sh',T/'activate.sh'),('creusot_why3.conf',T/'creusot-data/creusot_why3.conf'),
                ('why3-main.conf',T/'config/creusot/why3.conf')]:add_file(files,'inputs/tools/'+n,p)
    add_file(files,'inputs/tools/cargo-config.toml',T/'cargo/config.toml')
    add_file(files,'inputs/tools/why3-local-derived.conf',R/'.proof-config/creusot/why3.conf')
    add_file(files,'inputs/tools/base-activate.sh',pathlib.Path('/workspace/proof-tools/activate.sh'))
    add_file(files,'admission/run.log',admission_log)
    add_file(files,'admission/au-proof-reuse-identity.json',identity_path)
    add_file(files,'admission/au-proof-reuse-origin-audit.json',AU_INDEPENDENT_ORIGIN_AUDIT)
    add_file(files,'admission/au-compiled-capture.log',AU_COMPILED_CAPTURE_LOG)
    add_file(files,'admission/au-full-correspondence.log',AU_ADMISSION_CORRESPONDENCE_LOG)
    add_file(files,'inputs/au-proof-origin/au-full-diagnostic-v1.log',AU_ORIGIN_RUN_LOG)
    add_file(files,'inputs/au-proof-origin/au-full-diagnostic-v1.tar.gz',R/'evidence/au-full-diagnostic-v1.tar.gz')
    add_file(files,'inputs/au-proof-origin/au-full-diagnostic-v1.json',R/'evidence/au-full-diagnostic-v1.json')
    repository=R.parents[4]
    for name in ['Cargo.toml','Cargo.lock','Cargo.toml.orig']:
        path=repository/'bytes/1.11.1'/name
        if path.exists(): add_file(files,'inputs/repository/bytes/1.11.1/'+name,path)
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
    tree(repository/'bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09',
         'inputs/repository/bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    as_probe=repository/'bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09'
    tree(as_probe,'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09',
         ('evidence','verif','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    for name in ['AS_CANONICAL_AUDIT.json','AS_CANONICAL_AUDIT.md',
                 'as-positive-canonical-v1-audit.json','as-positive-canonical-v1.tar.gz',
                 'as-positive-canonical-v1.json','AS_EXTERNAL_TOOL_HASH_CHECK.json']:
        add_file(files,'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/'+name,as_probe/'evidence'/name)
    ap=repository/'bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09'
    for name in ['AP_CANONICAL_AUDIT.json','AP_CANONICAL_AUDIT.md','ap-positive-canonical-v1.tar.gz','ap-positive-canonical-v1.json']:
        add_file(files,'inputs/repository/bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09/evidence/'+name,ap/'evidence'/name)
    aq=repository/'bytes/1.11.1/verification/probes/original-shared-slice-views-2026-10-09'
    tree(aq,'inputs/repository/bytes/1.11.1/verification/probes/original-shared-slice-views-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    for name in ['AQ_CANONICAL_AUDIT.json','AQ_CANONICAL_AUDIT.md','aq-positive-canonical-v1.tar.gz','aq-positive-canonical-v1.json']:
        add_file(files,'inputs/repository/bytes/1.11.1/verification/probes/original-shared-slice-views-2026-10-09/evidence/'+name,aq/'evidence'/name)
    ar=repository/'bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09'
    tree(ar,'inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09',
         ('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    for name in ['AR_CANONICAL_AUDIT.json','AR_CANONICAL_AUDIT.md',
                 'ar-positive-canonical-v2.tar.gz','ar-positive-canonical-v2.json']:
        add_file(files,'inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09/evidence/'+name,ar/'evidence'/name)
    at=repository/'bytes/1.11.1/verification/probes/original-owned-view-clone-2026-10-09'
    at_prefix=AT_PACKAGE_PATH
    tree(at,at_prefix,('verif','evidence','target','.git','__pycache__','.proof-config','.why3find','.why3findcache'))
    for name in ['AT_CANONICAL_AUDIT.json','AT_CANONICAL_AUDIT.md',
                 'AT_EXTERNAL_TOOL_HASH_CHECK.json','at-positive-canonical-v2-audit.json',
                 'at-positive-canonical-v2.tar.gz','at-positive-canonical-v2.json']:
        add_file(files,at_prefix+'/evidence/'+name,at/'evidence'/name)
    if preflight:
        assert len(files)==len(set(files))
        print(json.dumps({'preflight':'pass','archive_members':len(files),
                          'target_pairs':len(current_rows)//2,
                          'origin_archive_sha256':AU_DIAGNOSTIC_ARCHIVE_SHA,
                          'AT_archive_sha256':AT_ARCHIVE_SHA,
                          'correspondence_sha256':sha(correspondence_bytes),
                          'control_count':controls['control_count']},indent=2))
        return
    origin_run_log_sha=sha(regular(AU_ORIGIN_RUN_LOG))
    control_receipt_sha=sha(control_bytes)
    correspondence_sha=sha(correspondence_bytes)
    capture_summary_sha=sha(capture_summary_bytes)
    admission_log_sha=sha(admission_log_bytes)
    proof_reuse=dict(
        reused_full_proof=True,
        prover_rerun_claimed=False,
        origin_archive='au-full-diagnostic-v1.tar.gz',
        origin_archive_sha256=AU_DIAGNOSTIC_ARCHIVE_SHA,
        origin_receipt_sha256=AU_DIAGNOSTIC_RECEIPT_SHA,
        origin_run_log_sha256=origin_run_log_sha,
        origin_status='diagnostic_non_admitted',
        origin_prover_process_exit_status='not-recorded-in-archive',
        origin_prover_completion_marker='Proved (157 files)',
        origin_prover_completion_marker_present=True,
        origin_correspondence_exit_status=2,
        current_target_pair_count=AU_TARGETS,
        current_coma_and_proof_hashes_match_origin=True,
        current_rust_cargo_tool_and_configuration_identity_match=True,
        identity_report_sha256=sha(identity_bytes),
        identity_report_path='admission/au-proof-reuse-identity.json',
    )
    new_admission=dict(
        admission_mode='current_correspondence_and_capture_after_exact_proof_reuse',
        correspondence_status='pass',
        correspondence_exit_status=0,
        correspondence_sha256=correspondence_sha,
        correspondence_log_sha256=admission_log_sha,
        compiled_capture_status='pass',
        compiled_artifact_count=4,
        compiled_capture_summary_sha256=capture_summary_sha,
        compiled_capture_log_sha256=sha(regular(AU_COMPILED_CAPTURE_LOG)),
        structural_control_count=1,
        structural_control_receipt_sha256=control_receipt_sha,
        structural_control_result='1 forged native callee mapping rejected by full source/native/proof-reuse audit',
        full_prover_rerun=False,
    )
    target_rows=[]
    for c in policy['included']:
        proof=pathlib.Path(c).with_suffix('')/'proof.json'
        target_rows.append(dict(coma='probe/'+c,coma_sha256=sha(current_files[c]),
                                proof='probe/'+proof.as_posix(),
                                proof_sha256=sha(current_files[proof.as_posix()])))
    summary=current_stats
    members=[]
    with tarfile.open(archive,'w:gz') as tar:
        for name,path in sorted(files.items()):
            tar.add(path,arcname=name,recursive=False);members.append(dict(path=name,sha256=sha(path.read_bytes())))
    receipt=dict(archive=archive.name,archive_sha256=sha(archive.read_bytes()),status=status,
                 statistics=summary,targets=target_rows,target_policy=policy,members=members,
                 scope=AU_SCOPE,proof_reuse=proof_reuse,new_admission=new_admission,
                 checker_scope=correspondence['checker_scope'],
                 ancestry=dict(AT_archive_sha256=AT_ARCHIVE_SHA,AT_receipt_sha256=AT_RECEIPT_SHA,
                               AT_archive_replayed_by_AU_checker=True,AT_full_source_and_checker_tree_embedded=True),
                 origin_policy_disclosure='The origin target-policy receipt is diagnostic because the then-current correspondence gate was skipped (exit 2). Its captured prover log completed all 157 targets; this archive does not claim a new prover run. Admission is based on exact proof/source identity plus the new passing AU correspondence, fresh Cargo capture, and structural control.')
    manifest.write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:receipt[k] for k in ['archive','archive_sha256','status','statistics']}))

def audit(label):
    receipt_path=R/'evidence'/(label+'.json')
    r=json.loads(regular(receipt_path));a=R/'evidence'/r['archive'];archive_bytes=regular(a)
    assert sha(archive_bytes)==r['archive_sha256']
    assert r.get('status')=='admitted_reuse' and r.get('scope')==AU_SCOPE
    files=checked_tar_bytes(archive_bytes,r['archive_sha256'],len(r['members']),'AU admission archive')
    member_rows=r['members'];member_table={x['path']:x['sha256'] for x in member_rows}
    assert len(member_table)==len(member_rows)==len(files)
    assert set(files)==set(member_table)
    for name,data in files.items():assert sha(data)==member_table[name],name

    policy=json.loads(files['probe/generated/proof-targets.json'])
    assert policy==r['target_policy'] and policy.get('scope')==AU_SCOPE
    assert policy.get('diagnostic') is True and policy.get('correspondence_exit_status')==2
    assert policy.get('features')==[] and policy.get('excluded')=={}
    assert len(policy.get('included',[]))==AU_TARGETS
    assert r['statistics']==dict(files=AU_TARGETS,prover=AU_PROVER_LEAVES,null=0,structural=0)
    assert len(r['targets'])==AU_TARGETS

    target_table={}
    proofs=[]
    for row in r['targets']:
        coma=row['coma'];proof=row['proof']
        assert coma.startswith('probe/verif/') and proof.startswith('probe/verif/')
        assert coma in files and proof in files
        assert sha(files[coma])==row['coma_sha256']
        assert sha(files[proof])==row['proof_sha256']
        coma_rel=coma.removeprefix('probe/')
        proof_rel=proof.removeprefix('probe/')
        assert coma_rel not in target_table and proof_rel not in target_table
        target_table[coma_rel]=row['coma_sha256'];target_table[proof_rel]=row['proof_sha256']
        proofs.append(json.loads(files[proof]))
    assert len(target_table)==AU_TARGETS*2
    assert sorted('probe/'+x for x in policy['included'])==sorted(x['coma'] for x in r['targets'])
    assert stats(proofs)==r['statistics']

    origin_archive=files['inputs/au-proof-origin/au-full-diagnostic-v1.tar.gz']
    origin_receipt_bytes=files['inputs/au-proof-origin/au-full-diagnostic-v1.json']
    origin_run_log=files['inputs/au-proof-origin/au-full-diagnostic-v1.log']
    origin=json.loads(origin_receipt_bytes)
    origin_files=checked_tar_bytes(origin_archive,AU_DIAGNOSTIC_ARCHIVE_SHA,AU_DIAGNOSTIC_MEMBERS,'embedded AU proof origin')
    assert sha(origin_receipt_bytes)==AU_DIAGNOSTIC_RECEIPT_SHA
    assert origin.get('status')=='diagnostic' and origin.get('archive_sha256')==AU_DIAGNOSTIC_ARCHIVE_SHA
    assert origin.get('statistics')==r['statistics']
    assert origin.get('target_policy',{}).get('diagnostic') is True
    assert origin.get('target_policy',{}).get('correspondence_exit_status')==2
    assert origin.get('target_policy',{}).get('features')==[] and origin.get('target_policy',{}).get('excluded')=={}
    assert origin_files.get('run.log')==origin_run_log
    assert b'Proved (157 files)' in origin_run_log
    origin_targets={}
    for row in origin.get('targets',[]):
        origin_targets[row['coma'].removeprefix('probe/')]=row['coma_sha256']
        origin_targets[row['proof'].removeprefix('probe/')]=row['proof_sha256']
    assert origin_targets==target_table,'admitted AU target pairs differ from immutable diagnostic origin'

    identity_bytes=files['admission/au-proof-reuse-identity.json']
    identity=json.loads(identity_bytes)
    verified_identity=verify_reuse_identity(identity,origin,origin_files)
    assert verified_identity==target_table
    assert sha(identity_bytes)==r['proof_reuse']['identity_report_sha256']
    assert files['admission/au-proof-reuse-origin-audit.json']
    assert sha(origin_run_log)==r['proof_reuse']['origin_run_log_sha256']
    assert r['proof_reuse'].get('origin_correspondence_exit_status')==2
    assert r['proof_reuse'].get('reused_full_proof') is True and r['proof_reuse'].get('prover_rerun_claimed') is False
    assert r['proof_reuse'].get('current_coma_and_proof_hashes_match_origin') is True
    assert r['proof_reuse'].get('current_rust_cargo_tool_and_configuration_identity_match') is True
    assert r['proof_reuse'].get('origin_prover_process_exit_status')=='not-recorded-in-archive'
    assert r['proof_reuse'].get('origin_prover_completion_marker_present') is True
    assert r['proof_reuse'].get('origin_prover_completion_marker')=='Proved (157 files)'

    correspondence_bytes=files['probe/generated/correspondence.json']
    correspondence=json.loads(correspondence_bytes)
    capture_summary=json.loads(files['probe/generated/compiled-capture-summary.json'])
    assert correspondence.get('status')=='pass' and correspondence.get('full_original_admitted') is False
    assert correspondence.get('checker_scope')==r.get('checker_scope')
    assert correspondence.get('checker_scope')=='AU complete source/native/compiled capture audit; no live Cargo target'
    reuse=correspondence.get('AU_completed_proof_reuse',{})
    assert reuse.get('origin_archive_sha256')==AU_DIAGNOSTIC_ARCHIVE_SHA
    assert reuse.get('prover_rerun_claimed') is False and reuse.get('new_admission_depends_on_current_correspondence_and_receipt') is True
    summary=correspondence.get('AU_compiled_production_input',{})
    assert summary.get('status')=='pass' and summary.get('captured_artifact_count')==4
    assert all(summary.get(k) is True for k in ('fingerprint_inputs_exact','build_output_exact','OUT_DIR_root_output_join_exact','public_records_reconstructed'))
    assert capture_summary.get('status')=='pass'
    compiled=capture_summary.get('AU_compiled_production_input',{})
    assert compiled.get('status')=='pass' and compiled.get('captured_artifact_count')==4 and compiled.get('snapshot_written') is True
    assert compiled.get('snapshot_matches_live_build',{}).get('status')=='pass'
    expected_compiled={'public_records.rs','cargo-run-build-fingerprint.json','cargo-build-output.txt','cargo-root-output.txt','public-records-build-receipt.json'}
    actual_compiled={pathlib.PurePosixPath(n).name for n in files if n.startswith('probe/generated/compiled-inputs/')}
    assert actual_compiled==expected_compiled
    control=json.loads(files['probe/generated/checker-controls-receipt.json'])
    fixture=files['probe/fixtures/checker-controls.json']
    control_fixture=json.loads(fixture)
    assert control_fixture.get('schema')=='au_named_direct_call_control_v1'
    control_rows=control_fixture.get('controls',[])
    assert len(control_rows)==1
    expected_control=control_rows[0]
    assert expected_control.get('id')=='refreshed_native_callee_identity'
    assert expected_control.get('field')=='named_call_summaries[0].native_callee'
    assert expected_control.get('expected_rejection')=="named-call mapping field 'native_callee' differs from native source/MIR facts"
    assert control.get('status')=='pass' and control.get('control_count')==1 and control.get('rejected_as_expected')==1
    assert control.get('accepted')==[] and control.get('errors')==[]
    assert control.get('control_id')==expected_control['id']
    assert control.get('expected_rejection')==expected_control['expected_rejection']
    assert control.get('checker_sha256')==sha(files['probe/check_correspondence.py'])
    assert control.get('fixture_sha256')==sha(fixture)
    assert sha(correspondence_bytes)==r['new_admission']['correspondence_sha256']
    assert sha(files['admission/run.log'])==r['new_admission']['correspondence_log_sha256']
    assert sha(files['probe/generated/compiled-capture-summary.json'])==r['new_admission']['compiled_capture_summary_sha256']
    assert sha(files['probe/generated/checker-controls-receipt.json'])==r['new_admission']['structural_control_receipt_sha256']
    assert r['new_admission']['correspondence_exit_status']==0 and r['new_admission']['full_prover_rerun'] is False

    at_prefix=AT_PACKAGE_PATH
    at_archive_name=at_prefix+'/evidence/at-positive-canonical-v2.tar.gz'
    at_receipt_name=at_prefix+'/evidence/at-positive-canonical-v2.json'
    at_audit_name=at_prefix+'/evidence/AT_CANONICAL_AUDIT.json'
    at_archive=files[at_archive_name];at_receipt_bytes=files[at_receipt_name]
    at=json.loads(at_receipt_bytes)
    at_files=checked_tar_bytes(at_archive,AT_ARCHIVE_SHA,AT_ARCHIVE_MEMBERS,'embedded published AT canonical v2')
    assert sha(at_receipt_bytes)==AT_RECEIPT_SHA and at.get('status')=='proved'
    assert at.get('statistics')==dict(files=AT_TARGETS,prover=AT_PROVER_LEAVES,null=0,structural=0)
    at_audit=json.loads(files[at_audit_name])
    assert at_audit.get('result')=='pass' and at_audit.get('archive_sha256')==AT_ARCHIVE_SHA
    assert at_audit.get('archive_integrity',{}).get('all_coma_and_proof_hashes_match') is True
    assert at_audit.get('archive_integrity',{}).get('diagnostic') is False
    assert sha(at_files['probe/check_correspondence.py'])==AT_AUDIT_CHECKER_SHA
    assert sha(files[at_prefix+'/check_correspondence.py'])==AT_AUDIT_CHECKER_SHA
    assert len([n for n in files if n.startswith(at_prefix+'/')])>0
    for required in [
        at_prefix+'/check_correspondence.py',at_prefix+'/check_native.py',
        at_prefix+'/native-check-controls.py',at_prefix+'/check_checker_controls.py',
        at_prefix+'/generated/checker-controls-receipt.json',at_prefix+'/generated/native-check-controls.json',
        at_prefix+'/fixtures/checker-controls.json',at_prefix+'/fixtures/native-check-controls.json',
        at_prefix+'/evidence/AT_CANONICAL_AUDIT.md',at_prefix+'/evidence/AT_EXTERNAL_TOOL_HASH_CHECK.json',
        at_prefix+'/evidence/at-positive-canonical-v2-audit.json',
    ]:assert required in files,required
    assert correspondence.get('published_AT_ancestry',{}).get('canonical_archive_replayed') is True
    assert correspondence.get('published_AT_ancestry',{}).get('canonical_archive',{}).get('sha256')==AT_ARCHIVE_SHA
    assert correspondence.get('published_AT_ancestry',{}).get('published_AT_checker_sha256')==AT_AUDIT_CHECKER_SHA

    required_ancestry={
        'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/AS_CANONICAL_AUDIT.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/AS_CANONICAL_AUDIT.md',
        'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/as-positive-canonical-v1.tar.gz',
        'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/as-positive-canonical-v1.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-nonnull-view-boundaries-2026-10-09/evidence/as-positive-canonical-v1-audit.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09/evidence/AR_CANONICAL_AUDIT.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-shared-slice-views-2026-10-09/evidence/AQ_CANONICAL_AUDIT.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-shared-finite-owners-2026-10-09/evidence/AP_CANONICAL_AUDIT.json',
        'inputs/repository/bytes/1.11.1/verification/probes/original-promotable-surviving-child-2026-10-09/evidence',
    }
    # The published AT canonical archive is the authoritative embedding of earlier
    # probe evidence. Require its ancestry audit, which carries and checks AP/AQ/AR/AS.
    required_ancestry.discard('inputs/repository/bytes/1.11.1/verification/probes/original-promotable-surviving-child-2026-10-09/evidence')
    assert required_ancestry<=set(files)
    assert r['ancestry'].get('AT_archive_replayed_by_AU_checker') is True
    assert r['ancestry'].get('AT_full_source_and_checker_tree_embedded') is True

    report=dict(archive_sha256=r['archive_sha256'],members_verified=len(files),statistics=r['statistics'],
                status=r['status'],self_contained_toolchain=False,scope=r['scope'],
                proof_reuse=dict(origin_archive_sha256=AU_DIAGNOSTIC_ARCHIVE_SHA,
                                 origin_prover_process_exit_status='not-recorded-in-archive',
                                 completion_marker='Proved (157 files)',
                                 current_target_pairs_identical=True,prover_rerun_claimed=False),
                new_admission=dict(correspondence='pass',compiled_artifacts=4,structural_controls='1/1 rejected',
                                   correspondence_log_sha256=r['new_admission']['correspondence_log_sha256']),
                ancestry=dict(AT_archive_sha256=AT_ARCHIVE_SHA,embedded=True),
                correspondence='Current AU checker receipt and source bundle are archived; this audit validates their bindings and uses the separately replayed checker receipt for adequacy.')
    (R/'evidence'/(label+'-audit.json')).write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();s=p.add_subparsers(dest='action',required=True)
    c=s.add_parser('capture');c.add_argument('label');c.add_argument('log');c.add_argument('--status',choices=['admitted_reuse'],required=True);c.add_argument('--reuse-identity');c.add_argument('--preflight',action='store_true')
    a=s.add_parser('audit');a.add_argument('label');q=p.parse_args()
    if q.action=='capture':capture(q.label,q.log,q.status,q.reuse_identity,q.preflight)
    else:audit(q.label)
