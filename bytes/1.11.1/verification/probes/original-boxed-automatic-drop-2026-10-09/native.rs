use bytes::Bytes;

pub fn boxed_read_then_drop(input: Box<[u8]>) -> Vec<u8> {
    let bytes = Bytes::from(input);
    let observed = AsRef::<[u8]>::as_ref(&bytes).to_vec();
    observed
}
