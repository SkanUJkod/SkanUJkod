package main

import (
    "testing"
)

// func TestCalculateGrade(t *testing.T) {
//     tests := []struct {
//         score    int
//         expected string
//     }{
//         {95, "A"},
//         {85, "B"},
//         {75, "C"},
//         {65, "D"},
//         {55, "F"},
//     }
    
//     for _, tt := range tests {
//         result := calculateGrade(tt.score)
//         if result != tt.expected {
//             t.Errorf("calculateGrade(%d) = %s; want %s", 
//                 tt.score, result, tt.expected)
//         }
//     }
// }

func TestSumArray(t *testing.T) {
    tests := []struct {
        numbers  []int
        expected int
    }{
        {[]int{1, 2, 3, 4, 5}, 15},
        {[]int{}, 0},
        {[]int{-1, -2, 3}, 0},
        {[]int{10}, 10},
    }
    
    for _, tt := range tests {
        result := sumArray(tt.numbers)
        if result != tt.expected {
            t.Errorf("sumArray(%v) = %d; want %d", 
                tt.numbers, result, tt.expected)
        }
    }
}

func TestFindMax(t *testing.T) {
    tests := []struct {
        numbers  []int
        expected int
    }{
        {[]int{3, 7, 2, 9, 1}, 9},
        {[]int{}, 0},
        {[]int{-5, -2, -10}, -2},
        {[]int{42}, 42},
    }
    
    for _, tt := range tests {
        result := findMax(tt.numbers)
        if result != tt.expected {
            t.Errorf("findMax(%v) = %d; want %d", 
                tt.numbers, result, tt.expected)
        }
    }
}

func TestGetDayName(t *testing.T) {
    tests := []struct {
        day      int
        expected string
    }{
        {1, "Monday"},
        {5, "Friday"},
        {7, "Sunday"},
        {0, "Invalid day"},
        {8, "Invalid day"},
    }
    
    for _, tt := range tests {
        result := getDayName(tt.day)
        if result != tt.expected {
            t.Errorf("getDayName(%d) = %s; want %s", 
                tt.day, result, tt.expected)
        }
    }
}

func TestDivide(t *testing.T) {
    // Test normal division
    result, err := divide(10, 2)
    if err != nil {
        t.Errorf("divide(10, 2) returned error: %v", err)
    }
    if result != 5 {
        t.Errorf("divide(10, 2) = %f; want 5", result)
    }
    
    // Test division by zero
    _, err = divide(10, 0)
    if err == nil {
        t.Error("divide(10, 0) should return error")
    }
}

func TestMinMax(t *testing.T) {
    tests := []struct {
        numbers []int
        min     int
        max     int
    }{
        {[]int{5, 2, 8, 1, 9}, 1, 9},
        {[]int{}, 0, 0},
        {[]int{42}, 42, 42},
        {[]int{-5, -2, -10}, -10, -2},
    }
    
    for _, tt := range tests {
        min, max := minMax(tt.numbers)
        if min != tt.min || max != tt.max {
            t.Errorf("minMax(%v) = (%d, %d); want (%d, %d)", 
                tt.numbers, min, max, tt.min, tt.max)
        }
    }
}

func TestProcessFile(t *testing.T) {
    err := processFile("test.txt")
    if err != nil {
        t.Errorf("processFile(\"test.txt\") returned error: %v", err)
    }

    err = processFile("")
    if err == nil {
        t.Error("processFile(\"\") should return error")
    }
}

func TestComplexFlow(t *testing.T) {
    tests := []struct {
        n        int
        expected int
    }{
        {5, 10},
        {0, 0},
        {1, 0},
        {10, 45},
    }

    for _, tt := range tests {
        result := complexFlow(tt.n)
        if result != tt.expected {
            t.Errorf("complexFlow(%d) = %d; want %d", 
                tt.n, result, tt.expected)
        }
    }
}

// Benchmark for performance testing
func BenchmarkSumArray(b *testing.B) {
    numbers := make([]int, 1000)
    for i := range numbers {
        numbers[i] = i
    }
    
    b.ResetTimer()
    for i := 0; i < b.N; i++ {
        sumArray(numbers)
    }
}
// Tests for deadCodeExample
func TestDeadCodeExample(t *testing.T) {
    // The 'false' branch is never executed; only the else path
    if got := deadCodeExample(3); got != 6 {
        t.Errorf("deadCodeExample(3) = %d; want 6", got)
    }
}
