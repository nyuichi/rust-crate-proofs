"""Exercise the real binary against local Git + an in-memory proofs.rs API."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

binary = str(Path(sys.argv[1]).resolve())
reports = {}
history = {}
keys = {}
next_claim = 0
polls = 0
lose_reply = False
race = False


def normalize(body):
    result = copy.deepcopy(body)
    for field in ['title', 'explanation', 'trusted_assumptions', 'environment', 'evidence_url', 'limitations']:
        result.setdefault(field, '')
    for claim in result['claims']:
        claim.setdefault('id', None)
        for field in ['title', 'explanation', 'trusted_assumptions', 'limitations', 'evidence_url']:
            claim.setdefault(field, '')
        if not claim['title']:
            claim['title'] = f"Verification of {claim['api_item_id']}"
    return result


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def reply(self, status, data):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def do_GET(self):
        assert self.headers.get('Authorization') == 'Bearer fixture-token'
        if self.path == '/api/v1/me':
            return self.reply(200, {'user': {'id': 'author'}})
        if self.path == '/api/v1/tools':
            return self.reply(200, {'items': [{'id': 'kani', 'name': 'Kani', 'active': 1}, {'id': 'creusot', 'name': 'Creusot', 'active': 1}, {'id': 'aeneas', 'name': 'Aeneas', 'active': 1}], 'versions': [{'id': 'kani-version', 'tool_id': 'kani', 'version': '0.66.0', 'selectable': 1}, {'id': 'creusot-version', 'tool_id': 'creusot', 'version': '0.9.0', 'selectable': 1}, {'id': 'aeneas-version', 'tool_id': 'aeneas', 'version': 'a' * 40, 'selectable': 1}]})
        if self.path.startswith('/api/v1/resolve-api?'):
            from urllib.parse import parse_qs, urlparse
            path = parse_qs(urlparse(self.path).query)['path'][0].removeprefix('fixture::')
            if path not in ['f', 'g']:
                return self.reply(404, {'error': 'api_not_found'})
            return self.reply(200, {'id': 'api-' + path})
        if self.path.startswith('/api/v1/reports/'):
            parts = self.path.split('/')
            report_id = int(parts[4])
            result = history[(report_id, int(parts[6]))] if len(parts) > 5 else reports[report_id]
            return self.reply(200, result)
        raise AssertionError(self.path)

    def do_POST(self):
        global next_claim, polls, lose_reply, race
        raw = self.rfile.read(int(self.headers['Content-Length']))
        if self.path.endswith('/sarif'):
            assert self.headers.get('Authorization') == 'Bearer fixture-token'
            sarif = json.loads(raw)['runs'][0]
            assert sarif['properties']['proofs']['contracts']
            assert sarif['properties']['proofs']['schemaVersion'] == 2
            assert isinstance(sarif['properties']['proofs']['dependencies'], list)
            assert sarif['automationDetails']['guid'] in self.path
            assert sarif['versionControlProvenance'][0]['revisionId']
            assert sarif['artifacts'][sarif['invocations'][0]['stdout']['index']]['contents']['text']
            assert not any(a.get('contents') for a in sarif['artifacts'][:-2])
            return self.reply(201, {'ok': True})
        body = json.loads(raw)
        assert not self.path.startswith('/api/v1/runs/'), 'Run registration must be one SARIF upload'
        if self.path == '/auth/device/code':
            assert body == {'client_id': 'proofs-cli', 'scope': 'publish'}
            return self.reply(200, {'device_code': 'fixture-device', 'user_code': 'ABCD-EFGH', 'verification_uri': origin + '/device', 'expires_in': 60, 'interval': 1})
        if self.path == '/auth/device/token':
            polls += 1
            assert body['device_code'] == 'fixture-device'
            if polls == 1:
                return self.reply(400, {'error': 'authorization_pending'})
            return self.reply(200, {'access_token': 'fixture-token', 'expires_in': 3600})
        assert self.headers.get('Authorization') == 'Bearer fixture-token'
        if self.path == '/api/v1/tokens/revoke':
            return self.reply(200, {'ok': True})
        if self.path == '/api/v1/publish/prepare':
            return self.reply(200, {'status': 'ready'})
        if self.path == '/api/v1/reports/validate':
            assert body['title']
            assert 1 <= len(body['claims']) <= 100
            if race:
                race = False
                web_edit()
            return self.reply(200, normalize(body))
        key = self.headers['Idempotency-Key']
        if key in keys:
            assert keys[key][0] == body, 'idempotency key reused with changed payload'
            return self.reply(201, keys[key][1])
        if self.path == '/api/v1/reports':
            report_id, revision = len(reports) + 1, 1
        else:
            report_id = int(self.path.split('/')[4])
            if body['expected_revision'] != reports[report_id]['revision_no']:
                return self.reply(409, {'error': 'conflict'})
            revision = body['expected_revision'] + 1
        result = normalize(body)
        for claim in result['claims']:
            if not claim['id']:
                next_claim += 1
                claim['id'] = f'claim-{next_claim}'
        result.update(id=report_id, revision_no=revision, author_id='author', withdrawn_at=None)
        reports[report_id] = result
        history[report_id, revision] = copy.deepcopy(result)
        response = {'id': report_id, 'revision_no': revision}
        keys[key] = (body, response)
        if lose_reply:
            lose_reply = False
            return self.reply(500, {'error': 'simulated_lost_reply_after_commit'})
        self.reply(201, response)


def web_edit():
    report = reports[1]
    report['title'] = 'Web title'
    report['claims'][0]['title'] = 'Preserve my claim title'
    report['revision_no'] += 1
    history[1, report['revision_no']] = copy.deepcopy(report)


with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    work = root / 'work'
    work.mkdir()
    real_git = shutil.which('git')

    def git(*args):
        return subprocess.run([real_git, *args], cwd=work, check=True, capture_output=True, text=True).stdout.strip()

    git('init', '-q')
    git('config', 'user.name', 'Test')
    git('config', 'user.email', 'test@example.test')
    git('init', '--bare', '-q', str(root / 'remote.git'))
    git('remote', 'add', 'origin', str(root / 'remote.git'))
    (work / 'src').mkdir()
    (work / 'Cargo.toml').write_text('[package]\nname="fixture"\nversion="1.0.0"\nedition="2021"\n')
    # Preserve Cargo metadata's lock file behavior without making it an unrelated dirty file.
    (work / '.gitignore').write_text('/target/\n/Cargo.lock\n')
    source = '#[cfg_attr(kani, kani::requires(x > 0))]\npub fn f(x:u8) {}\n#[cfg(kani)]\n#[kani::proof_for_contract(f)]\nfn check_f() {}\n'
    extra = 'pub fn g() {}\n#[cfg(kani)]\n#[kani::proof_for_contract(g)]\nfn check_g() {}\n'
    (work / 'src/lib.rs').write_text(source)
    wrapper = root / 'bin'
    wrapper.mkdir()
    script = wrapper / 'git'
    # Only fake GitHub's advertised URL. Fetch/ancestry checks use the real local remote.
    script.write_text('#!/bin/sh\nif [ "$1" = remote ] && [ "$2" = get-url ]; then\n echo https://github.com/fixture/source\nelse\n exec "$TEST_REAL_GIT" "$@"\nfi\n')
    script.chmod(0o755)
    env = dict(os.environ, PATH=str(wrapper) + os.pathsep + os.environ['PATH'], TEST_REAL_GIT=real_git, PROOFS_CONFIG_DIR=str(root / 'credentials'))
    env['XDG_DATA_HOME'] = str(root / 'data')
    # Exercise aliased temporary roots on every Unix runner, including Linux.
    # macOS normally uses /var paths whose canonical form starts /private/var.
    (root / 'real-tmp').mkdir()
    (root / 'alias-tmp').symlink_to(root / 'real-tmp', target_is_directory=True)
    env['TMPDIR'] = str(root / 'alias-tmp')
    tool = wrapper / 'cargo-kani'
    tool.write_text(r"""#!/usr/bin/env python3
