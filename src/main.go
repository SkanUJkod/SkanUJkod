package main

import "fmt"

func add(x int, y int) int {
	return x + y
}
func add2(x int, y int) int {
	return x + y
}
func add3(x int, y int) int {
	return x + y
}
func main() {
	var a = 1
	var b = 2
	var c = add(a, b)
	{
		var x = 3
		fmt.Println("x is", x)
	}
	if c > 0 {
		fmt.Println("The sum is positive")
	} else if c < 0 {
		fmt.Println("The sum is negative")
	} else {
		fmt.Println("The sum is zero")
	}
	fmt.Println("The sum of", a, "and", b, "is", c)
	fmt.Println("hello world")
}
