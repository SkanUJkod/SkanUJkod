package main

import "fmt"

func onlyUnreachable() {
    // fmt.Println("never") 
    return
    fmt.Println("never") 
}


// func nestedIf(x, y int) {
//     if x > 0 {
//         fmt.Println("x")
//         if y > 0 {
//             fmt.Println("y") 
//         }
//     }
// }

func main() {
}