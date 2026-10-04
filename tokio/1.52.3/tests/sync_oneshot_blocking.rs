#![warn(rust_2018_idioms)]
#![cfg(all(feature = "sync", not(target_family = "wasm")))]

use tokio::sync::oneshot;

#[test]
fn blocking_recv_immediate_value_and_closed() {
    let (tx, rx) = oneshot::channel();
    tx.send(7).unwrap();
    assert_eq!(rx.blocking_recv(), Ok(7));

    let (tx, rx) = oneshot::channel::<u8>();
    drop(tx);
    assert_eq!(
        rx.blocking_recv().unwrap_err().to_string(),
        "channel closed"
    );
}

#[test]
fn blocking_recv_parks_until_send() {
    let (tx, rx) = oneshot::channel();
    let receiver = std::thread::spawn(move || rx.blocking_recv());
    tx.send(11).unwrap();
    assert_eq!(receiver.join().unwrap(), Ok(11));
}