import sys,re,pathlib,os
if '--version' in sys.argv:
    print('kani 0.66.0');sys.exit(0)
source=pathlib.Path('src/lib.rs').read_text()
if os.getenv('TEST_MUTATE_SOURCE'):
    pathlib.Path('src/lib.rs').write_text(source+'// changed input\n')
for harness in re.findall(r'fn (check_\w+)\(',source):
    print('Checking harness fixture::'+harness+'...')
    print('RESULTS:')
    print('Check 1: f.pointer.1')
    print(' - Status: SUCCESS')
    print(' - Description: "pointer check"')
    print('VERIFICATION:- SUCCESSFUL')
""")
    tool.chmod(0o755)
    creusot = wrapper / 'cargo-creusot'
    creusot.write_text("""#!/usr/bin/env python3
import sys,pathlib,json
if 'version' in sys.argv:
    print('creusot 0.9.0');sys.exit(0)
for name in ['f','g']:
    p=pathlib.Path('verif/fixture_rlib')/name/'proof.json'
    p.parent.mkdir(parents=True,exist_ok=True)
    p.write_text(json.dumps({'proofs':{'M':{'vc_'+name:{'prover':'z3','time':0.1}}}}))
print('Proved (2 files)')
""")
    creusot.chmod(0o755)
    lake = wrapper / 'lake'
    lake.write_text("""#!/usr/bin/env python3
