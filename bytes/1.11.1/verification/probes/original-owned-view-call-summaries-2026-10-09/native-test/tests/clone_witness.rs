use bytes_owned_view_call_summaries_native::owned_view_call_scope;
use std::collections::BTreeSet;

#[test]
fn clone_witness() {
    let mut cases = 0;
    for len in [1usize, 2, 7, 31] {
        let input: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 251) as u8).collect();
        let ranges: BTreeSet<_> = [(0, len), (len / 2, len), (len - 1, len), (0, 1)].into_iter().collect();
        for (a, b) in ranges {
            let span = b - a;
            let sequences: BTreeSet<_> = [vec![], vec![0], vec![span], vec![span + 1], vec![usize::MAX], vec![1; span + 1], vec![0, 1, usize::MAX, 3]].into_iter().collect();
            for steps in sequences {
                let mut consumed = 0;
                for &step in &steps {
                    consumed += step.min(span - consumed);
                }
                assert_eq!(owned_view_call_scope(input.clone().into_boxed_slice(), a, b, &steps), input[a + consumed..b]);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 84);
    println!("modular owned-view native cases: {cases}");
}
