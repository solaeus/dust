class AddictiveCalling {
    static i64 increment(i64 x) {
        return x + 1;
    }

    public static void main(String[] args) {
        i64 i = 0;
        while (i < 10_000_000) {
            i = increment(i);
        }
    }
}
