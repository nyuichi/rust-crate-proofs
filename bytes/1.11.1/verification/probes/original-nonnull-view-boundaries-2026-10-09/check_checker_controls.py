#!/usr/bin/env python3
"""AS in-memory controls for exact source, native and Cargo correspondence."""
from __future__ import annotations
import copy, hashlib, importlib.util, json, pathlib, sys
import argparse
from typing import Any, Callable
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parent
CHECKER = ROOT / 'check_correspondence.py'
MANIFEST_PATH = ROOT / 'fixtures/checker-controls.json'
DEFAULT_RECEIPT = ROOT / 'generated/checker-controls-receipt.json'
spec = importlib.util.spec_from_file_location('as_controls_draft_target', CHECKER)
assert spec and spec.loader
C = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = C
spec.loader.exec_module(C)

def sha(data: bytes) -> str: return hashlib.sha256(data).hexdigest()
def replace_once(data: bytes, old: bytes, new: bytes, label: str) -> bytes:
    if data.count(old) != 1: raise RuntimeError(f'{label}: expected one anchor, got {data.count(old)}')
    return data.replace(old, new, 1)

def expect_reject(identifier: str, action: Callable[[], Any], expected: str | None = None) -> dict[str, str]:
    try: result = action()
    except C.CheckError as exc:
        reason = str(exc)
        if expected and expected not in reason:
            raise RuntimeError(f'{identifier}: rejected for unexpected reason: {reason}') from exc
        return {'id': identifier, 'status': 'rejected_as_expected', 'reason': reason}
    except Exception as exc:
        raise RuntimeError(f'{identifier}: checker crashed: {type(exc).__name__}: {exc}') from exc
    raise RuntimeError(f'{identifier}: accepted mutation: {result!r}')

def full_audit_reject(expected: str) -> None:
    try:
        result = C.audit()
    except C.CheckError as exc:
        reason = str(exc)
        if expected not in reason:
            raise RuntimeError(f'full audit rejected for an unrelated reason, wanted {expected!r}: {reason}') from exc
        raise
    if result.get('status') == 'reject' and expected in str(result.get('reason', '')):
        raise C.CheckError(str(result['reason']))
    raise RuntimeError(f'full audit failed to reject for {expected!r}: {result!r}')

