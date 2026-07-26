final class NegativeB {
    static String labels(long[] samples) {
        StringBuilder text = new StringBuilder();
        for (long sample : samples) {
            if (sample < 0) {
                text.append('N');
            } else if (sample == 0) {
                text.append('Z');
            } else {
                text.append('P');
            }
        }
        text.reverse();
        return text.toString();
    }
}
