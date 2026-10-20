# cargo-elicit integration record

The selected-root Elicit run verifies 24 mapped Rust API contracts through Charon, Aeneas, and Lean 4.35.0. The original 10 accepted contracts plus 12 newly added generic adapter contracts yield 22 accepted of 24 in the fresh, stable audit, using only Lean's standard `propext` and `Quot.sound` axioms. The remaining two Bytes field-projection proofs compile but fail the strict custom-axiom audit because their translated models contain opaque declarations.

The 22 accepted adapter contracts are:

- `Take::new`, `Take::into_inner`, `Take::limit`, `Take::set_limit`, `Take::get_ref`, and `Take::get_mut`.
- `Chain::new`, `Chain::first_ref`, `Chain::first_mut`, `Chain::last_ref`, `Chain::last_mut`, and `Chain::into_inner`.
- `Limit::new`, `Limit::get_ref`, `Limit::get_mut`, `Limit::limit`, `Limit::set_limit`, and `Limit::into_inner`.
- `IntoIter::new`, `IntoIter::get_ref`, `IntoIter::get_mut`, and `IntoIter::into_inner`.

Mutable getters specify Aeneas's returned value and continuation: replacing the inner value preserves the `Take` or `Limit` limit, while `IntoIter` replaces its only field. These contracts were matched against the Rust methods and generated bodies and semantically reviewed. The selected Charon roots in `bytes-adapted/Cargo.toml` include the newly covered generic APIs; the native verification runner already reads their corresponding roots from `adapter-extraction-roots.json`.

`Bytes::len` remains unresolved because its theorem depends on `ElicitGenerated.bytes.Vtable`. `BytesMut::len` remains unresolved because its theorem depends on `ElicitGenerated.core.ptr.non_null.NonNull`, `ElicitGenerated.core.sync.atomic.Atomic`, and `ElicitGenerated.core.sync.atomic.private.Align8`. No custom axiom was removed or whitelisted. This is selected API verification, not a whole-crate result.

Reproduce from the adapted crate directory:

```sh
cd /workspace/bytes-aeneas-fresh/bytes-adapted
source /workspace/bytes-aeneas-fresh/env.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
cargo elicit check
cargo elicit audit
cargo elicit status --all
```

`check-expanded-final.log`, `audit-expanded-stable.log`, `review-expanded-final.log`, and `status-expanded-final.log` record the final CLI output. The authoritative 24-goal audit receipt is `audit-expanded-final-receipt.json`; it is also refreshed at `audit-selected-artifacts/receipt.json` alongside the corresponding generated Lean files and `crate.llbc`. `check-expanded-final-receipt.json` and `check-expanded-final-artifacts/` preserve the check receipt and generated files. Earlier diagnostics remain in `ElicitGenerated.failed.log`, `check-whole-crate-failed.log`, `check-selected-proofs-failed.log`, and `Proofs.failed.log`.
