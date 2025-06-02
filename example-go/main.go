package main

import (
    "fmt"
)

func calculateGrade(score int) string {
    if score >= 90 {
        return "A"
    } else if score >= 80 {
        return "B"
    } else if score >= 70 {
        return "C"
    } else if score >= 60 {
        return "D"
    }
    return "F"
}

func sumArray(numbers []int) int {
    total := 0
    for i := 0; i < len(numbers); i++ {
        total += numbers[i]
    }
    return total
}

func findMax(numbers []int) int {
    if len(numbers) == 0 {
        return 0
    }
    
    max := numbers[0]
    for _, num := range numbers {
        if num > max {
            max = num
        }
    }
    return max
}

func getDayName(day int) string {
    switch day {
    case 1:
        return "Monday"
    case 2:
        return "Tuesday"
    case 3:
        return "Wednesday"
    case 4:
        return "Thursday"
    case 5:
        return "Friday"
    case 6:
        return "Saturday"
    case 7:
        return "Sunday"
    default:
        return "Invalid day"
    }
}

func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, fmt.Errorf("division by zero")
    }
    return a / b, nil
}


func minMax(numbers []int) (int, int) {
    if len(numbers) == 0 {
        return 0, 0
    }
    
    min, max := numbers[0], numbers[0]
    for _, n := range numbers {
        if n < min {
            min = n
        }
        if n > max {
            max = n
        }
    }
    return min, max
}

func processFile(filename string) error {
    fmt.Printf("Opening file: %s\n", filename)
    defer fmt.Println("File processing completed")
    
    if filename == "" {
        return fmt.Errorf("empty filename")
    }
    
    fmt.Println("Processing...")
    return nil
}

func complexFlow(n int) int {
    result := 0
    i := 0
    
loop:
    if i >= n {
        goto end
    }
    result += i
    i++
    goto loop
    
end:
    return result
}

func deadCodeExample(x int) int {
    if false {
        return -1
    } else {
        return x * 2
    }

    y := x + 100
    return y
}


func main() {
    fmt.Println("Grade for 85:", calculateGrade(85))
    fmt.Println("Sum:", sumArray([]int{1, 2, 3, 4, 5}))
    fmt.Println("Max:", findMax([]int{3, 7, 2, 9, 1}))
    fmt.Println("Day 3:", getDayName(3))
    
    if result, err := divide(10, 2); err == nil {
        fmt.Println("10/2 =", result)
    }
    
    min, max := minMax([]int{5, 2, 8, 1, 9})
    fmt.Printf("Min: %d, Max: %d\n", min, max)
    
    processFile("test.txt")
}