# Proof-only getter leaf diagnostic

This is the changed-premise follow-up to the first vtable-leaf experiment. The
first experiment kept a `SHARED_VTABLE` static definition in both feature
configurations; Creusot rejected the static item before reaching either record
field expression. This version removes that definition from proof cfg entirely
while retaining it in native cfg.

The proof configuration has a private, trusted `shared_vtable() ->
&'static Vtable` function with only `requires(true)` and `ensures(true)`. Its
proof-only body is `unimplemented!()` and is excluded from runtime behavior by
`cfg(creusot)`; trust means the body is not used to derive any property. The
native configuration defines `SHARED_VTABLE` with the callback initializer and
implements `shared_vtable()` as `&SHARED_VTABLE`. The `Bytes` record body calls
the getter in the `vtable` field.

Unlike the earlier opaque `extern STATIC` fixture, proof cfg declares no static
symbol and performs no direct static read. It uses a function call whose return
type is `&'static Vtable`. Its contract gives no table identity, callback,
ownership, or bytes facts. Native code and proof abstraction are deliberately
different; there is no correspondence proof between the trusted proof-only
function and native getter.

## Result

With the pinned activated toolchain and `/tmp/itoa-creusot-proof.lock`,
`./run.sh` ran:

- `BYTES_TRANSLATE_ONLY=1 cargo creusot --only=coma -- --locked`: exit 0.
  Creusot emitted `artifacts/proof-local-trusted-stub-verif/bytes_vtable_leaf_extern_getter_rlib/from_vec_record.coma`.
- `cargo check --locked` under native cfg: exit 0. The local static and native
  getter compile with the probe's divergent callback stubs.

The generated COMA declares the complete `Vtable` shape, with `clone`,
`into_vec`, `into_mut`, `is_unique`, and `drop` lowered as `Opaque.ptr`. It also
emits a `from_vec_record` translation, but its returned logical value is
abstracted (`Any.any_l()`); this is not a checked relation between the input
fields and returned `Bytes`. Creusot also warns that `AtomicPtr::new` has no
contract and would yield an impossible caller precondition in a proof. No Why3
phase ran and no VC or proof claim exists.

This establishes only that the source-sliced field-construction translation
can pass when its vtable reference is supplied by the true/true trusted getter.
It does not establish the production static-table initializer, actual
`From<Vec<u8>>`, callback semantics, or a Bytes API proof. The probe retains the
production field layout and exact `Vtable` declaration; `BytesMut` and callback
bodies remain stand-ins.

Literal extern-block contract syntax attempts are preserved under
`logs/extern-abi-exploration/` as diagnostics. The canonical positive case is
the ordinary local trusted proof stub described above.
