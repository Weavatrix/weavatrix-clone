fn summarize(samples: &[u64]) -> u64 {
    let mut total = 0_u64;
    let mut accepted = 0_u64;
    for sample in samples {
        if *sample > 11 && *sample < 9000 {
            total = total.saturating_add(*sample);
            accepted += 1;
        }
    }
    if accepted == 0 {
        return 0;
    }
    total / accepted
}
