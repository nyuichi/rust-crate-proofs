"""Experimental trusted LLBC normalization, not an equivalence proof.
Replace scalar-char switches with ordered Eq comparisons and bool switches.
Input must be Charon's --no-dedup-serialized-ast JSON. Rust is unchanged.
"""
import copy
import json
from pathlib import Path

BASE = Path(__file__).parent
source = json.loads((BASE / 'normalize_line_endings.llbc').read_text())
assert not source['has_errors']
BOOL = {'Untagged': {'Scalar': 'Bool'}}
CHAR = {'Untagged': {'Scalar': 'Char'}}
serial = 1000000
rewritten = 0

def fresh():
    global serial
    serial += 1
    return serial

def statement(span, kind):
    return {'span': copy.deepcopy(span), 'id': fresh(), 'kind': kind, 'comments_before': []}

def block(span, stmts):
    return {'span': copy.deepcopy(span), 'id': fresh(), 'statements': stmts}

def rewrite_block(b, locals_):
    global rewritten
    result = []
    for st in b['statements']:
        kind = st['kind']
        if isinstance(kind, dict) and 'Switch' in kind:
            sw = kind['Switch']
            for branch in sw['branches']:
                rewrite_block(branch, locals_)
            data = sw['data']
            value = data['scrutinee'].get('Value')
            if value is not None and data['branches'] and all(c['Untagged'][1] == CHAR for c, _ in data['branches']):
                # Char is Copy. Restrict the experiment to Copy to avoid changing
                # move/deinitialization semantics when testing multiple labels.
                assert set(value) == {'Copy'} and value['Copy']['ty'] == CHAR
                assert data['fallback'] is not None
                assert all(set(c['Untagged'][0]) == {'Char'} for c, _ in data['branches'])
                span = st['span']
                tail = copy.deepcopy(sw['branches'][data['fallback']])
                for constant, branch_id in reversed(data['branches']):
                    local_id = len(locals_)
                    locals_.append({'index': local_id, 'name': 'char_switch_eq', 'span': copy.deepcopy(span), 'ty': copy.deepcopy(BOOL), 'drop_flag_for': None})
                    place = {'kind': {'Local': local_id}, 'ty': copy.deepcopy(BOOL)}
                    yes = copy.deepcopy(sw['branches'][branch_id])
                    no = tail
                    for branch in (yes, no):
                        branch['statements'].insert(0, statement(span, {'StorageDead': local_id}))
                    test = {'data': {'scrutinee': {'Value': {'Move': copy.deepcopy(place)}},
                                     'branches': [[{'Untagged': [{'Bool': True}, copy.deepcopy(BOOL)]}, 0]], 'fallback': 1},
                            'branches': [yes, no]}
                    tail = block(span, [statement(span, {'StorageLive': local_id}),
                                        statement(span, {'Assign': [place, {'BinaryOp': ['Eq', copy.deepcopy(value), {'Const': copy.deepcopy(constant)}]}]}),
                                        statement(span, {'Switch': test})])
                result.extend(tail['statements'])
                rewritten += 1
                continue
        result.append(st)
    b['statements'] = result

for f in source['translated']['fun_decls']:
    if f and isinstance(f['body'], dict) and 'Structured' in f['body']:
        body = f['body']['Structured']
        rewrite_block(body['body'], body['locals']['locals'])
assert rewritten == 2, rewritten
(BASE / 'normalized.llbc').write_text(json.dumps(source))
print(f'Experimentally normalized {rewritten} char switches; Rust source unchanged')
