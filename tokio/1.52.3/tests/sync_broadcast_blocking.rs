#![warn(rust_2018_idioms)]
#![cfg(all(feature = "sync", not(target_family = "wasm")))]

use tokio::sync::broadcast;

#[test]
fn blocking_recv_immediate_value_and_closed() {
    let (tx, mut rx) = broadcast::channel(2);
    tx.send(7).unwrap();
    assert_eq!(rx.blocking_recv(), Ok(7));
    drop(tx);
    assert_eq!(rx.blocking_recv(), Err(broadcast::error::RecvError::Closed));
}

#[test]
fn blocking_recv_parks_until_send() {
    let (tx, mut rx) = broadcast::channel(2);
    let receiver = std::thread::spawn(move || rx.blocking_recv());
    tx.send(11).unwrap();
    assert_eq!(receiver.join().unwrap(), Ok(11));
}

#[test]
fn blocking_recv_preserves_lagged_count() {
    let (tx, mut rx) = broadcast::channel(2);
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    assert_eq!(
        rx.blocking_recv(),
        Err(broadcast::error::RecvError::Lagged(1))
    );
    assert_eq!(rx.blocking_recv(), Ok(2));
}
