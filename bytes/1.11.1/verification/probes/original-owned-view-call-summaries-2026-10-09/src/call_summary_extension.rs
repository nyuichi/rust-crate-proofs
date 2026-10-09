
// AU: ordinary, separately verified native function summary. The additional
// scope argument is ghost-erased; no owning resource is extracted from a model.
#[requires(source.api_view_valid() && source.view_owned() && source.view_accepts(*scope.inner_logic()))]
#[requires(amount<=source.len)]
#[ensures(result.api_view_valid() && result.view_owned())]
#[ensures(result.view_content()==source.view_content().subsequence(amount@,source.len@))]
#[ensures(result.len@==source.len@-amount@)]
#[ensures(result.ptr.addr_logic()@==source.ptr.addr_logic()@+amount@)]
#[ensures(result.shares_view_allocation(*source))]
#[ensures(result.view_bound()@==Some((source.view_bound()@.unwrap_logic().0,
    source.view_bound()@.unwrap_logic().1,source.view_bound()@.unwrap_logic().2+amount@)))]
#[ensures(result.view_public()==source.view_public() && result.view_accepts(^scope) && source.view_accepts(^scope))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.view_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_suffix_checked(source:&Bytes,amount:usize,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let mut result=clone_owned_api(source,ghost! {&mut **scope});
    advance_api(&mut result,amount);
    result
}
