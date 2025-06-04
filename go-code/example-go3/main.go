package main

import (
	"fmt"
)

func deeplyNestedIfs(x int) {
	if x > 0 {
		fmt.Println("x is positive")
		if x > 10 {
			fmt.Println("x is greater than 10")
			if x%2 == 0 {
				fmt.Println("x is even")
				if x < 100 {
					fmt.Println("x is less than 100")
					if x%5 == 0 {
						fmt.Println("x is divisible by 5")
						if x > 50 {
							fmt.Println("x is greater than 50")
							if x < 80 {
								fmt.Println("x is less than 80")
								if x%3 == 0 {
									fmt.Println("x is divisible by 3")
									if x > 60 {
										fmt.Println("x is greater than 60")
										if x < 70 {
											fmt.Println("x is less than 70")
										}
									}
								}
							}
						}
					}
				}
			}
		}
	} else {
		fmt.Println("x is not positive")
	}
}

func main() {
	deeplyNestedIfs(66)
}
