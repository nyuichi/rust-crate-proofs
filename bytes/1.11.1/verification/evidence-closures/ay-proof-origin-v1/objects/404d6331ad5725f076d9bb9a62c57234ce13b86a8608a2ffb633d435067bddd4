use bytes_root_phase_view_native::root_phase_scope;

#[test]
fn optional_promotion_peer_drop_common_operations_return_exact_suffix() {
    let mut cases = 0usize;
    for len in [1usize, 2, 3, 7, 16, 257] {
        let input: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(37) % 256) as u8).collect();
        for first in 0..=len {
            let remaining = len - first;
            let mut seconds = vec![0, remaining];
            if remaining > 0 { seconds.push(1); }
            seconds.sort_unstable();
            seconds.dedup();
            for second in seconds {
                for promote in [false, true] {
                    let expected = input[first + second..].to_vec();
                    let actual = root_phase_scope(input.clone().into_boxed_slice(), first, second, promote);
                    assert_eq!(actual, expected, "len={len}, first={first}, second={second}, promote={promote}");
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 1716);
    println!("AY native phase cases: {cases}");
}
