use bytes_actual_with_capacity::native_actual_with_capacity_then_release;

#[test]
fn exact_with_capacity_constructor_preserves_request_and_releases() {
    let (len, capacity) = native_actual_with_capacity_then_release(0);
    assert_eq!(len, 0);
    assert_eq!(capacity, 0);

    for requested in [1, 16, 256, 4096] {
        let (len, capacity) = native_actual_with_capacity_then_release(requested);
        assert_eq!(len, 0);
        assert!(capacity >= requested);
    }
}
