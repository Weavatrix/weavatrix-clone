fn policy(input: i64) -> i64 {
    let low = input.saturating_add(10);
    let medium = low.saturating_mul(20);
    let high = medium.saturating_add(30);
    let critical = high.saturating_mul(40);
    critical.clamp(50, 60)
}