import sys,os
if sys.argv[1:2] == ['build']:
    assert '+Proofs' in sys.argv
    if os.getenv('TEST_LAKE_FAIL'): sys.exit(1)
    print('Built Proofs')
else:
    assert sys.argv[1:] == ['env', 'lean', '--stdin']
    source = sys.stdin.read()
    assert 'import Proofs' in source
    assert 'Lean.collectAxioms' in source and 'some (.thmInfo _)' in source
    if os.getenv('TEST_LEAN_FAIL'):
        print('Unapproved axiom in fixture.f_spec: sorryAx');sys.exit(1)
    print('proofs.rs:checked:fixture.f_spec')
""")
    lake.chmod(0o755)
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    origin = f'http://127.0.0.1:{server.server_port}'
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    def cli(*args, ok=True):
        output = subprocess.run([binary, 'proofs', '--server', origin, *args], cwd=work, env=env, input='', capture_output=True, text=True, timeout=30)
        assert (output.returncode == 0) == ok, (args, output.stdout, output.stderr)
        return output.stdout + output.stderr

    def commit_push():
        subprocess.run(['cargo', 'metadata', '--format-version', '1'], cwd=work, env=env, check=True, capture_output=True)
        git('add', 'src/lib.rs', 'Cargo.toml', '.gitignore', 'proofs.toml')
        git('add', '-f', 'Cargo.lock')
        git('commit', '-qm', 'fixture change')
        git('push', '-q', 'origin', 'HEAD:refs/heads/main')

    try:
        cli('init', '--title', 'My report', '--tool-version', '0.66.0')
        cli('login', '--no-browser')
        git('add', 'src/lib.rs', 'Cargo.toml', '.gitignore')
        git('commit', '-qm', 'fixture')
        assert 'No recorded run' in cli('publish', ok=False)
        assert 'Commit and push' in cli('run', '--', 'cargo', 'kani', ok=False)
        commit_push()
        cli('run', '--', 'cargo', 'kani')
        assert len(git('worktree', 'list', '--porcelain').split('worktree ')) == 2
        env['TEST_MUTATE_SOURCE'] = '1'
        assert 'Tracked inputs changed' in cli('run', '--', 'cargo', 'kani', ok=False)
        assert (work / 'src/lib.rs').read_text() == source
        assert len(git('worktree', 'list', '--porcelain').split('worktree ')) == 2
        del env['TEST_MUTATE_SOURCE']
        cli('run', '--', 'cargo', 'kani')
        for recorded in (root / 'data').rglob('run.sarif.json'):
            assert sorted(p.name for p in recorded.parent.iterdir()) == ['run.sarif.json']
        # Nothing generated by run requires a push.
        cli('publish', '--dry-run')
        assert not reports
        cli('publish')
        assert len(reports[1]['claims']) == 1
        original_ids = [c['id'] for c in reports[1]['claims']]
        assert all(c['precondition'] == '(x > 0)' for c in reports[1]['claims'])
        assert all(c['evidence_url'].startswith('https://github.com/fixture/source/blob/') and '/src/lib.rs#L' in c['evidence_url'] for c in reports[1]['claims'])
        assert 'No changes' in cli('publish')
        assert reports[1]['revision_no'] == 1
        web_edit()
        assert 'conflict' in cli('publish', ok=False)
        cli('publish', '--force')
        assert [c['id'] for c in reports[1]['claims']] == original_ids
        assert reports[1]['claims'][0]['title'] == 'Preserve my claim title'
        (work / 'src/lib.rs').write_text(source + extra)
        assert 'No changes' in cli('publish')  # Still publishes the frozen recorded source.
        commit_push()
        cli('run', '--', 'cargo', 'kani')
        cli('publish')
        added_ids = [c['id'] for c in reports[1]['claims'][1:]]
        assert len(reports[1]['claims']) == 2
        (work / 'src/lib.rs').write_text(source)
        commit_push()
        cli('run', '--', 'cargo', 'kani')
        assert 'Deletion requires confirmation' in cli('publish', ok=False)
        cli('publish', '--yes')
        assert len(reports[1]['claims']) == 1
        (work / 'src/lib.rs').write_text(source + extra)
        commit_push()
        cli('run', '--', 'cargo', 'kani')
        cli('publish')
        assert [c['id'] for c in reports[1]['claims'][1:]] == added_ids
        (work / 'proofs.toml').write_text((work / 'proofs.toml').read_text().replace('My report', 'Changed report'))
        race = True
        assert '409' in cli('publish', ok=False)
        # Definite conflict cleared its pending journal; a fresh forced publish works.
        cli('publish', '--force')
        (work / 'proofs.toml').write_text((work / 'proofs.toml').read_text().replace('Changed report', 'Retry report'))
        lose_reply = True
        assert '--resume' in cli('publish', ok=False)
        revision = reports[1]['revision_no']
        assert 'interrupted publication' in cli('publish', ok=False)
        cli('publish', '--resume')
        assert reports[1]['revision_no'] == revision
        assert 'No changes' in cli('publish')
        # A new crate version starts a separate report using the Creusot adapter.
        manifest = work / 'Cargo.toml'
        manifest.write_text(manifest.read_text().replace('1.0.0', '1.0.1'))
        (work / 'proofs.toml').unlink()
        cli('init', '--tool', 'creusot', '--tool-target', 'annotated', '--tool-version', '0.9.0')
        (work / 'src/lib.rs').write_text(
            '#[cfg_attr(creusot, requires(x@ > 0))]\npub fn f(x: u32) {}\n'
            'pub fn g() {}\n#[trusted] pub fn ignored() {}\n'
            '#[logic] pub fn model(x: u32) -> Int { x@ }\n')
        commit_push()
        cli('run', '--', 'cargo', 'creusot', 'prove')
        cli('publish', '--dry-run')
        assert len(reports) == 1
        cli('publish')
        assert reports[2]['tool_version_id'] == 'creusot-version'
        assert len(reports[2]['claims']) == 1
        claim = reports[2]['claims'][0]
        assert claim['precondition'] == '(x@ > 0)'
        assert '/blob/' in claim['evidence_url'] and '/src/lib.rs#L' in claim['evidence_url']
        assert 'No changes' in cli('publish')
        config = work / 'proofs.toml'
        config.write_text(config.read_text().replace('target = "annotated"', 'target = "all"'))
        commit_push()
        cli('run', '--', 'cargo', 'creusot', 'prove')
        cli('publish')
        assert len(reports[2]['claims']) == 2
        assert reports[2]['claims'][1]['precondition'] == 'true'
        config.write_text(config.read_text().replace('target = "all"', 'target = "annotated"'))
        commit_push()
        cli('run', '--', 'cargo', 'creusot', 'prove')
        assert 'Deletion requires confirmation' in cli('publish', ok=False)
        cli('publish', '--yes')
        assert reports[2]['claims'][0]['id'] == claim['id']
        # Aeneas uses committed Lean evidence and explicit API/theorem mappings.
        manifest.write_text(manifest.read_text().replace('1.0.1', '1.0.2'))
        config.unlink()
        cli('init', '--tool', 'aeneas', '--tool-version', 'a' * 40)
        config.write_text(config.read_text() + "\n[[tool.proofs]]\napi='f'\nmodule='Proofs'\ntheorem='fixture.f_spec'\nprecondition='x > 0'\n")
        (work / 'src/lib.rs').write_text('pub fn f(x: u32) {}\n')
        (work / 'Proofs.lean').write_text('namespace fixture\ntheorem f_spec : True := by trivial\nend fixture\n')
        (work / 'lean-toolchain').write_text('leanprover/lean4:v4.35.0\n')
        (work / 'lakefile.toml').write_text('name="fixture"\n[[lean_lib]]\nname="Proofs"\n')
        (work / 'lake-manifest.json').write_text(json.dumps({'packages': [{
            'name': 'aeneas', 'type': 'git', 'url': 'https://github.com/AeneasVerif/aeneas',
            'rev': 'a' * 40, 'subDir': 'backends/lean'}]}))
        git('add', 'Proofs.lean', 'lean-toolchain', 'lakefile.toml', 'lake-manifest.json')
        commit_push()
        assert 'Use cargo proofs run -- lake build' in cli('run', '--', 'cargo', 'aeneas', ok=False)
        env['TEST_LAKE_FAIL'] = '1'
        assert 'Lake build failed' in cli('run', '--', 'lake', 'build', ok=False)
        assert 'did not produce publishable' in cli('publish', ok=False)
        del env['TEST_LAKE_FAIL']
        env['TEST_LEAN_FAIL'] = '1'
        assert 'sorryAx' in cli('run', '--', 'lake', 'build', ok=False)
        assert 'did not produce publishable' in cli('publish', ok=False)
        del env['TEST_LEAN_FAIL']
        cli('run', '--', 'lake', 'build')
        cli('publish', '--dry-run')
        cli('publish')
        assert reports[3]['tool_version_id'] == 'aeneas-version'
        assert len(reports[3]['claims']) == 1
        claim = reports[3]['claims'][0]
        assert claim['precondition'] == 'x > 0'
        assert '/Proofs.lean#L1-L3' in claim['evidence_url']
        config.write_text(config.read_text().replace("x > 0", "x > 1"))
        assert 'Aeneas proof mappings changed' in cli('publish', ok=False)
        config.write_text(config.read_text().replace("x > 1", "x > 0"))
        assert 'No changes' in cli('publish')
        cli('logout')
        assert 'Not logged in' in cli('publish', ok=False)
        print('End-to-end: login, SARIF-only recording, published source, dry-run, create, no-op, conflict, force, removal, restore, retry, Creusot targets/claims/evidence, Aeneas Lean recording/failures/mappings, logout passed')
    finally:
        server.shutdown()
        thread.join()

