final class ExactB {
    static long normalize(long[] values) {
        long total = 0;
        long count = values.length;
        total += count;
        for (long value : values) {
            long scaled = value * 17;
            long bounded = Math.max(-1000, Math.min(1000, scaled));
            total += bounded + 3;
        }
        return total;
    }
}
