final class RenamedB {
    static long checksum(long[] samples) {
        long state = 13;
        int index = 0;
        while (index < samples.length) {
            long rotated = Long.rotateLeft(samples[index], 3);
            state ^= rotated;
            state += index;
            index++;
        }
        return state;
    }
}
