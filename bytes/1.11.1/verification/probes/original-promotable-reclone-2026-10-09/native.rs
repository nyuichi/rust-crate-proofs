use bytes::Bytes;

/// First promotion, promoted-root re-clone, lexical peer Drops, then saved return.
pub fn promoted_reclone_scope(input: Box<[u8]>) -> Vec<u8> {
    let original = Bytes::from(input);
    {
        let first = original.clone();
        let second = original.clone();
    }
    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();
    observed
}
