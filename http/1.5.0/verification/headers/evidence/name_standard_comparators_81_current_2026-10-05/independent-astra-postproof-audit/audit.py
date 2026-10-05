#!/usr/bin/env python3
"""Independent solver-free audit. Writes only this auditor-owned directory."""
from pathlib import Path
import collections, concurrent.futures, gzip, hashlib, json, re, subprocess, tempfile

OUT = Path(__file__).resolve().parent
BASE = OUT.parent
REPO = BASE.parents[5]
WHY3 = '/workspace/proof-tools/creusot-data/bin/why3'
CONFIG = '/workspace/proof-tools/config/creusot/why3.conf'
LIB = '/workspace/proof-tools/creusot-data/share/why3find/packages/creusot'
REPLAY = Path('/tmp/uri-eq-replay-build/replay')
REPLAY_SOURCE = REPO / 'http/1.5.0/verification/method/evidence/current-source-reconciliation-20261005/independent-astra-nested/replay.ml'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def archive(path, data):
    target = OUT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.compress(data, mtime=0))
    return {'file': path, 'raw_bytes': len(data), 'raw_sha256': sha(data), 'gzip_sha256': sha(target.read_bytes())}

def tree(node, depth=0):
    if node is None:
        return {'null': 1, 'provers': collections.Counter(), 'nested': 0, 'max_time': 0}
    if 'prover' in node:
        return {'null': 0, 'provers': collections.Counter({node['prover']: 1}), 'nested': 0, 'max_time': node.get('time', 0)}
    children = node.get('children', [])
    result = {'null': 0, 'provers': collections.Counter(), 'nested': int(depth > 0 and bool(children)), 'max_time': 0}
    for child in children:
        sub = tree(child, depth + 1)
        result['null'] += sub['null']; result['nested'] += sub['nested']
        result['provers'].update(sub['provers']); result['max_time'] = max(result['max_time'], sub['max_time'])
    return result

def audit_target(item):
    group, entry = item
    coma = BASE / entry['coma_path']; proof = BASE / entry['proof_json_path']
    assert sha(coma.read_bytes()) == entry['coma_sha256']
    assert sha(proof.read_bytes()) == entry['proof_json_sha256']
    roots = json.loads(proof.read_text())['proofs']['Coma']
    owned = [name for name, node in roots.items() if node is None or 'prover' in node or node.get('children')]
    assert len(owned) == 1, (coma, owned)
    goal = owned[0]; stats = tree(roots[goal]); key = str(coma.relative_to(BASE / 'emission')).replace('/', '__').removesuffix('.coma')
    prefix = coma.stem + '-Coma-' + goal.replace("'", 'qt')
    with tempfile.TemporaryDirectory(prefix='astra-name-postproof-') as directory:
        command = [str(REPLAY), str(coma), goal, '-', directory]
        run = subprocess.run(command, capture_output=True)
        assert run.returncode == 0, (coma, run.stderr.decode())
        count = int(re.search(rb'depth=0 arity=(\d+)', run.stdout).group(1))
        assert count == len(roots[goal]['children']) == entry['solver_leaf_count']
        tasks = []
        for index in range(count):
            data = (Path(directory) / f'depth-0-child-{index}.why').read_bytes()
            saved = BASE / 'why3_tasks' / key / (prefix + (str(index) if index else '') + '.why')
            assert data == saved.read_bytes(), (coma, index)
            row = archive(f'contexts/{key}/child-{index}.why.gz', data)
            row.update({'child_index': index, 'saved_task': str(saved.relative_to(BASE)), 'byte_identical': True})
            tasks.append(row)
        root_context = archive(f'contexts/{key}/root.why.gz', (Path(directory) / 'root.why').read_bytes())
        archive(f'contexts/{key}/api.stdout.gz', run.stdout)
        archive(f'contexts/{key}/api.stderr.gz', run.stderr)
    printed = subprocess.run([WHY3, 'prove', '-C', CONFIG, '-L', LIB, '-a', 'split_vc', '-D', 'why3', str(coma)], capture_output=True)
    assert printed.returncode == 0
    assert len(re.findall(rb'^goal ', printed.stdout, re.M)) == count
    full_stdout = archive(f'contexts/{key}/full.stdout.gz', printed.stdout)
    archive(f'contexts/{key}/full.stderr.gz', printed.stderr)
    nested_replays = []
    def replay_nested(node, indices):
        if not isinstance(node, dict) or 'children' not in node:
            return
        if indices and node['children']:
            with tempfile.TemporaryDirectory(prefix='astra-name-postproof-nested-') as directory:
                run = subprocess.run([str(REPLAY), str(coma), goal, ','.join(map(str, indices)), directory], capture_output=True)
                assert run.returncode == 0
                arities = [int(value) for value in re.findall(rb'arity=(\d+)', run.stdout)]
                assert arities[-1] == len(node['children'])
                parent_key = '-'.join(map(str, indices))
                parent_data = (Path(directory) / f'parent-{len(indices)}.why').read_bytes()
                if len(indices) == 1:
                    assert sha(parent_data) == tasks[indices[0]]['raw_sha256']
                parent_context = archive(f'contexts/{key}/nested-{parent_key}/parent.why.gz', parent_data)
                children = [archive(f'contexts/{key}/nested-{parent_key}/child-{i}.why.gz', (Path(directory) / f'depth-{len(indices)}-child-{i}.why').read_bytes()) for i in range(arities[-1])]
                nested_replays.append({'parent_path': indices, 'arities': arities, 'parent_matches_saved_direct_task': len(indices) == 1, 'parent_context': parent_context, 'children': children})
                archive(f'contexts/{key}/nested-{parent_key}/api.stdout.gz', run.stdout)
                archive(f'contexts/{key}/nested-{parent_key}/api.stderr.gz', run.stderr)
        for index, child in enumerate(node['children']):
            replay_nested(child, indices + [index])
    replay_nested(roots[goal], [])
    return {'group': group, **entry, 'owned_root': goal, 'initial_arity': count, **stats,
            'zero_child_roots': [name for name, node in roots.items() if node == {'tactic': 'split_vc', 'children': []}],
            'root_context': root_context, 'full_stdout': full_stdout, 'tasks': tasks, 'nested_replays': nested_replays}

