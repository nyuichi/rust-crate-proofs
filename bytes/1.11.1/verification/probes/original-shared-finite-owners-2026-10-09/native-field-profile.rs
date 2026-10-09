//! Compile-time check of the exact four default-native Bytes field types.
use core::sync::atomic::AtomicPtr;
const _: () = assert!(!core::mem::needs_drop::<*const u8>());
const _: () = assert!(!core::mem::needs_drop::<usize>());
const _: () = assert!(!core::mem::needs_drop::<AtomicPtr<()>>());
const _: () = assert!(!core::mem::needs_drop::<&'static ()>());
fn main() {
    println!("native Bytes fields have no independent drop glue");
}
