def increment(x)
    x + 1
end

i = 0
while i < 1_000_000
    i = increment(i)
end
