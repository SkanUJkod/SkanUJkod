package main

import (
	"fmt"
	"strings"
)

// Dodaje dwie liczby całkowite
func add(a, b int) int {
	return a + b
}

// Sprawdza, czy tekst jest palindromem
func isPalindrome(s string) bool {
	s = strings.ToLower(s)
	runes := []rune(s)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		if runes[i] != runes[j] {
			return false
		}
	}
	return true
}

// Zwraca największy element w tablicy liczb całkowitych
func max(nums []int) int {
	if len(nums) == 0 {
		panic("pusta tablica")
	}
	maxVal := nums[0]
	for _, v := range nums {
		if v > maxVal {
			maxVal = v
		}
	}
	return maxVal
}

func main() {
	fmt.Println("Dodawanie 3 + 5 =", add(3, 5))
	fmt.Println("Czy 'Kajak' to palindrom?", isPalindrome("Kajak"))
	fmt.Println("Największa liczba w [1, 7, 3, 9, 2]:", max([]int{1, 7, 3, 9, 2}))
}
