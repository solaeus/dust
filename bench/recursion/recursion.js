function decrement(i) {
  if (i == 0) {
    return "Done!";
  }

  return decrement(i - 1);
}

console.log(decrement(10_000));
