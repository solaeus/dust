function increment(x) {
  return x + 1;
}

let i = 0;
while (i < 10000000) {
  i = increment(i);
}
