final class NearB {
    static long riskScore(long[] events) {
        long score = 7;
        for (int index = 0; index < events.length; index++) {
            long event = events[index];
            if (event == 0) {
                continue;
            }
            long weighted = event * 5;
            score += weighted;
        }
        score ^= events.length;
        return Math.max(0, score);
    }
}
