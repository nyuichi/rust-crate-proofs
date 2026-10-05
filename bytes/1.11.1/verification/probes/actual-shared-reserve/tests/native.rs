use bytes_actual_shared_reserve::{
    extend_shared_growth, reserve_shared, reserve_shared_with_sibling, resize_shared_growth,
};

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

#[test]
fn shared_growth_methods_write_before_publishing() {
    for value in [0, 7, u8::MAX] {
        assert_eq!(resize_shared_growth(value), value);
        assert_eq!(extend_shared_growth(value), value);
    }
}
