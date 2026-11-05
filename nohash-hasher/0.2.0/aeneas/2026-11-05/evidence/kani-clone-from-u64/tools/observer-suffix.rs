
// Verification-only read-only observer; original source above is an exact byte prefix.
#[cfg(all(kani, debug_assertions))]
pub fn __verification_observe_u64(h: &NoHashHasher<u64>) -> (u64, Option<bool>) {
    (h.0, Some(h.1))
}
#[cfg(all(kani, not(debug_assertions)))]
pub fn __verification_observe_u64(h: &NoHashHasher<u64>) -> (u64, Option<bool>) {
    (h.0, None)
}
