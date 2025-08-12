def increment(x)
    x + 1
end

i = 0
while i < 10_000_000
    i = increment(i)
end
