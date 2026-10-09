// Matches the native non-owning Bytes field categories; &T has no field glue.
const _: [(); 0] = [(); core::mem::needs_drop::<(
    *const u8, usize, core::sync::atomic::AtomicPtr<()>, &'static ()
)>() as usize];
fn main() {
    for len in [0, 1, 7, 63, 1024] {
        let mut input = Vec::with_capacity(len);
        input.extend((0..len).map(|i| (i % 251) as u8));
        let expected = input.clone();
        let input=input.into_boxed_slice();
        assert_eq!(bytes_boxed_drop_native::boxed_read_then_drop(input), expected);
    }
    println!("actual boxed Bytes automatic normal Drop: 5 inputs passed");
}
