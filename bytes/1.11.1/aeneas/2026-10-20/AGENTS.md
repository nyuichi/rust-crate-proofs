# Independent Aeneas checkpoint

This directory is an archival Aeneas/Lean checkpoint, independent of the parent
Creusot target. Preserve the snapshot files byte-for-byte: historical receipts
hash their exact contents. Use `python3 check.py` for artifact integrity and
`python3 replay.py --aeneas-home ...` for saved-Lean replay in a new directory.
Neither command runs the parent's Creusot/Why3 workflow or changes its evidence.

Do not treat a saved-Lean replay as fresh Rust extraction. Do not combine these
partial results with the parent crate's Creusot coverage. Keep new results apart
from this immutable checkpoint. The experimental chain-binder patch is not an
applied tool patch for the recorded proofs.
