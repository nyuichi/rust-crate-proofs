use bytes::Bytes;

/// Exercise first cloning of a boxed-slice-backed Bytes handle. Releasing the
/// clone must leave the original readable until its own explicit cleanup.
pub fn boxed_clone_drop_child_read_parent_drop(input: Box<[u8]>) -> Vec<u8> {
    let original = Bytes::from(input);
    let child = original.clone();
    child.cleanup();
    let observed = AsRef::<[u8]>::as_ref(&original).to_vec();
    original.cleanup();
    observed
}
