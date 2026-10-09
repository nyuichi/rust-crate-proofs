use bytes_promotable_suffix_promotion_native::promotable_suffix_scope;

#[test]
fn root_advance_before_first_clone_preserves_exact_suffix() {
    let mut cases = 0usize;
    for len in [1usize, 2, 3, 7, 16, 257] {
        let input: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(37) % 256) as u8).collect();
        for amount in 0..=len {
            let expected = input[amount..].to_vec();
            let actual = promotable_suffix_scope(input.clone().into_boxed_slice(), amount);
            assert_eq!(actual, expected, "len={len}, amount={amount}");
            cases += 1;
        }
    }
    assert_eq!(cases, 292);
    println!("AV native suffix cases: {cases}");
}
