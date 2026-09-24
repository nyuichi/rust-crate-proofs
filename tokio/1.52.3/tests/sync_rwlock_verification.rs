#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::sync::Arc;
use tokio::sync::{
    OwnedRwLockMappedWriteGuard, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock,
    RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use tokio_test::task::spawn;
use tokio_test::{assert_pending, assert_ready};

#[test]
fn writer_preference_and_cancelled_writer_loses_place() {
    let lock = RwLock::with_max_readers(0usize, 2);
    let first_reader = lock.try_read().unwrap();

    let mut writer = spawn(lock.write());
    let mut later_reader = spawn(lock.read());
    assert_pending!(writer.poll());
    assert_pending!(later_reader.poll());

    drop(writer);
    assert!(later_reader.is_woken());
    let second_reader = assert_ready!(later_reader.poll());
    assert!(lock.try_write().is_err());
    drop(first_reader);
    drop(second_reader);
    assert!(lock.try_write().is_ok());
}

#[test]
fn max_reader_permits_and_try_paths_are_exact() {
    let lock = RwLock::with_max_readers(1usize, 2);
    let first = lock.try_read().unwrap();
    let second = lock.try_read().unwrap();
    assert!(lock.try_read().is_err());
    assert!(lock.try_write().is_err());
    drop(first);
    assert!(lock.try_read().is_ok());
    drop(second);

    let write = lock.try_write().unwrap();
    assert!(lock.try_read().is_err());
    assert!(lock.try_write().is_err());
    drop(write);
    assert!(lock.try_write().is_ok());
}

#[test]
fn borrowed_read_maps_preserve_one_reader_permit() {
    let lock = RwLock::new((vec![1usize, 2], 3usize));
    let guard = lock.try_read().unwrap();
    let guard = RwLockReadGuard::try_map(guard, |_| None::<&Vec<usize>>).unwrap_err();
    let mapped = RwLockReadGuard::map(guard, |value| &value.0);
    let mapped = RwLockReadGuard::map(mapped, |value| &value[1]);
    assert_eq!(*mapped, 2);
    assert!(lock.try_write().is_err());
    assert!(lock.try_read().is_ok());
    drop(mapped);
}

#[test]
fn borrowed_write_maps_preserve_all_permits_until_drop() {
    let lock = RwLock::new((vec![1usize, 2], 3usize));
    let guard = lock.try_write().unwrap();
    let guard = RwLockWriteGuard::try_map(guard, |_| None::<&mut Vec<usize>>).unwrap_err();
    let mapped = RwLockWriteGuard::map(guard, |value| &mut value.0);
    let mapped = RwLockMappedWriteGuard::try_map(mapped, |_| None::<&mut usize>).unwrap_err();
    let mut mapped = RwLockMappedWriteGuard::map(mapped, |value| &mut value[0]);
    *mapped = 7;
    assert!(lock.try_read().is_err());
    drop(mapped);
    assert_eq!(lock.try_read().unwrap().0.as_slice(), &[7, 2]);

    let mapped = RwLockWriteGuard::into_mapped(lock.try_write().unwrap());
    assert!(lock.try_read().is_err());
    drop(mapped);
}

#[test]
fn downgrade_retains_one_permit_and_prior_writer_stays_first() {
    let lock = RwLock::with_max_readers(5usize, 2);
    let write = lock.try_write().unwrap();
    let mut queued_writer = spawn(lock.write());
    let mut queued_reader = spawn(lock.read());
    assert_pending!(queued_writer.poll());
    assert_pending!(queued_reader.poll());

    let read = RwLockWriteGuard::downgrade(write);
    assert_pending!(queued_writer.poll());
    assert_pending!(queued_reader.poll());
    assert_eq!(*read, 5);
    drop(read);

    let writer = assert_ready!(queued_writer.poll());
    assert_pending!(queued_reader.poll());
    drop(writer);
    let _reader = assert_ready!(queued_reader.poll());
}

#[tokio::test]
async fn owned_read_write_map_and_downgrade_surfaces_retain_arc() {
    let lock = Arc::new(RwLock::new((vec![1usize, 2], 3usize)));
    let weak = Arc::downgrade(&lock);

    let read = lock.clone().read_owned().await;
    assert!(Arc::ptr_eq(OwnedRwLockReadGuard::rwlock(&read), &lock));
    let read = OwnedRwLockReadGuard::try_map(read, |_| None::<&Vec<usize>>).unwrap_err();
    let read = OwnedRwLockReadGuard::map(read, |value| &value.0[0]);
    assert_eq!(*read, 1);
    drop(read);

    let write = lock.clone().write_owned().await;
    assert!(Arc::ptr_eq(OwnedRwLockWriteGuard::rwlock(&write), &lock));
    let write = OwnedRwLockWriteGuard::try_downgrade_map(write, |_| None::<&usize>).unwrap_err();
    let read = OwnedRwLockWriteGuard::downgrade_map(write, |value| &value.1);
    assert_eq!(*read, 3);
    drop(read);

    let write = lock.clone().write_owned().await;
    let read = OwnedRwLockWriteGuard::downgrade(write);
    assert!(Arc::ptr_eq(OwnedRwLockReadGuard::rwlock(&read), &lock));
    drop(read);

    let write = lock.clone().write_owned().await;
    let mapped = OwnedRwLockWriteGuard::map(write, |value| &mut value.0);
    assert!(Arc::ptr_eq(
        OwnedRwLockMappedWriteGuard::rwlock(&mapped),
        &lock
    ));
    let mapped = OwnedRwLockMappedWriteGuard::try_map(mapped, |_| None::<&mut usize>).unwrap_err();
    let mut mapped = OwnedRwLockMappedWriteGuard::map(mapped, |value| &mut value[1]);
    *mapped = 8;
    drop(lock);
    let lock = weak.upgrade().unwrap();
    drop(mapped);
    assert_eq!(lock.read().await.0.as_slice(), &[1, 8]);

    let mapped = OwnedRwLockWriteGuard::into_mapped(lock.clone().write_owned().await);
    drop(mapped);
}

#[test]
fn owned_try_paths_preserve_exclusion_and_release() {
    let lock = Arc::new(RwLock::new(4usize));
    let read = lock.clone().try_read_owned().unwrap();
    assert!(lock.clone().try_write_owned().is_err());
    drop(read);
    let write = lock.clone().try_write_owned().unwrap();
    assert!(lock.clone().try_read_owned().is_err());
    drop(write);
    assert!(lock.try_read().is_ok());
}

#[test]
fn blocking_value_and_formatting_surfaces() {
    let mut lock = RwLock::from(String::from("value"));
    lock.get_mut().push_str("-mut");
    let write = lock.try_write().unwrap();
    assert_eq!(format!("{lock:?}"), "RwLock { data: <locked> }");
    assert_eq!(format!("{write:?}"), "\"value-mut\"");
    assert_eq!(format!("{write}"), "value-mut");
    drop(write);
    assert_eq!(format!("{lock:?}"), "RwLock { data: \"value-mut\" }");
    assert_eq!(&*lock.blocking_read(), "value-mut");
    lock.blocking_write().push('!');
    assert_eq!(lock.into_inner(), "value-mut!");

    let defaulted = RwLock::<Vec<u8>>::default();
    assert!(defaulted.into_inner().is_empty());
}

#[test]
fn constructor_rejects_above_production_max() {
    let too_many = (u32::MAX >> 3) + 1;
    let panic = std::panic::catch_unwind(|| RwLock::with_max_readers((), too_many));
    assert!(panic.is_err());
}
