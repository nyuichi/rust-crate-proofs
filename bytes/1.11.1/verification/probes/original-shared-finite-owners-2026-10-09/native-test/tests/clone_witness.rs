use bytes_shared_finite_owners_native::finite_shared_scope;

#[test]
fn finite_owners_preserve_survivor_contents() {
    for len in [1usize, 2, 31, 256] {
        for count in [0usize, 1, 2, 7, 31] {
            let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
            assert_eq!(finite_shared_scope(expected.clone().into_boxed_slice(), count), expected);
        }
    }
}
