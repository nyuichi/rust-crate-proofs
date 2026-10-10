# Outer Error formatter proof

This harness translates the production `src/error.rs` and `src/method.rs`
implementations. It uses the exact published HTTP 1.5.0 payload types through
`verification/runtime-http`; it introduces no local error payload stand-ins.

Run `./prove-formatters.sh` from this directory to clean, emit, and prove the
two outer formatter bodies under the same named cfg profile. The profile
excludes only the dynamic `std::error::Error` methods (`is`, `get_ref`, and
`source`) from this proof harness, because Creusot currently rejects those
trait-object signatures. The normal source build retains those methods.

The formatter bodies dispatch on the actual `ErrorKind` and call the same
concrete payload formatting implementations selected by the original trait
object. The contracts check that formatting extends the existing formatter
model; they do not expose the exact rendered character sequence.
