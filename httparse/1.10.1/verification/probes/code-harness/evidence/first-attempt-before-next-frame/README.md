# First `parse_code` proof batch evidence

This snapshot preserves the nine targets that completed before the first
attempt at `parse_code` was interrupted. The run used the isolated string-model
Creusot toolchain and the one-prover, 1000 MiB wrapper. All nine recorded
targets returned `Valid` with Z3 4.15.3; their exact Coma files, `proof.json`,
Why3 sessions, and SHA-256 hashes are retained here.

The run then spent 4m37s on `parse_code.coma` without producing a final Why3
status or a target proof summary. It was interrupted after review identified
that the old `Iterator::next` contract did not preserve `Bytes.mark` or state
the exact byte/cursor relation. No result is claimed for `parse_code` from that
attempt, and the interruption is not classified as `Unknown`, `Timeout`, or
`Invalid`.

The nine completed targets were the ASCII-digit predicate, decimal conversion,
exact code model, slice-head lemma, `Bytes::byte_permission`, `peek`,
`advance`, `bump`, and `Iterator::next`. This is preliminary dependency
evidence only; the final run must reprove these targets against the stronger
`next` contract.
