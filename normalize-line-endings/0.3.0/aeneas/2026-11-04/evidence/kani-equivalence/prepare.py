#!/usr/bin/env python3
"""Attach observation probes to complete, unedited source files.

No implementation body is transcribed into the harness. Each generated module
starts with every byte of the referenced Rust source and adds only the probe.
"""
from pathlib import Path
import hashlib
import json

here = Path(__file__).resolve().parent
root = here.parent.parent
out = here / 'generated'
out.mkdir(exist_ok=True)
probe = b'''

/// Verification-only observer of one call to the source's actual next method.
pub fn verification_probe_step<I: Iterator<Item = char>>(
    iter: I,
    prev_was_cr: bool,
) -> (Option<char>, bool, I) {
    let mut adapter = Normalized { iter, prev_was_cr };
    let result = adapter.next();
    (result, adapter.prev_was_cr, adapter.iter)
}
'''
manifest = {}
for name, source in [('original', root / 'src/lib.rs'),
                     ('adapted', root / 'translation-source/src/lib.rs')]:
    data = source.read_bytes()
    target = out / f'{name}.rs'
    target.write_bytes(data + probe)
    assert target.read_bytes()[:len(data)] == data
    manifest[name] = {'source': str(source.relative_to(root)),
                      'source_sha256': hashlib.sha256(data).hexdigest(),
                      'generated_sha256': hashlib.sha256(data + probe).hexdigest(),
                      'source_prefix_bytes': len(data)}
(here / 'source-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('Prepared original and adapted modules with byte-identical source prefixes')
