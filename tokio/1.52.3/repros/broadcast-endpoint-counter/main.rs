use std::sync::atomic::{AtomicU16, AtomicU8, Ordering::SeqCst};
use std::sync::{Arc, Barrier};
use std::thread;

const INITIAL_ENDPOINTS: u8 = 254;
const OWNER_LIMIT: u16 = u8::MAX as u16;
const CALLERS: usize = 2;

/// Models the production order:
/// logical counter increment -> suspension point -> owner-token creation.
fn demonstrate_bad_ordering() {
    let logical = AtomicU8::new(INITIAL_ENDPOINTS);
    let owners = AtomicU16::new(INITIAL_ENDPOINTS as u16);

    // Both workers and the observing main thread participate in both barriers.
    let at_midpoint = Arc::new(Barrier::new(CALLERS + 1));
    let release_midpoint = Arc::new(Barrier::new(CALLERS + 1));

    thread::scope(|scope| {
        for _ in 0..CALLERS {
            let at_midpoint = Arc::clone(&at_midpoint);
            let release_midpoint = Arc::clone(&release_midpoint);
            let logical = &logical;
            let owners = &owners;

            scope.spawn(move || {
                logical.fetch_add(1, SeqCst);

                at_midpoint.wait();
                release_midpoint.wait();

                // This is the delayed Arc::clone/owner-token creation.
                owners.fetch_add(1, SeqCst);
            });
        }

        // Both logical increments have happened; neither owner exists yet.
        at_midpoint.wait();
        assert_eq!(logical.load(SeqCst), 0, "the reduced counter must wrap");
        assert_eq!(owners.load(SeqCst), 254, "owner creation must be paused");
        assert_ne!(
            u16::from(logical.load(SeqCst)),
            owners.load(SeqCst),
            "logical and physical ownership accounting must disagree"
        );
        release_midpoint.wait();
    });

    assert_eq!(logical.load(SeqCst), 0);
    assert_eq!(owners.load(SeqCst), 256);
}

/// Atomically obtains one slot from a deliberately finite owner-token pool.
fn try_reserve_owner(owners: &AtomicU16) -> bool {
    let mut observed = owners.load(SeqCst);
    loop {
        if observed >= OWNER_LIMIT {
            return false;
        }

        match owners.compare_exchange_weak(observed, observed + 1, SeqCst, SeqCst) {
            Ok(_) => return true,
            Err(current) => observed = current,
        }
    }
}

/// Models the fixed order: reserve owner token -> logical counter increment.
fn demonstrate_fixed_ordering() {
    let logical = AtomicU8::new(INITIAL_ENDPOINTS);
    let owners = AtomicU16::new(INITIAL_ENDPOINTS as u16);
    let accepted = AtomicU8::new(0);

    thread::scope(|scope| {
        for _ in 0..CALLERS {
            let logical = &logical;
            let owners = &owners;
            let accepted = &accepted;

            scope.spawn(move || {
                if try_reserve_owner(owners) {
                    // A successful return represents transferring this reserved
                    // owner token into the newly created endpoint handle.
                    let previous = logical.fetch_add(1, SeqCst);
                    assert!(previous < u8::MAX);
                    accepted.fetch_add(1, SeqCst);
                }
            });
        }
    });

    assert_eq!(accepted.load(SeqCst), 1);
    assert_eq!(owners.load(SeqCst), OWNER_LIMIT);
    assert_eq!(logical.load(SeqCst), u8::MAX);
    assert_eq!(u16::from(logical.load(SeqCst)), owners.load(SeqCst));
}

fn main() {
    demonstrate_bad_ordering();
    demonstrate_fixed_ordering();
    println!("ok: bad ordering wrapped without owners; fixed ordering rejected before wrap");
}
