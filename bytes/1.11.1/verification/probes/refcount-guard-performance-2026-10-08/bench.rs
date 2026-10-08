//! Native atomic microbenchmark; includes the exact production guard/helper.
//! Not a whole-Bytes or platform-independent performance conclusion.
mod loom { pub mod sync { pub mod atomic { pub use std::sync::atomic::*; } } }
#[path = "../../../src/ref_count_limit.rs"] mod ref_count_limit;
#[path = "../../../src/ref_count_ops.rs"] mod ref_count_ops;
fn abort() -> ! { std::process::abort() }
use std::{hint::black_box, sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}}, time::Instant};
fn trial(guarded: bool, threads: usize, iterations: usize) -> f64 {
    let count = Arc::new(AtomicUsize::new(1));
    let start = Arc::new(Barrier::new(threads + 1));
    let mut workers = Vec::new();
    for _ in 0..threads {
        let count = count.clone(); let start = start.clone();
        workers.push(std::thread::spawn(move || {
            start.wait();
            for _ in 0..iterations {
                let c = black_box(&*count);
                if guarded { ref_count_ops::increment(c); }
                else { if c.fetch_add(1, Ordering::Relaxed) > ref_count_limit::MAX_REF_COUNT { abort(); } }
                black_box(c.fetch_sub(1, Ordering::Release));
            }
        }));
    }
    let now = Instant::now(); start.wait();
    for worker in workers { worker.join().unwrap(); }
    let elapsed = now.elapsed().as_secs_f64() * 1e9 / (threads * iterations) as f64;
    assert_eq!(count.load(Ordering::Relaxed), 1);
    elapsed
}
fn main() {
    println!("native increment + Release decrement, ns/pair; 7 alternating trials");
    for threads in [1, 4] {
        let mut old = Vec::new(); let mut new = Vec::new();
        for trial_index in 0..7 {
            let order = if trial_index % 2 == 0 { [false, true] } else { [true, false] };
            for guarded in order {
                let ns = trial(guarded, threads, 1_000_000);
                println!("threads={threads} trial={trial_index} guarded={guarded} ns={ns:.3}");
                if guarded { new.push(ns); } else { old.push(ns); }
            }
        }
        old.sort_by(f64::total_cmp); new.sort_by(f64::total_cmp);
        println!("median threads={threads} baseline={:.3} guarded={:.3} ratio={:.3}", old[3], new[3], new[3]/old[3]);
    }
}
