use bytes_vec_capacity_guarantee::native_allocate_and_release;

#[test]
fn requested_capacity_is_preserved_through_b1_detach() {
    let (len, capacity) = native_allocate_and_release(0);
    assert_eq!(len, 0);
    assert_eq!(capacity, 0);

    for requested in [1, 16, 256, 4096] {
        let (len, capacity) = native_allocate_and_release(requested);
        assert_eq!(len, 0);
        assert!(capacity >= requested);
    }
}
