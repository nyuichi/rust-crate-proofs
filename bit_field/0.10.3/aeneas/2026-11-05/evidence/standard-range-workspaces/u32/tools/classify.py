#!/usr/bin/env python3
"""Classify future source-only freeze members, preserving raw inputs unchanged."""
from pathlib import PurePosixPath
import fnmatch
SIDE_SUFFIX_GLOBS=('*.olean*','*.ir*','*.ilean','*.o')
CACHE_COMPONENTS=frozenset({'target','.lake','__pycache__'})
def classify(path):
    p=PurePosixPath(path)
    if any(x in CACHE_COMPONENTS for x in p.parts):return 'build-cache'
    if any(fnmatch.fnmatchcase(p.name,pattern) for pattern in SIDE_SUFFIX_GLOBS):return 'compiler-sidecar'
    return 'retain-source-or-evidence'
def source_members(files):
    return {path:digest for path,digest in files.items() if classify(path)=='retain-source-or-evidence'}
def assert_source_only(files):
    rejected={p:classify(p) for p in files if classify(p)!='retain-source-or-evidence'}
    if rejected:raise ValueError('Compiler/cache members in source-only freeze: '+repr(rejected))
    return True
if __name__=='__main__':
    import argparse,json
    parser=argparse.ArgumentParser();parser.add_argument('manifest');args=parser.parse_args()
    with open(args.manifest) as stream:doc=json.load(stream)
    files=doc.get('files',doc)
    print(json.dumps({'excluded':{p:{'sha256':h,'classification':classify(p)} for p,h in files.items() if classify(p)!='retain-source-or-evidence'},'retained_count':len(source_members(files))},indent=2))
