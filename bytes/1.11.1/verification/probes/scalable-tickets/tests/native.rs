use bytes_scalable_tickets::replace_twice_and_return_all;

#[test]
fn empty_ticket_count_is_independent_of_byte_coverage() {
    replace_twice_and_return_all(Vec::new());
    replace_twice_and_return_all(Vec::with_capacity(8));
    replace_twice_and_return_all(vec![1, 2, 3, 4]);
    let mut spare = Vec::with_capacity(16);
    spare.extend_from_slice(&[1, 2, 3, 4]);
    replace_twice_and_return_all(spare);
}
