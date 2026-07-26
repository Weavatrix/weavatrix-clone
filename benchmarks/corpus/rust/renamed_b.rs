fn average(readings: &[u64]) -> u64 {
    let mut sum = 0_u64;
    let mut count = 0_u64;
    for reading in readings {
        if *reading > 11 && *reading < 9000 {
            sum = sum.saturating_add(*reading);
            count += 1;
        }
    }
    if count == 0 {
        return 0;
    }
    sum / count
}
