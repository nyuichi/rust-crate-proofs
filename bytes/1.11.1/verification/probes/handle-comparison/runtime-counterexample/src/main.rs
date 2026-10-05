use bytes::BytesMut;
use std::cmp::Ordering;
fn main() {
    let left = vec![0u8];
    let right = BytesMut::from(&[1u8][..]);
    let actual = left.partial_cmp(&right);
    let expected = left.as_slice().partial_cmp(right.as_ref());
    println!("Vec([0]).partial_cmp(BytesMut([1])) = {actual:?}; byte-slice comparison = {expected:?}");
    assert_eq!(expected, Some(Ordering::Less));
    assert_eq!(actual, expected, "heterogeneous ordering must preserve receiver/RHS order");
}
