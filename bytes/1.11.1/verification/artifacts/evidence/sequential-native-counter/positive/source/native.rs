use bytes_sequential_native_counter::{scalar_transitions, two_handle_countdown};
#[test]
fn native_atomic_values() {
    assert_eq!(two_handle_countdown(), 0);
    assert_eq!(scalar_transitions(0, 0, 0), 0);
    assert_eq!(scalar_transitions(7, 4, 3), 8);
    assert_eq!(scalar_transitions(usize::MAX, 0, usize::MAX), 0);
    assert_eq!(scalar_transitions(usize::MAX - 1, 1, 1), usize::MAX - 1);
}
