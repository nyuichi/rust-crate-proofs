//! Checks the Rust correspondence of the explicit Lean wrapping models and
//! the complete original Rust state-transition body.
include!(concat!(env!("OUT_DIR"), "/modules.rs"));

#[cfg(kani)]
#[kani::proof]
fn std_wrapping_operator_models() {
    use core::num::Wrapping;
    let x: u32 = kani::any();
    let y: u32 = kani::any();
    let shift: usize = kani::any();
    assert_eq!((Wrapping(x) << shift).0, x.wrapping_shl(shift as u32));
    assert_eq!((Wrapping(x) >> shift).0, x.wrapping_shr(shift as u32));
    assert_eq!((Wrapping(x) ^ Wrapping(y)).0, x ^ y);
}

#[cfg(kani)]
#[kani::proof]
fn original_next_u32_matches_bitvector_formula() {
    let words: [u32; 4] = kani::any();
    let (output, next) = original::verification_probe(words);
    let t = words[0] ^ words[0].wrapping_shl(11);
    let expected = words[3] ^ words[3].wrapping_shr(19) ^ (t ^ t.wrapping_shr(8));
    assert_eq!(output, expected);
    assert_eq!(next, [words[1], words[2], words[3], expected]);
}
