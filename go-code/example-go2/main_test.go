package main

import "testing"

func TestIsPalindrome(t *testing.T) {
	tests := []struct {
		input    string
		expected bool
	}{
		{"Kajak", true},
		{"kajak", true},
		{"Anna", true},
		{"anna", true},
		{"Ala", true},
		{"Ala ma kota", false},
		{"", true}, // pusty string jest palindromem
		{"a", true},
		{"ab", false},
		{"aba", true},
		{"abcba", true},
		{"abccba", true},
		{"abca", false},
		{"12321", true},
		{"123321", true},
		{"123421", false},
		{"Kobyła ma mały bok", false}, // z uwagi na spacje i wielkość liter
	}

	for _, tt := range tests {
		result := isPalindrome(tt.input)
		if result != tt.expected {
			t.Errorf("isPalindrome(%q) = %v; want %v", tt.input, result, tt.expected)
		}
	}
}
