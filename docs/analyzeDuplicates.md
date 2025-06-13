## Duplicate Analyzer Guide

The `analyze_duplicates` function provides a robust and efficient solution for maintaining high-quality source code.

It programmatically scans for exact duplicated code sequences, helping developers:
* Identify and eliminate redundancy.
* Simplify codebase maintenance.
* Ensure consistency across projects.

### Parameters

- **`path: &str`**  
  Path to the Go file or directory to scan (e.g., `"main.go"` or `"my_project/"`).

- **`min_tokens: usize`**  
  Minimum number of **consecutive tokens** for a sequence to be considered a duplicate.  

- **`is_directory: bool`**  
  `true` if the path is a directory, `false` if it's a single file.

---

### Returns

- `Ok(String)` — Report listing duplicated code sequences (can be empty if none found).
- `Err(Box<dyn Error>)` — An error occurred (e.g., file not found, parsing failure).

---

### Example

```rust
fn main() {
    let path = "example.go"; // Path to a Go file or directory
    let min_tokens = 20;            // Token length threshold
    let is_directory = false;       // Set to true for directory analysis

    match analyze_duplicates(path, min_tokens, is_directory) {
        Ok(report) => println!("{}", report),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```
