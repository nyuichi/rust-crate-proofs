use bytes::Bytes;

/// Original owner retires first; child survives, reads and finally retires.
pub fn promoted_surviving_child_scope(input: Box<[u8]>) -> Vec<u8> {
    let survivor = {
        let original = Bytes::from(input);
        original.clone()
    };
    let observed = AsRef::<[u8]>::as_ref(&survivor).to_vec();
    observed
}
