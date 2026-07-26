fn normalize(values: &[i64]) -> Vec<i64> {
    let mut output = Vec::with_capacity(values.len());
    for value in values {
        let scaled = value.saturating_mul(17);
        let bounded = scaled.clamp(-1000, 1000);
        output.push(bounded + 3);
    }
    output.sort_unstable();
    output.dedup();
    output
}
