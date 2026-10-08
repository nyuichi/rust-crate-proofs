//! Generic Relaxed RMW release-sequence carry, separately trusted from protocols.
use creusot_std::{prelude::*, ghost::Perm, logic::Int,
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::Relaxed},
        committer::Committer, view::{HasTimestamp, SyncView, ReleaseSyncView}}};

/// Complete precisely this RMW, carrying its immediate predecessor publication.
/// Publication is only a Snapshot: it is neither an acquired current view nor
/// a proof that this thread's current view has been published. Relaxed ordering
/// advances this atomic's timestamp but supplies no prior <= current relation.
#[trusted]
#[check(ghost)]
#[requires(!c.shot_store() && c.ward() == *own.ward())]
#[ensures((*c).hist_inv(^c) && (^c).shot_store())]
#[ensures((*own).ward() == (^own).ward())]
#[ensures(*current <= ^current)]
#[ensures(c.ward().get_timestamp(*current) <= c.timestamp())]
#[ensures(c.timestamp() < c.ward().get_timestamp(^current))]
#[ensures(match (*own).val().get(c.timestamp()) { Some((v,_)) => v == c.val_load(), None=>false })]
#[cfg_attr(not(feature="negative_drop_relaxed_prior"), ensures(match (*own).val().get(c.timestamp()) { Some((_,prior)) => prior <= *result, None=>false }))]
#[ensures(release@ <= *result)]
#[ensures(forall<t:Int> (*own).val().get(t) != None ==> t <= c.timestamp())]
#[ensures((^own).val() == (*own).val().insert(c.timestamp()+1,(c.val_store(),*result)))]
pub fn relaxed_rmw(c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>,
    own:&mut Perm<ModelAtomic>,current:&mut SyncView,release:ReleaseSyncView)->Snapshot<SyncView> {
    panic!("ghost atomic semantics rule")
}
