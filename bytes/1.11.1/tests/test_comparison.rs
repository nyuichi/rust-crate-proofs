use bytes::BytesMut;
use std::cmp::Ordering;

#[test]
fn vec_partial_order_with_bytes_mut_preserves_operand_order() {
    let cases: &[(&[u8], &[u8], Ordering)] = &[
        (b"", b"", Ordering::Equal),
        (&[0], &[1], Ordering::Less),
        (&[1], &[0], Ordering::Greater),
        (&[0], &[0, 1], Ordering::Less),
        (&[0, 1], &[0], Ordering::Greater),
    ];

    for &(vec_bytes, mutable_bytes, expected) in cases {
        let vec = vec_bytes.to_vec();
        let bytes_mut = BytesMut::from(mutable_bytes);

        assert_eq!(vec.partial_cmp(&bytes_mut), Some(expected));
        assert_eq!(bytes_mut.partial_cmp(&vec), Some(expected.reverse()));
    }
}
