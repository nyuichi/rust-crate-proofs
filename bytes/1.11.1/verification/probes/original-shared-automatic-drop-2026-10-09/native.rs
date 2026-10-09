use bytes::Bytes;
/// Native normal-return scope: no explicit cleanup or mem::drop call.
#[cfg_attr(creusot, requires(input@.len() < creusot_std::std::vec::capacity_model(input)))]
#[cfg_attr(creusot, ensures(result@ == input@))]
pub fn shared_automatic_scope(input: Vec<u8>) -> Vec<u8> {
    let first = Bytes::from(input);
    let second = first.clone();
    let observed = AsRef::<[u8]>::as_ref(&second).to_vec();
    observed
}
