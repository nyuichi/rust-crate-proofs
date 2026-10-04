# Shared native compilation blocker

These attempts stopped before Creusot translation. In the raw-vec source captured here, `BoundPtr` uses `#[repr(transparent)]` while the `cfg(creusot)` representation contains both `NonNull<u8>` and `Ghost<Option<BoundPtrBinding>>`. The pinned compiler reports Rust E0690 because the transparent type has two non-trivial fields. No B2/B3/B4 verification condition was generated or observed in these attempts. The logs are retained only as compile-failure diagnostics, not as negative proof evidence.
