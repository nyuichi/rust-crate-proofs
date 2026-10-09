use bytes_promotable_surviving_child_native::promoted_surviving_child_scope;

#[test]
fn promoted_inner_drop_preserves_readable_original() {
    for len in [1usize, 2, 31, 256] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let input = expected.clone().into_boxed_slice();
        assert!(!input.is_empty(), "this witness requires a nonempty Box");
        assert_eq!(
            promoted_surviving_child_scope(input),
            expected,
            "contents changed at length {len}",
        );
    }
}
