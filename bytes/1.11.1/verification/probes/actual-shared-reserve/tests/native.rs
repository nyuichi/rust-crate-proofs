use bytes_actual_shared_reserve::{reserve_shared, reserve_shared_with_sibling};

#[test]
fn public_shared_reserve_paths() {
    for len in [0, 1, 4, 16] {
        for cut in [0, len / 2, len] {
            for additional in [0, 1, 17, 128] {
                let capacity = reserve_shared(vec![3; len], cut, additional, 9);
                assert!(capacity >= len - cut + additional);
            }
        }
        assert!(reserve_shared_with_sibling(vec![3; len], 128) >= len + 128);
    }
}
