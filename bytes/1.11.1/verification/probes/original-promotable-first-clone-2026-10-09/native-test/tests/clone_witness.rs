use bytes_promotable_first_clone_native::boxed_clone_drop_child_read_parent_drop;

#[test]
fn boxed_clone_cleanup_preserves_readable_original() {
    for len in [1usize, 2, 31, 256] {
        let expected: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let input = expected.clone().into_boxed_slice();
        assert!(!input.is_empty(), "this witness requires a nonempty Box");
        assert_eq!(
            boxed_clone_drop_child_read_parent_drop(input),
            expected,
            "contents changed at length {len}",
        );
    }
}
