local function increment(x)
    return x + 1
end

local i = 0
while i < 10000000 do
    i = increment(i)
end
