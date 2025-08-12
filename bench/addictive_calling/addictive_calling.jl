function increment(x)
    x + 1
end

function addictive_calling()
    i = 0
    while i < 10_000_000
        i = increment(i)
    end
end

addictive_calling()
