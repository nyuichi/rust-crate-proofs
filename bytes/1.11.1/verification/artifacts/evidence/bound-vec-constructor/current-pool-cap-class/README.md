# Current constructor and physical-pool source gate

The exact-source constructor/explicit unique-release probe passes 41 proof
files on the same current raw_vec/owned_region/capacity/bytes_mut source.
The native extracted-constructor tests pass 3/3. This removes the mixed-kernel
snapshot limitation of the preceding 35-file cap-class checkpoint.

The run also checks the pool helper bodies included in these physical modules;
it does not call pool retirement from actual BytesMut split or Shared. It proves
neither native refcount nor automatic Drop nor full-crate integration. The
physical primitives remain trusted; no new trust is added.
