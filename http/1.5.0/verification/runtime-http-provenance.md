# Opaque runtime type dependency

`verification/runtime-http` is the unmodified published `http` 1.5.0 crate
archive from `provenance/http-1.5.0.crate`, extracted for leaf harnesses that
need published HTTP component types to remain opaque without translating the
annotated local crate as a dependency. The archive SHA-256 is
`918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0`.

The Request/Response and Error harnesses still include the exact local
production source bodies under verification. This dependency only supplies the
same-version opaque component types; it does not prove their local annotated
implementations or an integrated HTTP crate build.
