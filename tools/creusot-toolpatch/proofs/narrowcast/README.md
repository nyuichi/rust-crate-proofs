The bundled source is universal over all `u128` values. Under the final bundled
BV128-only overlay, the fresh run passed both proof units (`trunc_u64_bw` and
`trunc_u64_int`) with one aggregate VC each, discharged by CVC5 1.3.1. The
original narrow-cast-only experiment split these into four bitwise and two
integer VCs, also all green. The captured final-overlay `.coma` and
per-function proof JSON files are in this directory.
