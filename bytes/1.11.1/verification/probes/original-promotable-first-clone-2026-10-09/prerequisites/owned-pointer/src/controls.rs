//! Deliberate generic prerequisite contract violations; never positive admission.
use creusot_std::{prelude::*, std::sync::view::SyncView};

#[cfg(feature="wrong_expected")]
#[requires(initial != wrong)]
pub fn wrong_expected(initial:*mut (),wrong:*mut (),new:*mut ()) {
    let mut current=ghost! {SyncView::new().into_inner()};
    let (field,mut own)=crate::pointer_event::new_pointer(initial,current.borrow_mut());
    let _=crate::owned_pointer::exchange_singleton(&field,wrong,new,own.borrow_mut(),current.borrow_mut());
}

#[cfg(feature="missing_store")]
pub fn missing_store(initial:*mut (),new:*mut ()) {
    use creusot_std::std::sync::{atomic::{AtomicPtr,ordering::{Acquire,Release,None as NoStore}},committer::Committer};
    let mut current=ghost! {SyncView::new().into_inner()};
    let (field,own)=crate::pointer_event::new_pointer(initial,current.borrow_mut());
    let _=crate::owned_pointer::compare_exchange(&field,initial,new,ghost! {
        |event:Result<&mut Committer<AtomicPtr<()>,*mut (),Acquire,Release>,&Committer<AtomicPtr<()>,*mut (),Acquire,NoStore>>| {
            match event {
                Ok(c)=> {c.shoot_load(&*own,&mut *current);},
                Err(c)=> {c.shoot_load(&*own,&mut *current);},
            }
        }
    });
}

#[cfg(feature="stale_readonly")]
#[requires(initial != new)]
pub fn stale_readonly(initial:*mut (),new:*mut ()) {
    let mut current=ghost! {SyncView::new().into_inner()};
    let (field,mut own)=crate::pointer_event::new_pointer(initial,current.borrow_mut());
    let _=crate::owned_pointer::exchange_singleton(&field,initial,new,own.borrow_mut(),current.borrow_mut());
    let _invalid_binding=crate::pointer_event::bind_read_only(&field,initial,own);
}
