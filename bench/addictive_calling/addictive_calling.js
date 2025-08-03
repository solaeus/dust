function increment(x) {
  return x + 1;
}

let i = 0;
while (i < 1000000) {
  i = increment(i);
}
