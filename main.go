package main

import "fmt"

func add(x int, y int) int {
	return x + y
}

type person struct {
	name string
	age  int
}

func main() {
	var a = 1
	var b = 2
	fmt.Println("The sum of", a, "and", b)
	fmt.Println("hello world")
}
