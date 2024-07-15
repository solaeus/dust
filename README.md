# Dust

High-level programming language with effortless concurrency, automatic memory management, type
safety and familiar syntax.

```dust
io.write_line("Guess the number.")

secret_number = random.range(0..100)

loop {
  	io.write_line("Input your guess.")

  	input = io.read_line()
  	guess = int.parse(input)

  	if guess < secret_number {
  	  	io.write_line("Too low!")
	  } else if guess > secret_number {
	    	io.write_line("Too high!")
	  } else {
	    	io.write_line("You win!")
	    	break
	  }
}
```
