#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::error::Error;
use std::sync::Arc;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard, OwnedMappedMutexGuard, OwnedMutexGuard};
use tokio_test::task::spawn;
use tokio_test::{assert_pending, assert_ready};

#[test]
fn borrowed_fifo_cancellation_and_drop_release() {
    let mutex = Mutex::new(vec![0usize, 1]);
    let first = mutex.try_lock().unwrap();

    let mut oldest = spawn(mutex.lock());
    let mut cancelled = spawn(mutex.lock());
    let mut newest = spawn(mutex.lock());
    assert_pending!(oldest.poll());
    assert_pending!(cancelled.poll());
    assert_pending!(newest.poll());

    drop(cancelled);
    drop(first);

    let oldest_guard = assert_ready!(oldest.poll());
    assert_pending!(newest.poll());
    drop(oldest_guard);
    let newest_guard = assert_ready!(newest.poll());
    drop(newest_guard);
    assert!(mutex.try_lock().is_ok());
}

#[test]
fn borrowed_map_try_map_and_nested_map_keep_one_permit() {
    let mutex = Mutex::new((vec![1usize, 2], 3usize));
    let guard = mutex.try_lock().unwrap();
    assert!(mutex.try_lock().is_err());

    let guard = MutexGuard::try_map(guard, |_| None::<&mut Vec<usize>>).unwrap_err();
    assert!(mutex.try_lock().is_err());
    assert!(std::ptr::eq(MutexGuard::mutex(&guard), &mutex));

    let mapped = MutexGuard::map(guard, |value| &mut value.0);
    let mut nested = MappedMutexGuard::map(mapped, |value| &mut value[1]);
    *nested = 9;
    assert!(mutex.try_lock().is_err());
    drop(nested);

    assert_eq!(mutex.try_lock().unwrap().0.as_slice(), &[1, 9]);
}

#[test]
fn mapped_try_map_failure_returns_the_same_live_guard() {
    let mutex = Mutex::new(vec![10usize]);
    let guard = MutexGuard::map(mutex.try_lock().unwrap(), |value| value.as_mut_slice());
    let mut guard = MappedMutexGuard::try_map(guard, |_| None::<&mut usize>).unwrap_err();
    guard[0] = 11;
    assert!(mutex.try_lock().is_err());
    drop(guard);
    assert_eq!(mutex.into_inner(), vec![11]);
}

#[tokio::test]
async fn owned_guards_retain_arc_and_preserve_projection() {
    let mutex = Arc::new(Mutex::new((vec![4usize, 5], 6usize)));
    let weak = Arc::downgrade(&mutex);
    let guard = mutex.clone().lock_owned().await;
    assert!(Arc::ptr_eq(OwnedMutexGuard::mutex(&guard), &mutex));
    drop(mutex);

    let guard = OwnedMutexGuard::try_map(guard, |_| None::<&mut Vec<usize>>).unwrap_err();
    let mapped = OwnedMutexGuard::map(guard, |value| &mut value.0);
    let mut nested = OwnedMappedMutexGuard::map(mapped, |value| &mut value[0]);
    *nested = 7;
    let mutex = weak.upgrade().unwrap();
    drop(nested);

    assert_eq!(mutex.lock().await.0.as_slice(), &[7, 5]);
}

#[test]
fn owned_try_lock_and_owned_mapped_try_map_release_once() {
    let mutex = Arc::new(Mutex::new(vec![1usize]));
    let guard = mutex.clone().try_lock_owned().unwrap();
    assert!(mutex.clone().try_lock_owned().is_err());
    let mapped = OwnedMutexGuard::map(guard, |value| value.as_mut_slice());
    let mapped = OwnedMappedMutexGuard::try_map(mapped, |_| None::<&mut usize>).unwrap_err();
    assert!(mutex.try_lock().is_err());
    drop(mapped);
    assert!(mutex.try_lock().is_ok());
}

#[test]
fn value_ownership_surfaces_are_exact() {
    let mut from = Mutex::from(String::from("from"));
    from.get_mut().push_str("-mut");
    assert_eq!(from.into_inner(), "from-mut");

    let defaulted = Mutex::<Vec<u8>>::default();
    assert!(defaulted.into_inner().is_empty());
}

#[test]
fn blocking_borrowed_and_owned_surfaces_release() {
    let mutex = Mutex::new(1usize);
    *mutex.blocking_lock() = 2;
    assert_eq!(*mutex.try_lock().unwrap(), 2);

    let mutex = Arc::new(Mutex::new(3usize));
    *mutex.clone().blocking_lock_owned() = 4;
    assert_eq!(*mutex.try_lock().unwrap(), 4);
}

#[test]
fn error_and_formatting_surfaces_match_locked_state() {
    let mutex = Mutex::new(12usize);
    assert_eq!(format!("{mutex:?}"), "Mutex { data: 12 }");
    let guard = mutex.try_lock().unwrap();
    assert_eq!(format!("{mutex:?}"), "Mutex { data: <locked> }");
    assert_eq!(format!("{guard:?}"), "12");
    assert_eq!(format!("{guard}"), "12");
    let error = mutex.try_lock().unwrap_err();
    assert_eq!(error.to_string(), "operation would block");
    assert!(error.source().is_none());
}
