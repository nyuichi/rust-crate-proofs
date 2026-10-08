# Safe registered function-pointer diagnostic

This probe separates callable contracts from function-item reification. It does
not verify original Bytes vtable dispatch, unsafe callbacks, or public Clone.
`through_trait` and `typed_callback` use shipped Std contracts; the later
registration getter **is new generic project TCB**. The initial header was
corrected after the negative capture; executable code is byte-identical.

`through_trait` invokes a safe `fn(u32)->u32` through explicit `Fn::call`, requiring
its shipped `FnExt::precondition` and ensuring its `postcondition`. The strong
`add_one` body and `registered_item_call` are checked. The private registration
macro takes one target identifier and uses that same item in the native return
and in both contract relations. Only that function-item-to-pointer relation is
trusted; `add_one`'s functional theorem is body proved.

This is a narrow, reviewed reification rule. There is no public factory accepting
an arbitrary pointer/spec pair, and no equality between code addresses is used.
It does not assume that function addresses uniquely identify code: compiler
merging/duplication of code is not an injectivity theorem. Integration with any
future function-pointer equality/address model needs separate compatibility
review. The contract vocabulary currently follows the opaque callable model.

The getter's trusted signature equates the pointer's pre/post to those of the
same checked function item. Its native body is just coercion of that item.
A future supported FnDef-to-FnPtr translation with contract preservation can
replace the getter without changing the checked clients. Such support is not
claimed to exist, and a small compiler patch is not promised. Corrupting the
trusted getter itself is a TCB change; negative callers do not validate arbitrary
changes inside trusted code.

## Distinguishing cases and expected classification

- Default: four body files prove, including the exact `x+1` registered caller.
- `negative_wrong_target`: the same registered `add_one` call claims `x+2`;
  the final result contract must fail. This tests an incorrect client theorem,
  not a mismatched-certificate API (the unsafe sibling probe tests the latter).
- `known_item`: direct `add_one` coercion encounters the actual unsupported
  ReifyFnPointer translation. Earlier arithmetic typing errors are only authoring
  diagnostics and must not be labeled reification failures.
- `unsafe_pointer`: Rust rejects using an unsafe function pointer as `Fn`.
- `negative_move_affine`: a non-Copy Ghost resource cannot be moved from `&self`.
  Relocating C's quota into a Bytes sidecar does not make public Clone callable.

Each archive, not this expected-outcome list, records an actual completed run.
The primitive assumes only normal-return behavior; no termination/unwind or
bytes ownership/refcount law is added.

## Capturing frozen results

Run captures only after the corresponding wrapper exits, before another run
cleans `verif`. The script never invokes a compiler/prover or changes input code.

```sh
python3 capture.py negative-wrong-target-5 --kind negative --exit 1 \
  --features negative_wrong_target --expected-files 5 --proof-log /path/to/negative.log
python3 capture.py positive-registered-4 --kind positive --exit 0 \
  --expected-files 4 --proof-log /path/to/positive.log --native-log /path/to/native.log
```

`--extra-log PATH` can be repeated for earlier frontend logs; label each according
to its actual source and diagnostic. A frontend capture uses `--kind frontend`
and deliberately excludes any stale proof tree. Positive captures require full
Coma/JSON correspondence, exit zero, zero recursive nulls, and the matching
completed engine summary. Captures include exact source/configuration, selected
Std Fn/resource analogues, and the relevant frontend source restrictions.

## Next prerequisite

The sibling unsafe probe proves scalar unsafe dispatch and checked forwarding
of an affine resource *outside* the callback. Actual bytes vtable integration
needs a checked shim that takes and transforms its own affine ghost input and
returns its ghost result, together with a structurally certified erasure to the
same native function and native signature. An arbitrary native-pointer/proof-shim
pairing is insufficient. Resource compatibility, wrong certificate, duplication,
and ghost erasure controls belong to that distinct experiment. Public Clone's
quota-free registration and automatic Drop remain separate obligations.
