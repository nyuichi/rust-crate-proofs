use bytes_root_phase_clone_native::root_clone_steps;

#[test]
fn arbitrary_step_samples_return_exact_capped_suffix() {
    let mut cases = 0usize;
    for len in [1usize, 2, 3, 7, 16, 257] {
        let input: Vec<u8> = (0..len).map(|i| (i.wrapping_mul(37) % 256) as u8).collect();
        let patterns = [
            vec![], vec![0], vec![1], vec![len], vec![len + 1], vec![usize::MAX],
            vec![0, 0, 0], vec![1, 1, 1], vec![len, 0, usize::MAX],
            vec![usize::MAX, usize::MAX], vec![0, len, 0], vec![1, usize::MAX, 1, 0],
            vec![0; 64], vec![1; len + 2], vec![len / 2, len / 2, 1],
        ];
        for steps in patterns {
            let mut offset = 0usize;
            for step in &steps { offset += (*step).min(len - offset); }
            let actual = root_clone_steps(input.clone().into_boxed_slice(), &steps);
            assert_eq!(actual, input[offset..], "len={len}, steps={steps:?}");
            cases += 1;
        }
    }
    assert_eq!(cases, 90);
    println!("AZ native step-pattern cases: {cases}");
}
