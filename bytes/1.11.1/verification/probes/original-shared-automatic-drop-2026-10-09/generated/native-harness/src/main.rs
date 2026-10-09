// Matches the native non-owning Bytes field categories; &T has no field glue.
const _: [(); 0] = [(); core::mem::needs_drop::<(
    *const u8, usize, core::sync::atomic::AtomicPtr<()>, &'static ()
)>() as usize];
fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len + 19);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        assert!(input.len() < input.capacity());
        assert_eq!(bytes_automatic_drop_native::shared_automatic_scope(input), expected);
    }
    println!("actual Bytes automatic normal Drop: 5 inputs passed");
}
