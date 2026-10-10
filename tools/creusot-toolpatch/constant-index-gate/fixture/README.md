# Constant array-index translation gate

This isolated fixture checks Creusot's narrow support for array-pattern lane reads. `get_pattern` accepts an arbitrary `[u8; 4]`, matches the literal `GET `, and promises the equivalent four-lane predicate. It does not constrain its input. The original `httparse` `parse_method` and `parse_token` bodies were separately extracted byte-for-byte in a local parser preparation; that work is outside this gate bundle.

The translated arbitrary-input positive target produced nine split VCs and all nine were `Valid` under Z3 4.15.3. Its COMA contains reads at offsets 0, 1, 2, and 3 through `Slice64.get`; the first four generated goals are the `Slice64.get` obligations at `slice.coma:101`, and the remaining five establish the result contract. This confirms that the compiler lowering retained and discharged ordinary bounds obligations for this array-pattern fixture.

The first negative control, `reject_false_contract([u8; 4])`, has the same pattern body and an impossible `result == false` postcondition. Its frozen run discharged four goals, then timed out at 30 seconds on the postcondition goal with no counterexample. Treat that outcome as inconclusive. Do not rerun that target or report it as a rejected contract.

The later `get_witness` and `reject_witness_contract` functions form a separate, concrete caller shape. Each calls only the positively specified `get_pattern` with `[71, 69, 84, 32]`; its translation and type-only check passed. `check-ground-diagnostic.py` guards the evidence path and checks all four translated lanes, the original matching branch's `bb6` true return, and the false postcondition. The resulting quantifier-free ground query returned `sat` under Z3 4.15.3. This confirms only the small fixed-witness diagnostic; it is not a counterexample result for the frozen full negative VC, which remains inconclusive after its timeout.

Fresh result evidence and the bounded interpretation are in [`evidence/proof-witness-20261005T181420Z/RESULTS.md`](evidence/proof-witness-20261005T181420Z/RESULTS.md). The `get_witness` body produced two `Valid` VCs (0.01089s and 0.00394s); its CoMa was generated with compiler SHA-256 `49e855f2d3222a7cd294e0b64e75a637207ad08207e97cda01390ce54c8694ac`, and the copied Why3 configuration hashes to `8bcc5dbb29bd1dfb8586f63d00b6592680468dbefe76d7a688101063000fbafe`. The setup-only attempt in `evidence/proof-witness-20261005T181117Z/` used the stale Why3 data path and produced no solver results; it is preserved separately.

To rebuild the isolated compiler, run `./build-compiler.sh` from `tools/creusot-toolpatch/constant-index-gate/`. Then run:

```sh
./translate.sh
./typecheck.sh
python3 ./check-ground-diagnostic.py
```

This gate does not establish parser-body proof obligations or parser integration. The isolated positive target tests array reads; although the compiler patch also lowers writes through `Slice.set`, this gate has no write-path VC.
