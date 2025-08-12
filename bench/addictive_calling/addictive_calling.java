class AddictiveCalling {
    static int increment(int x) {
        return x + 1;
    }

    public static void main(String[] args) {
        int i = 0;
        while (i < 10_000_000) {
            i = increment(i);
        }
    }
}
