#[ensures(result@ == input@)]
pub(crate) fn boxed_read_then_drop(input:Box<[u8]>)->Vec<u8> {
    let bytes=original_bytes_from_box(input);
    let metadata=snapshot!(bytes.boxed_metadata());
    let observed=original_bytes_as_slice(&bytes).to_vec();
    let saved_return=observed;
    let mut completion=ghost! {None::<BoxedCompletion>};
    bytes_terminal_drop(bytes,completion.borrow_mut());
    proof_assert!(completion.inner_logic() != None && completion.inner_logic().unwrap_logic().valid(*metadata));
    saved_return
}
