#[test]
fn disjoint_views_preserve_bytes_and_allocation() {
    for size in 0..40 {
        for offset in 0..=size {
            for len in 0..=offset.min(size - offset) {
                let mut input = Vec::with_capacity(size + 17);
                input.extend((0..size).map(|i| (i * 13 + 7) as u8));
                bytes_unique_reclaim::check_reclaim(input, offset, len);
            }
        }
    }
}

#[test]
fn empty_allocation_and_empty_one_past_view() {
    bytes_unique_reclaim::check_reclaim(Vec::new(), 0, 0);
    bytes_unique_reclaim::check_reclaim(vec![1, 2, 3, 4], 4, 0);
}
