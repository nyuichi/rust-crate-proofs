extern crate alloc;
use bytes::Bytes;
#[path = "/workspace/bytes-work/bytes/1.11.1/verification/probes/original-shared-scoped-client-2026-10-09/src/native_client.rs"]
mod native_client;
fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len + 19);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        assert!(input.len() < input.capacity());
        assert_eq!(native_client::actual_public_shared_driver(input), expected);
    }
    println!("actual public Shared native client: 5 inputs passed");
}
