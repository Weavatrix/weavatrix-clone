final class NegativeA {
    static long rollingBits(long[] samples) {
        long state = 0x5a5a;
        int index = 0;
        while (index < samples.length) {
            state = Long.rotateLeft(state, 7);
            state ^= samples[index];
            if ((state & 1) == 0) {
                state += index;
            }
            index++;
        }
        return state;
    }
}
