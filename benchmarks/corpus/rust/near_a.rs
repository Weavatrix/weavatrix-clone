fn risk_score(events: &[i64]) -> i64 {
    let mut score = 7_i64;
    for event in events {
        let weighted = event.saturating_mul(5);
        score = score.saturating_add(weighted);
        if score > 50_000 {
            score /= 2;
        }
    }
    score.clamp(-75_000, 75_000)
}
