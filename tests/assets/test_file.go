package main

import "fmt"

func complexFunction(n int) int {
    if n <= 0 {
        return 0
    }
    
    if n == 1 {
        return 1
    }
    
    result := 0
    for i := 0; i < n; i++ {
        if i%2 == 0 {
            result += i
        } else {
            result -= i
        }
        
        if i > 5 {
            break
        }
    }
    
    return result
}

func main() {
    fmt.Println(complexFunction(10))
}
