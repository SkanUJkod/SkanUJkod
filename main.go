package main

type type1 struct {
	ID   int
	Name string
}
type type2 struct {
	ID   int
	Name string
}
type type3 struct {
	ID   int
	Name string
}

func foo1() {
	print("foo1")
}

func foo2() {
	print("foo2")
}
func foo3() {
	print("foo3")
}
func foo4() {
	print("foo4")
}

func foo_4() {
	print("foo_4")
}

func main() {
	// Example usage
	var t1 type1
	var t2 type2
	var t3 type3
	var num1 = 4
	var num2 = 5
	print(t1, t2, t3, num1, num2)
	if num1 > num2 {
		foo1()
	}

}