def run() -> dict[str, Any]:
    fixture = json.loads(MANIFEST_PATH.read_text())
    fixture_ids = [row.get('id') for row in fixture.get('controls', [])]
    if fixture.get('version') != 1 or fixture_ids != AS_IDS or len(set(fixture_ids)) != len(fixture_ids):
        raise RuntimeError('AS control fixture does not exactly match the implemented 36 controls')
    baseline = C.audit()
    if baseline.get('status') != 'pass': raise RuntimeError(f'AS complete baseline failed: {baseline!r}')
    ar = C._import_ar_checker()
    tree = C._regular_tree(C.ROOT / 'src', 'AS')
    mapping = json.loads((C.ROOT / 'generated/mapping.json').read_text())
    active = (C.ROOT / 'generated/active.rs').read_bytes()
    positive = (C.ROOT / 'generated/positive.rs').read_bytes()

    def source_check(changed: dict[str, bytes], *, map_override=mapping, active_override=active,
                     positive_override=positive) -> Any:
        return C.assert_as_transformations(ar, changed, map_override, active_override, positive_override)

    actions: dict[str, Callable[[], Any]] = {}
    changed = dict(tree)
    changed['promotion.rs'] = replace_once(changed['promotion.rs'], C.VIEW_NEW.encode(), C.VIEW_OLD.encode(), 'view nonnull')
    actions['view_global_nonnull_removed'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    weakened_view = C.VIEW_NEW.replace('!self.ptr.is_null_logic() &&', 'self.ptr.is_null_logic() &&', 1)
    changed['promotion.rs'] = replace_once(changed['promotion.rs'], C.VIEW_NEW.encode(), weakened_view.encode(), 'view nonnull weakened')
    actions['view_global_nonnull_weakened'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['physical_projection.rs'] = replace_once(changed['physical_projection.rs'], C.EMPTY_NEW.encode(), C.EMPTY_OLD.encode(), 'empty nonnull')
    actions['empty_boundary_nonnull_removed'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['view_pointer.rs'] = replace_once(changed['view_pointer.rs'], C.WRAP_NEW.encode(), C.WRAP_OLD.encode(), 'wrapping nonnull')
    actions['wrapping_boundary_nonnull_removed'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['physical_projection.rs'] = replace_once(changed['physical_projection.rs'],
        b'core::slice::from_raw_parts(pointer,0)', b'core::slice::from_raw_parts(core::ptr::null(),0)',
        'empty boundary native body')
    actions['empty_boundary_native_body_changed'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['view_pointer.rs'] = replace_once(changed['view_pointer.rs'],
        b'pointer.wrapping_add(count)', b'pointer.wrapping_add(0)', 'wrapping boundary native body')
    actions['wrapping_boundary_native_body_changed'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['promotion.rs'] += b'\n#[trusted] fn hidden_runtime_effect() {}\n'
    actions['promotion_added_hidden_runtime'] = lambda x=changed: source_check(x)

    changed = dict(tree); changed['unrouted.rs'] = b'#[trusted] fn hidden() {}\n'
    actions['source_extra_unrouted_module'] = lambda x=changed: source_check(x)

    changed = dict(tree)
    changed['lib.rs'] = changed['lib.rs'].replace(b'#[cfg(creusot)] mod view_pointer;', b'#[cfg(creusot)] #[path="../../evil.rs"] mod view_pointer;', 1)
    actions['lib_module_route_redirected'] = lambda x=changed: source_check(x)

    def mutate_mapping(identifier: str, path: tuple[Any, ...], value: Any, expected: str = 'AS mapping') -> None:
        altered = copy.deepcopy(mapping); node: Any = altered
        for part in path[:-1]: node = node[part]
        node[path[-1]] = value
        actions[identifier] = lambda m=altered: source_check(dict(tree), map_override=m)
    mutate_mapping('mapping_ancestor_redirected', ('source_transformations', 0, 'ancestor_source'), '../unreviewed/positive.rs')
    mutate_mapping('mapping_ancestor_digest_changed', ('source_transformations', 0, 'ancestor_sha256'), '0'*64)
    mutate_mapping('mapping_transform_literal_changed', ('source_transformations', 0, 'new'), 'unreviewed')
    mutate_mapping('mapping_transform_count_changed', ('source_transformations', 0, 'replacements'), 2)
    mutate_mapping('mapping_support_inventory_changed', ('support_inventory', 'src/promotion.rs'), '0'*64)
    mutate_mapping('mapping_native_edge_changed', ('normal_edges', 0, 'owner'), 'wrong-owner', 'inherited AR')
    mutate_mapping('mapping_callback_changed', ('callbacks', 0), 'wrong-callback', 'inherited AR')
    mutate_mapping('mapping_active_digest_changed', ('active_sha256',), '0'*64)
    changed = copy.deepcopy(mapping); changed['source_transformations'].append(copy.deepcopy(changed['source_transformations'][0]))
    actions['mapping_extra_transform_added'] = lambda m=changed: source_check(dict(tree), map_override=m)

    # Manifest validator accepts optional source overrides in the pending patch.
    manifest = (C.ROOT/'Cargo.toml').read_text(); lock = (C.ROOT/'Cargo.lock').read_bytes()
    build = (C.ROOT/'build.rs').read_bytes(); extractor = (C.ROOT/'extract_public.py').read_bytes()
    generator = (C.ROOT/'elaborate.py').read_bytes(); launcher = (C.ROOT/'run-proof.sh').read_bytes()
    for identifier, replacement in (
        ('cargo_patch_redirect', manifest+'\n[patch.crates-io]\ncreusot-std={path="../../evil"}\n'),
        ('cargo_default_negative_feature', manifest+'\ndefault=["negative_missing_acquire"]\n'),
        ('cargo_generator_control_as_feature', manifest+'\nomit_view_nonnull=[]\n'),
        ('cargo_missing_declared_feature', manifest.replace('negative_missing_control_free = []\n','')),
    ):
        actions[identifier] = lambda m=replacement: C.assert_probe_manifest(
            environment={}, manifest_text=m, lock_bytes=lock, build_bytes=build,
            extractor_bytes=extractor, generator_bytes=generator, launcher_bytes=launcher)
    for identifier, env in (
        ('env_generator_feature_admission', {'BYTES_DROP_FEATURE':'omit_view_nonnull'}),
        ('env_source_control_admission', {'BYTES_SCOPE_SOURCE_CONTROL':'omit_root_recovery_publication'}),
        ('env_diagnostic_admission', {'BYTES_SCOPE_DIAGNOSTIC':'1'}),
        ('env_translate_only_admission', {'BYTES_TRANSLATE_ONLY':'1'}),
    ):
        actions[identifier] = lambda e=env: C.assert_probe_manifest(environment=e)

    changed = copy.deepcopy(mapping); changed['normal_edges'][0]['owner'] = 'wrong-owner'
    actions['native_mapping_client_edge_changed'] = lambda m=changed: C.audit_native(m)
    changed = copy.deepcopy(mapping); changed['native_mir_ready'] = False
    actions['native_mapping_capture_ready_removed'] = lambda m=changed: C.audit_native(m)

    # Build inputs are taken from an already captured positive AS Cargo build.
    expected, input_hashes = C._expected_public_records(ar)
    artifacts, receipt_bytes = C._read_captured_artifacts()
    receipt = json.loads(receipt_bytes)
    source_map_bytes = (C.ROOT/'generated/source-map.json').read_bytes()
    def compiled_check(r: dict[str, Any], a: dict[str, bytes]) -> Any:
        b = (json.dumps(r, indent=2)+'\n').encode()
        return C.assert_compiled_capture(r, a, b, expected, source_map_bytes, input_hashes)

    changed_r = copy.deepcopy(receipt); changed_a = dict(artifacts)
    detached='/tmp/as-detached/out'; changed_r.update(actual_out_dir=detached, compiled_input_path=detached+'/public_records.rs', root_output_path='/tmp/as-detached/root-output')
    changed_a['cargo-root-output.txt'] = detached.encode()
    for k, v in [('root_output_sha256', changed_a['cargo-root-output.txt']), ('captured_root_output_sha256', changed_a['cargo-root-output.txt'])]: changed_r[k]=sha(v)
    actions['compiled_out_dir_detached_fresh_hashes'] = lambda r=changed_r,a=changed_a: compiled_check(r,a)

    changed_r=copy.deepcopy(receipt); changed_a=dict(artifacts); fp=json.loads(changed_a['cargo-run-build-fingerprint.json'])
    fp['local']=[dict(row,RerunIfChanged=dict(row['RerunIfChanged'],output='debug/build/unreviewed/output')) if isinstance(row,dict) and isinstance(row.get('RerunIfChanged'),dict) else row for row in fp.get('local',[])]
    changed_a['cargo-run-build-fingerprint.json']=(json.dumps(fp,separators=(',',':'))+'\n').encode(); h=sha(changed_a['cargo-run-build-fingerprint.json'])
    changed_r.update(cargo_build_fingerprint_sha256=h,captured_cargo_build_fingerprint_sha256=h)
    actions['compiled_fingerprint_output_redirect_fresh_hashes'] = lambda r=changed_r,a=changed_a: compiled_check(r,a)

    changed_r=copy.deepcopy(receipt); changed_a=dict(artifacts); fp=json.loads(changed_a['cargo-run-build-fingerprint.json'])
    fp['local']=[dict(row,RerunIfChanged=dict(row['RerunIfChanged'],paths=['../../evil.rs']+row['RerunIfChanged'].get('paths',[])[1:])) if isinstance(row,dict) and isinstance(row.get('RerunIfChanged'),dict) else row for row in fp.get('local',[])]
    changed_a['cargo-run-build-fingerprint.json']=(json.dumps(fp,separators=(',',':'))+'\n').encode(); h=sha(changed_a['cargo-run-build-fingerprint.json'])
    changed_r.update(cargo_build_fingerprint_sha256=h,captured_cargo_build_fingerprint_sha256=h)
    actions['compiled_fingerprint_source_redirect_fresh_hashes'] = lambda r=changed_r,a=changed_a: compiled_check(r,a)

    changed_r=copy.deepcopy(receipt); changed_a=dict(artifacts); changed_a['public_records.rs']+=b'\n// forged\n'; h=sha(changed_a['public_records.rs'])
    changed_r.update(compiled_input_sha256=h,reconstructed_generated_sha256=h,captured_input_sha256=h)
    actions['compiled_record_forged_fresh_hashes'] = lambda r=changed_r,a=changed_a: compiled_check(r,a)

    changed_r=copy.deepcopy(receipt); changed_a=dict(artifacts); changed_a['cargo-root-output.txt']=b'/tmp/evil/out'; h=sha(changed_a['cargo-root-output.txt'])
    changed_r.update(root_output_sha256=h,captured_root_output_sha256=h)
    actions['compiled_root_output_forged_fresh_hashes'] = lambda r=changed_r,a=changed_a: compiled_check(r,a)

    # Verify the complete audit, not only helpers, reaches the three sensitive joins.
    original_tree_reader=C._regular_tree
    altered=dict(tree); altered['physical_projection.rs']=replace_once(altered['physical_projection.rs'],C.EMPTY_NEW.encode(),C.EMPTY_OLD.encode(),'full audit')
    def full_source():
        def reader(directory: pathlib.Path, label: str):
            if directory.resolve()==(C.ROOT/'src').resolve() and label=='AS': return altered
            return original_tree_reader(directory,label)
        with mock.patch.object(C,'_regular_tree',reader): full_audit_reject('AS source differs from the exact reviewed transformation')
    actions['full_audit_source_transform_mutation']=full_source

    original_manifest=C.assert_probe_manifest
    def full_manifest():
        def bad_manifest(*args,**kwargs):
            return original_manifest(environment={},manifest_text=manifest+'\n[patch.crates-io]\ncreusot-std={path="../../evil"}\n',
                lock_bytes=lock,build_bytes=build,extractor_bytes=extractor,generator_bytes=generator,launcher_bytes=launcher)
        with mock.patch.object(C,'assert_probe_manifest',bad_manifest): full_audit_reject('AS Cargo package, feature/build route')
    actions['full_audit_manifest_redirect']=full_manifest

    original_compiled=C.assert_compiled_capture
    def full_compiled():
        def bad_capture(r,a,s,e,sm,ih):
            forged=copy.deepcopy(r); changed_artifacts=dict(a); target='/tmp/as-full-audit/out'
            forged.update(actual_out_dir=target,compiled_input_path=target+'/public_records.rs',root_output_path='/tmp/as-full-audit/root-output')
            changed_artifacts['cargo-root-output.txt']=target.encode(); h=sha(changed_artifacts['cargo-root-output.txt'])
            forged.update(root_output_sha256=h,captured_root_output_sha256=h)
            return original_compiled(forged,changed_artifacts,(json.dumps(forged,indent=2)+'\n').encode(),e,sm,ih)
        with mock.patch.object(C,'assert_compiled_capture',bad_capture): full_audit_reject('AS captured Cargo root-output/OUT_DIR/public_records paths do not resolve consistently')
    actions['full_audit_compiled_join_mutation']=full_compiled

    if set(actions) != set(AS_IDS): raise RuntimeError(f'control implementation/fixture mismatch: missing={set(AS_IDS)-set(actions)}, extra={set(actions)-set(AS_IDS)}')
    rows=[expect_reject(identifier,actions[identifier]) for identifier in AS_IDS]
    return {'status':'pass','baseline_status':'pass','control_count':len(rows),
        'rejected_as_expected':len(rows),'controls':rows,'mutations_in_memory_only':True,
        'cargo_or_rust_build_invoked':False,'solver_invoked':False,
        'checker_sha256':sha(CHECKER.read_bytes()),'fixture_sha256':sha(MANIFEST_PATH.read_bytes()),
        'baseline_scope':'AS exact source transformation, native correspondence, AR ancestry and four-artifact Cargo capture'}

AS_IDS = [
 'view_global_nonnull_removed','view_global_nonnull_weakened','empty_boundary_nonnull_removed',
 'wrapping_boundary_nonnull_removed','empty_boundary_native_body_changed','wrapping_boundary_native_body_changed',
 'promotion_added_hidden_runtime','source_extra_unrouted_module','lib_module_route_redirected',
 'mapping_ancestor_redirected','mapping_ancestor_digest_changed','mapping_transform_literal_changed',
 'mapping_transform_count_changed','mapping_support_inventory_changed','mapping_native_edge_changed',
 'mapping_callback_changed','mapping_active_digest_changed','mapping_extra_transform_added',
 'cargo_patch_redirect','cargo_default_negative_feature','cargo_generator_control_as_feature',
 'cargo_missing_declared_feature','env_generator_feature_admission','env_source_control_admission',
 'env_diagnostic_admission','env_translate_only_admission','native_mapping_client_edge_changed',
 'native_mapping_capture_ready_removed','compiled_out_dir_detached_fresh_hashes',
 'compiled_fingerprint_output_redirect_fresh_hashes','compiled_fingerprint_source_redirect_fresh_hashes',
 'compiled_record_forged_fresh_hashes','compiled_root_output_forged_fresh_hashes',
 'full_audit_source_transform_mutation','full_audit_manifest_redirect','full_audit_compiled_join_mutation']

if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=pathlib.Path,default=DEFAULT_RECEIPT);args=parser.parse_args()
    result=run(); rendered=json.dumps(result,indent=2)+'\n'; args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(rendered);print(rendered,end='')
