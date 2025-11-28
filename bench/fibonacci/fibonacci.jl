function fib(n)
    if n <= 0
        return 0
    elseif n == 1
        return 1
    else
        return fib(n - 1) + fib(n - 2)
    end
end

fib(25)
