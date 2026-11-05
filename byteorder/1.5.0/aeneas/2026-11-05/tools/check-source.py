from pathlib import Path
import tarfile,hashlib
p=Path(__file__).resolve().parents[1]
with tarfile.open(p/'evidence/upstream.crate') as tf:
 for member in tf.getmembers():
  rel=Path(*Path(member.name).parts[1:])
  if member.isfile() and (rel.parts[0]=='src' or str(rel)=='Cargo.toml'):
   original=tf.extractfile(member).read();current=(p/rel).read_bytes()
   assert original==current,rel
   print(hashlib.sha256(current).hexdigest(),rel)
print('Original Rust source and package manifest match crates.io archive')
