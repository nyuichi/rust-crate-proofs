use bytes_owned_view_clone_native::owned_view_clone_scope;
use std::collections::BTreeSet;

#[test]
fn clone_witness() {
    let mut cases = 0;
    for len in [1usize, 2, 7, 31] {
        let input: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let ranges: BTreeSet<_> = [(0, len), (len / 2, len), (len - 1, len), (0, 1)].into_iter().collect();
        for (a, b) in ranges {
            let advances: BTreeSet<_> = [0, (b - a) / 2, b - a].into_iter().collect();
            for advance_by in advances {
                for rounds in [0, 1, 2, 7, 31] {
                    assert_eq!(owned_view_clone_scope(input.clone().into_boxed_slice(), a, b, advance_by, rounds), input[a + advance_by..b]);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 145);
    println!("owned-view Clone native cases: {cases}");
}
