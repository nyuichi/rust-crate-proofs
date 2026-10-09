
/// Saved return value followed by certified native terminal normal Drop edges.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn shared_automatic_scope(input:Vec<u8>)->Vec<u8> {
    let (first,mut cursor)=original_shared_from_vec(input);
    let second=original_shared_clone(&first,cursor.borrow_mut());
    let first_id=snapshot!(first.ticket_id());
    let second_id=snapshot!(second.ticket_id());
    let live_before=snapshot!((*cursor.inner_logic().observation()).0);
    let borrowed=original_shared_as_slice(&second);
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut second_receipt=ghost! {None::<Completion>};
    bytes_terminal_drop(second,cursor.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0 == (*live_before).remove(*second_id));
    proof_assert!((*cursor.inner_logic().observation()).0.contains(*first_id));
    proof_assert!(!(*cursor.inner_logic().observation()).0.contains(*second_id));
    let mut first_receipt=ghost! {None::<Completion>};
    bytes_terminal_drop(first,cursor.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic() != None && first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0.len() == 0);
    saved_return
}