def verify_ledger(relative):
    path = BASE / relative; entries = [line.split(maxsplit=1) for line in path.read_text().splitlines()]
    for digest, filename in entries:
        assert sha((BASE / filename).read_bytes()) == digest, filename
    return {'file': relative, 'sha256': sha(path.read_bytes()), 'entries': len(entries), 'all_match': True}

def main():
    results_path = BASE / 'proof_results.json'; results = json.loads(results_path.read_text())
    inputs = [(group['group'], entry) for group in results['groups'] for entry in group['comas']]
    assert len(inputs) == len(set(entry['coma_path'] for _, entry in inputs)) == 87
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        audited = list(pool.map(audit_target, inputs))
    source = (BASE / 'sources/http/1.5.0/src/header/name.rs').read_text()
    spellings = re.findall(r'\((\w+),\s*(\w+),\s*(\d+),\s*b"([^"]*)",\s*"([^"]*)",\s*\[([^]]*)\],\s*(\w+)\);', source)
    assert len(spellings) == 81 and sorted(int(row[2]) for row in spellings) == list(range(81))
    assert len(set(row[6] for row in spellings)) == 81
    assert all(bytes(int(x) for x in re.findall(r'(\d+)u8', row[5])) == row[3].encode() == row[4].encode() for row in spellings)
    assert set(row[6] for row in spellings) == {Path(entry['coma_path']).stem for group, entry in inputs if group == '02_matchers_81'}
    assert 'if $matcher(name_bytes)' in source
    assert 'const fn $matcher(name_bytes: &[u8]) -> bool {\n            bytes_equal(name_bytes, &[$($name_byte),*])' in source
    assert 'ensures(result == (name_bytes@ == seq![$($name_byte),*]))' in source
    assert 'trusted' not in source
    totals = collections.Counter(); groups = []
    for group in results['groups']:
        selected = [row for row in audited if row['group'] == group['group']]
        provers = collections.Counter()
        for row in selected: provers.update(row['provers'])
        totals.update(provers)
        assert sum(provers.values()) == group['own_task_expected']
        log = BASE / group['proof_log_path']; command = BASE / group['command_file']
        assert sha(log.read_bytes()) == group['proof_log_sha256'] and len(log.read_bytes()) == group['proof_log_bytes']
        assert sha(command.read_bytes()) == group['command_sha256']
        clean_log = re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]', '', log.read_text())
        successes = [int(number) for number in re.findall(r'(?:Library|Theory) [^\r\n]*: ✔ \((\d+)\)', clean_log)]
        assert len(successes) == len(selected) and sum(successes) == sum(provers.values()), group['group']
        groups.append({'group': group['group'], 'targets': len(selected), 'own_leaves': sum(provers.values()), 'provers': provers, 'log_sha256': sha(log.read_bytes()), 'success_markers': successes})
    assert sum(row['initial_arity'] for row in audited) == sum(totals.values()) == 323
    assert not any(row['null'] for row in audited)
    assert sum(row['nested'] for row in audited) == 1
    (OUT / 'replay.ml').write_bytes(REPLAY_SOURCE.read_bytes())
    report = {'result': 'PASS', 'auditor': 'Astra', 'solver_invoked': False, 'frontend_invoked': False, 'source_modified': False,
              'source_sha256': sha(source.encode()), 'proof_results_sha256': sha(results_path.read_bytes()),
              'method': 'Original COMA Why3 API: select exact owned root, split_vc once; compare ordered child contexts including unsuffixed child 0. Also print complete raw split_vc stdout without normalization.',
              'replay_binary_sha256': sha(REPLAY.read_bytes()), 'replay_source_sha256': sha(REPLAY_SOURCE.read_bytes()),
              'ledgers': [verify_ledger(path) for path in ['sources.sha256', 'emission/comas.sha256', 'why3_tasks.sha256']],
              'selected_targets': 87, 'own_terminal_leaves': 323, 'null_leaves': 0, 'nested_nodes': sum(row['nested'] for row in audited),
              'provers': totals, 'max_leaf_seconds': max(row['max_time'] for row in audited),
              'zero_child_marker_occurrences': sum(len(row['zero_child_roots']) for row in audited),
              'numeric_spellings_checked': 81, 'all_spelling_forms_agree': True, 'all_parser_routes_use_actual_checked_wrappers': True,
              'groups': groups, 'targets': audited,
              'qualification': 'This proves the 87 selected program/helper roots under their recorded callee and existing Creusot contracts. Zero-child callee/translator roots are excluded. Remaining emitted bodies, including parse_hdr, are not newly proved by this batch. Bytes is an explicitly accepted dependency. No HTTP trusted setter or axiom was added.'}
    (OUT / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ['result','selected_targets','own_terminal_leaves','provers','max_leaf_seconds','zero_child_marker_occurrences']}))
    print('report_sha256=' + sha((OUT / 'report.json').read_bytes()))

if __name__ == '__main__':
    main()
