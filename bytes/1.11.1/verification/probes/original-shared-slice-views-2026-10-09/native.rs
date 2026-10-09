use bytes::Bytes;

pub fn nested_slice_scope(input: Box<[u8]>, a: usize, b: usize, c: usize, d: usize) -> Vec<u8> {
    let selected = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        let first = owner.slice(a..b);
        first.slice(c..d)
    };
    let observed = AsRef::<[u8]>::as_ref(&selected).to_vec();
    observed
}
