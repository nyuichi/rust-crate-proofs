#[requires(value.child_valid() && scope.inner_logic().valid() && scope.inner_logic().is_shared())]
#[requires(value.child_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).is_shared() && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures((^scope).same_root(*scope.inner_logic()))]
#[ensures((^scope).same_pointer_owner(*scope.inner_logic()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.remove(value.child_id()))]
#[ensures(^output!=None && (^output).unwrap_logic().valid(value.child_public().3))]
#[ensures((^output).unwrap_logic().reclaimed()==((*scope.inner_logic().observation()).0.len()==1))]
fn bytes_child_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_child(value,scope,output)
}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().is_shared())]
#[requires((*scope.inner_logic().observation()).0.len()==1)]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).phase==None && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures(^output!=None && (^output).unwrap_logic().reclaimed())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().root_metadata()))]
fn bytes_root_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_root(value,scope,output)
}
