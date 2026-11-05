from pathlib import Path
import tarfile, hashlib
root = Path(__file__).resolve().parents[1]
with tarfile.open(root/'evidence/upstream.crate') as archive:
    for name in ['src/lib.rs', 'Cargo.toml']:
        current = (root/name).read_bytes()
        original = archive.extractfile('nohash-hasher-0.2.0/'+name).read()
        assert current == original, f'Original source mismatch: {name}'
        print(hashlib.sha256(current).hexdigest(), name)
