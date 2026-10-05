# Error conversion leaves

This target includes the production `src/error/core.rs` implementation and
`src/uri/error.rs` empty-URI predicate. Its error payloads use the exact
published HTTP 1.5.0 component types from `verification/runtime-http`; the
target does not replace them with local error stand-ins. It excludes the outer
`src/error.rs` formatting and trait-object downcast methods because those
require dynamic `std::error::Error` behavior.

Run the source check and proof from this directory:

```sh
../../scripts/run-proof.sh cargo check --offline
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache -- --locked --offline
```

This is a leaf proof target, not an integrated HTTP crate proof. The body
results are from the latest source-matched Why3 run. These production bodies
passed with exact functional contracts:

- `error_core::Error::is_empty_uri` (3 VCs), using the real `InvalidUri`
  observer whose `Empty` tag is 10.
- All seven returning `From` conversions in `src/error/core.rs`: the unsuffixed
  `impl_From_for_Error::from` wraps `MaxSizeReached`; suffixes `_0` through
  `_5` wrap `InvalidStatusCode`, `InvalidMethod`, `InvalidUri`,
  `InvalidUriParts`, `InvalidHeaderName`, and `InvalidHeaderValue`, in that
  order. Each body VC and its `__refines` VC passed. The public opaque
  `ErrorModel` retains the exact payload value in each variant.
- `From<Infallible>` (`impl_From_for_Error_6::from`) and its `__refines` VC
  passed against `ensures(false)`, since this body has no returning execution.
- `uri_error::InvalidUri::is_empty` and `uri_error::From<ErrorKind>::from`
  each passed their one body VC.

The full target command exits unsuccessfully on five other files from the
included URI error module: derived `ErrorKind::eq` and its refinement, the
`Debug` refinements for `ErrorKind` and `InvalidUri`, and `InvalidUri::fmt`
(2/3 VCs). `InvalidUri::fmt` also calls an external formatter method without a
contract. Those outcomes do not fail the exact `Error` conversion VCs above;
they mean the full target is not integrated. Derived/debug-only successes and
the body-only `InvalidUri::s` check are not counted as exact functional API
proofs.

The outer `src/error.rs` methods were also attempted through a translation-only
`error_dynamic_api` cfg that includes the exact production file and its actual
`core` child. Translation reaches the `Debug`, `Display`, `is`, `get_ref`, and
`source` bodies but stops at the `get_ref` return type: Creusot reports
`forbidden dyn type: dyn std::error::Error (dyn support is currently minimal)`.
The formatter calls `debug_tuple`, `field`, `finish`, and `Display::fmt` also
warn that external methods have no contracts, so those formatting bodies have
no exact functional proof here. This is a concrete dynamic-trait translation
boundary for the outer methods; it does not block the concrete conversion
leaves above.
