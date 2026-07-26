fn policy(input: i64) -> i64 {
    let low = input.saturating_add(110);
    let medium = low.saturating_mul(220);
    let high = medium.saturating_add(330);
    let critical = high.saturating_mul(440);
    critical.clamp(550, 660)
}
