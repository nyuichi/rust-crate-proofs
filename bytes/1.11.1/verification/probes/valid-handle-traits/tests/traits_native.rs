//! Exercise the actual ordinary AsRef/AsMut trait implementations.
use bytes_valid_handle_traits::{traits_unique, unique_is_empty};

#[test]
fn unique_traits_cover_empty_known_and_spare_storage() {
    for value in [0, 255] {
        traits_unique(Vec::new(), value);
        traits_unique(Vec::with_capacity(8), value);
        traits_unique(Vec::from(Box::<[u8]>::from([1, 2, 3, 4])), value);
        let mut spare = Vec::with_capacity(8);
        spare.extend_from_slice(&[1, 2, 3, 4]);
        traits_unique(spare, value);
    }
}

#[test]
fn exact_is_empty_observer_covers_empty_and_nonempty_unique_handles() {
    assert!(unique_is_empty(Vec::new()));
    assert!(unique_is_empty(Vec::with_capacity(8)));
    assert!(!unique_is_empty(Vec::from([7, 8, 9])));
}
