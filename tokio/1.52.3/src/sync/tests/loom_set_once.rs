use crate::sync::SetOnce;

use loom::future::block_on;
use loom::sync::atomic::AtomicU32;
use loom::thread;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Clone)]
struct DropCounter {
    pub drops: Arc<AtomicU32>,
}

impl DropCounter {
    pub fn new() -> Self {
        Self {
            drops: Arc::new(AtomicU32::new(0)),
        }
    }

    fn assert_num_drops(&self, value: u32) {
        assert_eq!(value, self.drops.load(Ordering::Relaxed));
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn set_once_drop_test() {
    loom::model(|| {
        let set_once = Arc::new(SetOnce::new());
        let set_once_clone = Arc::clone(&set_once);

        let drop_counter = DropCounter::new();
        let counter_cl = drop_counter.clone();

        let thread = thread::spawn(move || set_once_clone.set(counter_cl).is_ok());

        let foo = drop_counter.clone();

        let set = set_once.set(foo).is_ok();
        let res = thread.join().unwrap();

        drop(set_once);

        drop_counter.assert_num_drops(2);
        assert!(res != set);
    });
}

#[test]
fn set_once_wait_test() {
    loom::model(|| {
        let tx = Arc::new(SetOnce::new());
        let rx_one = tx.clone();
        let rx_two = tx.clone();

        let thread = thread::spawn(move || {
            assert!(rx_one.set(2).is_ok());
        });

        block_on(async {
            assert_eq!(*rx_two.wait().await, 2);
        });

        thread.join().unwrap();
    });
}

#[test]
fn set_once_get_publication_test() {
    loom::model(|| {
        let cell = Arc::new(SetOnce::new());
        let writer = Arc::clone(&cell);

        let thread = thread::spawn(move || {
            writer.set((0x1234_u32, 0x5678_u32)).unwrap();
        });

        // Observe through `get`, rather than through `join`, so the Release
        // store and Acquire load in SetOnce are the publication edge checked
        // by loom.
        loop {
            if let Some(value) = cell.get() {
                assert_eq!(*value, (0x1234, 0x5678));
                break;
            }
            thread::yield_now();
        }

        thread.join().unwrap();
    });
}

#[test]
fn set_once_three_writers_test() {
    loom::model(|| {
        let cell = Arc::new(SetOnce::new());
        let first = Arc::clone(&cell);
        let second = Arc::clone(&cell);

        let first_thread = thread::spawn(move || first.set(10).is_ok());
        let second_thread = thread::spawn(move || second.set(20).is_ok());
        let third = cell.set(30).is_ok();

        let first = first_thread.join().unwrap();
        let second = second_thread.join().unwrap();
        assert_eq!(first as usize + second as usize + third as usize, 1);

        let value = *cell.get().unwrap();
        assert!(value == 10 || value == 20 || value == 30);
        assert_eq!(value == 10, first);
        assert_eq!(value == 20, second);
        assert_eq!(value == 30, third);
    });
}
