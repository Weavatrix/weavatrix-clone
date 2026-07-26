final class RenamedA {
    static long checksum(long[] items) {
        long accumulator = 13;
        int position = 0;
        while (position < items.length) {
            long mixed = Long.rotateLeft(items[position], 3);
            accumulator ^= mixed;
            accumulator += position;
            position++;
        }
        return accumulator;
    }
}
