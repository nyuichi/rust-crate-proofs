//! Body checked once-only callback loop adapted from shipped logically_atomic_faa.
use creusot_std::{prelude::*, ghost::FnGhost,
    std::sync::{atomic::{AtomicUsize, ordering::Relaxed}, committer::Committer}};

#[requires(forall<c:&mut Committer<AtomicUsize,usize,Relaxed,Relaxed>>
    !c.shot_store() && c.ward() == *at &&
    c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT &&
    c.val_store()@ == c.val_load()@ + 1 ==>
    f.precondition((c,)) && (f.postcondition_once((c,),()) ==> (^c).shot_store()))]
#[ensures(match result {
    Ok(old) => exists<c:&mut Committer<AtomicUsize,usize,Relaxed,Relaxed>>
        !c.shot_store() && c.ward() == *at &&
        c.val_load() <= crate::ref_count_limit::MAX_REF_COUNT &&
        c.val_store()@ == c.val_load()@ + 1 && old == c.val_load() &&
        f.postcondition_once((c,),()),
    Err(old) => old > crate::ref_count_limit::MAX_REF_COUNT,
})]
pub fn guarded_increment<F>(at:&AtomicUsize, f:Ghost<F>)->Result<usize,usize>
where F:FnGhost+FnOnce(&mut Committer<AtomicUsize,usize,Relaxed,Relaxed>) {
    let mut old = at.load::<_, Relaxed>(ghost!(|_: &_| {}));
    let old_f = snapshot!(f);
    let mut f = ghost!(Some(f.into_inner()));
    #[invariant(*f == Some(**old_f))]
    loop {
        let next = match crate::ref_count_limit::next_ref_count(old) {
            Some(next) => next,
            None => return Err(old),
        };
        match at.compare_exchange_weak::<_, Relaxed, Relaxed>(old, next,
            ghost!(|c:Result<&mut Committer<AtomicUsize,usize,Relaxed,Relaxed>,&_>| {
                if let Ok(c) = c { f.take().unwrap()(c) }
            })) {
            Ok(_) => return Ok(old),
            Err(observed) => old = observed,
        }
    }
}
