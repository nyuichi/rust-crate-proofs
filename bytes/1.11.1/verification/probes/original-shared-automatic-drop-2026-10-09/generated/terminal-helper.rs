#[requires(value.original_shared_valid() && value.accepts(*cursor.inner_logic()))]
#[requires(*output.inner_logic() == None)]
#[ensures((^cursor).model() == cursor.inner_logic().model() && (^cursor).public() == cursor.inner_logic().public())]
#[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.remove(value.ticket_id()))]
#[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1)]
#[ensures(^output != None && (^output).unwrap_logic().valid(cursor.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().reclaimed() == ((*cursor.inner_logic().observation()).0.len() == 1))]
fn bytes_terminal_drop(value:Bytes,cursor:Ghost<&mut Cursor>,output:Ghost<&mut Option<Completion>>) {
    original_shared_cleanup(value,cursor,output)
}
