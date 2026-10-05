use bytes::{Buf, BytesMut};

#[test]
fn growing_unique_views_preserve_contents() {
    for capacity in [0, 1, 8, 64] {
        let mut buf = BytesMut::with_capacity(capacity);
        buf.extend_from_slice(b"abcdefgh");
        for consumed in [0, 1, 3] {
            let mut advanced = BytesMut::from(&buf[..]);
            advanced.advance(consumed);
            let expected = advanced.to_vec();
            let request = advanced.capacity() + 17;
            advanced.reserve(request);
            assert_eq!(&advanced[..], &expected[..]);
            assert!(advanced.capacity() - advanced.len() >= request);
            advanced.extend_from_slice(b"ijk");
            assert_eq!(&advanced[..expected.len()], &expected[..]);
            assert_eq!(&advanced[expected.len()..], b"ijk");
        }
    }
}

#[test]
fn empty_growth_publishes_only_written_bytes() {
    let mut buf = BytesMut::with_capacity(0);
    buf.reserve(33);
    assert!(buf.is_empty());
    assert!(buf.capacity() >= 33);
    buf.resize(33, 0x5a);
    assert_eq!(&buf[..], &[0x5a; 33]);
}

#[test]
fn reclaim_moves_a_disjoint_view_without_allocating() {
    let mut buf = BytesMut::from(&b"abcdefgh"[..]);
    let base = buf.as_ptr();
    let original_capacity = buf.capacity();
    buf.advance(6);
    let request = buf.capacity() - buf.len() + 1;
    assert!(buf.try_reclaim(request));
    assert_eq!(buf.as_ptr(), base);
    assert_eq!(buf.capacity(), original_capacity);
    assert_eq!(&buf[..], b"gh");
    buf.extend_from_slice(b"ijk");
    assert_eq!(&buf[..], b"ghijk");
}

#[test]
fn failed_reclaim_preserves_the_view_before_growth() {
    let mut buf = BytesMut::from(&b"abcdefgh"[..]);
    buf.advance(1);
    let old_pointer = buf.as_ptr();
    let old_capacity = buf.capacity();
    let request = buf.capacity() - buf.len() + 1;
    assert!(!buf.try_reclaim(request));
    assert_eq!(buf.as_ptr(), old_pointer);
    assert_eq!(buf.capacity(), old_capacity);
    assert_eq!(&buf[..], b"bcdefgh");
    buf.reserve(request);
    assert_eq!(&buf[..], b"bcdefgh");
    assert!(buf.capacity() - buf.len() >= request);
}
