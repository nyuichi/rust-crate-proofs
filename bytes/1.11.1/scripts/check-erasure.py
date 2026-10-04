#!/usr/bin/env python3
"""Check the reviewed primitive substitutions in the extracted deallocation body.
This is source correspondence, not a proof of the entire compiler's ghost erasure.
"""
import hashlib, json, pathlib, re
root = pathlib.Path(__file__).resolve().parents[1]

def body(text, signature):
    begin = text.index('{', text.index(signature))
    depth = 0
    for i in range(begin, len(text)):
        depth += (text[i] == '{') - (text[i] == '}')
        if depth == 0:
            return text[begin + 1:i]
    raise ValueError('unclosed function body')

runtime = body((root/'src/bytes.rs').read_text(), 'unsafe fn free_boxed_slice(')
probe = body((root/'verification/probes/deallocation/src/lib.rs').read_text(), 'pub unsafe fn free_boxed_slice(')
probe, count = re.subn(r'let live = ghost! \{ ownership.live\(\) \};', '', probe)
assert count == 1
probe = probe.replace('offset_from_live(offset, buf, live)', 'offset.offset_from(buf)')
probe = probe.replace('dealloc_bytes(buf, Layout::from_size_align(cap, 1).unwrap(), ownership)',
                      'dealloc(buf, Layout::from_size_align(cap, 1).unwrap())')
normalize = lambda s: re.sub(r'\s+', '', s)
assert normalize(runtime) == normalize(probe), 'runtime/probe erasure correspondence changed'
manifest = dict(component='bytes::free_boxed_slice',
    runtime_body_sha256=hashlib.sha256(normalize(runtime).encode()).hexdigest(),
    probe_erased_body_sha256=hashlib.sha256(normalize(probe).encode()).hexdigest(),
    substitutions=['erase ghost live witness', 'native offset_from wrapper', 'native dealloc wrapper consuming ghost ownership'],
    review='Astra reviewed primitive contracts; compiler-wide erasure is part of the TCB',
    limitation='Bytes callers, vtable dispatch and automatic Drop are not connected')
(root/'verification/artifacts/deallocation-correspondence.json').write_text(json.dumps(manifest, indent=2)+'\n')
print('free_boxed_slice: reviewed ghost/primitive substitution restores the exact runtime body')
