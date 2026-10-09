use bytes::Bytes;

/// Actual lexical child Drop, then root read, return evaluation and root Drop.
pub fn promoted_automatic_scope(input: Box<[u8]>) -> Vec<u8> {
    let original = Bytes::from(input);
    {
        let child = original.clone();
    }
    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();
    observed
}
