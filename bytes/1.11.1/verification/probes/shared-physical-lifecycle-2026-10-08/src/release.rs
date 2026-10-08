use super::*;
/// RMW-specific completion: publication joins the previous modification's
/// carried view and this thread's release view, without acquiring the former.
/// The returned snapshot is publication metadata, never a current-view witness:
/// returning `SyncView` itself would allow recovery without an Acquire.
#[trusted]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[ensures((*c).hist_inv(^c) && (^c).shot_store())]
#[ensures((*own).ward() == (^own).ward())]
#[ensures(*current <= ^current && ^current <= *result)]
#[ensures(c.ward().get_timestamp(*current) <= c.timestamp())]
#[ensures(c.timestamp() < c.ward().get_timestamp(^current))]
#[ensures(match (*own).val().get(c.timestamp()) { Some((v,_)) => v == c.val_load(), None=>false })]
#[cfg_attr(not(feature="negative_drop_prior_publication"), ensures(match (*own).val().get(c.timestamp()) { Some((_,prior)) => prior <= *result, None=>false }))]
#[ensures(forall<t:Int> (*own).val().get(t) != None ==> t <= c.timestamp())]
#[ensures((^own).val() == (*own).val().insert(c.timestamp()+1,(c.val_store(),*result)))]
pub fn release_rmw(c:&mut Committer<ModelAtomic,usize,Relaxed,Release>,own:&mut Perm<ModelAtomic>,current:&mut SyncView)->Snapshot<SyncView> {
    panic!("ghost atomic semantics rule")
}

