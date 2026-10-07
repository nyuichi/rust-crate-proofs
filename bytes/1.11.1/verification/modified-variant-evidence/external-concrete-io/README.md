# External concrete IO contract consumer

Three actual downstream caller bodies prove with zero unresolved leaves; both
native tests pass. The dependency source/Cargo matches every corresponding byte
in `production-closed-io-263` (commit `011d21cc`), whose actual bodies were proved
separately. This consumer does not reprove those dependency bodies or establish
universal generic IO laws.

- `read_concrete`: `<Cursor as Read>::read`; exact min-length result, consumed
  suffix, copied destination prefix and unchanged destination tail.
- `write_then_close`: `<ExclusiveBytes as Write>::write`; exact count and arbitrary
  indexed observation of the concatenated bytes, then explicit close.
- `flush_then_close`: `<ExclusiveBytes as Write>::flush`; byte preservation through
  arbitrary indexed observation, then explicit close.

The IO Result values are returned unchanged. No error unwrap/dyn payload drop,
read_exact/write_all default-method, panic, unwind or allocation-failure theorem
is claimed. Native tests additionally cover 30 read geometries and empty/nonempty
write/flush cases. An initial native test-only vec macro ambiguity was fixed with
an explicit std::vec import; exact failed source/log are inside the archive.

Archive SHA256:
`fb859419b31359be23685512a93ba8acda42fdb42e9fae06c3aa855910f60892`.
All 66 indexed archive member hashes were independently checked after copying.
Proof completed exit0 with `Proved (3 files)`; each caller's own VC is present and
proved. Effective proof feature graph includes cargo-creusot's implicit
creusot-std/creusot and nightly flags; bytes features are verified,std.

To reproduce, extract `evidence.tar.gz` into a disposable directory, activate the
pinned bytes toolchain, then run `cargo test --locked` followed by `./verify.sh`
from its `consumer/` directory. The relative dependency path resolves to the
archived `dependency/`. Run the proof elevated; its wrapper holds the shared lock,
uses one prover and 1024 MiB, and rejects sc-drf. Full modified API/configuration
coverage remains a separate task.
