increment <- function(x) {
  x + 1
}

i <- 0

while (i < 1000000) {
  i <- increment(i)
}
