use std::collections::HashMap;
use std::error::Error;
use std::fs;
use walkdir::WalkDir;

/// Contains information about the location of a token.
struct TokenInfo {
    file_path: String,
    line_number: usize,
}

/// An improved tokenizer for Go that correctly handles strings and comments.
/// It processes the code line by line, maintaining state to correctly parse
/// multi-line comments and string literals.
fn tokenize_go(code: &str) -> Vec<(String, usize)> {
    let mut tokens = Vec::new();
    let mut current_line = 1;
    let mut in_multiline_comment = false;

    for line in code.lines() {
        let line_trimmed = line.trim();

        if line_trimmed.starts_with("/*") {
            in_multiline_comment = true;
        }

        if in_multiline_comment {
            if line_trimmed.contains("*/") {
                in_multiline_comment = false;
            }
            current_line += 1;
            continue;
        }

        if line_trimmed.starts_with("//") || line_trimmed.is_empty() {
            current_line += 1;
            continue;
        }

        let mut current_token = String::new();
        let mut in_string = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if in_string {
                current_token.push(c);
                if c == '"' {
                    tokens.push((current_token.clone(), current_line));
                    current_token.clear();
                    in_string = false;
                }
                continue;
            }

            match c {
                // Start of a single-line comment, ignore the rest of the line
                '/' if chars.peek() == Some(&'/') => {
                    if !current_token.is_empty() {
                        tokens.push((current_token.clone(), current_line));
                        current_token.clear();
                    }
                    break; // Move to the next line
                }
                '"' => {
                    if !current_token.is_empty() {
                        tokens.push((current_token.clone(), current_line));
                        current_token.clear();
                    }
                    in_string = true;
                    current_token.push(c);
                }
                ',' | ';' | '(' | ')' | '{' | '}' | '[' | ']' | ':' | '=' | '.' | '@' => {
                    if !current_token.is_empty() {
                        tokens.push((current_token.clone(), current_line));
                        current_token.clear();
                    }
                    tokens.push((c.to_string(), current_line));
                }
                ' ' | '\t' | '\r' => {
                    if !current_token.is_empty() {
                        tokens.push((current_token.clone(), current_line));
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(c);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push((current_token, current_line));
        }

        current_line += 1;
    }

    tokens
}


/// Analyzes Go source files for code duplication using the Rabin-Karp algorithm.
pub fn analyze_duplicates(path: &str, min_tokens: usize, is_directory: bool) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    let files = if is_directory {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("go"))
            .map(|e| e.path().to_str().unwrap().to_string())
            .collect::<Vec<_>>()
    } else {
        if path.ends_with(".go") {
            vec![path.to_string()]
        } else {
            eprintln!("Skipping non-Go file: {}", path);
            return Ok(String::new());
        }
    };

    if files.is_empty() {
        eprintln!("No Go (.go) files found.");
        return Ok(output);
    }

    let mut token_sequence = Vec::new();
    let mut token_info = Vec::new();
    let mut token_to_id = HashMap::new();
    let mut id_to_token = HashMap::new();
    let mut next_id: u64 = 0;

    for file_path in &files {
        let code = fs::read_to_string(file_path)?;
        let tokens = tokenize_go(&code);
        for (token, line) in tokens {
            let id = *token_to_id.entry(token.clone()).or_insert_with(|| {
                let id = next_id;
                id_to_token.insert(id, token);
                next_id += 1;
                id
            });
            token_sequence.push(id);
            token_info.push(TokenInfo {
                file_path: file_path.clone(),
                line_number: line,
            });
        }
    }

    if token_sequence.len() < min_tokens {
        eprintln!(
            "Not enough tokens to analyze ({} available, {} required)",
            token_sequence.len(),
            min_tokens
        );
        return Ok(output);
    }

    let n = min_tokens;
    let base: u64 = 31;
    let modulus: u64 = (1u64 << 61) - 1;
    
    // --- FIX HERE ---
    // Use u128 for intermediate calculations to prevent overflow.
    let modulus_u128 = modulus as u128;

    let mut h: u64 = 1;
    for _ in 0..n.saturating_sub(1) {
        // FIXED LINE
        h = ((h as u128 * base as u128) % modulus_u128) as u64;
    }
    
    let mut hash_map: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut current_hash: u64 = 0;

    // 1. Calculate the hash for the first window.
    if n <= token_sequence.len() {
        for i in 0..n {
            // FIXED LOGIC
            let term = token_sequence[i] as u128;
            current_hash = (((current_hash as u128 * base as u128) % modulus_u128 + term) % modulus_u128) as u64;
        }
        hash_map.entry(current_hash).or_default().push(0);
    }

    // 2. "Roll" the window across the entire sequence.
    for start in 1..=token_sequence.len() - n {
        let old_token = token_sequence[start - 1];
        let new_token = token_sequence[start + n - 1];
        
        // --- FIX HERE ---
        // All multiplications are performed in the u128 context.
        let old_contrib = ((old_token as u128 * h as u128) % modulus_u128) as u64;

        current_hash = (current_hash.wrapping_sub(old_contrib).wrapping_add(modulus)) % modulus;
        current_hash = ((current_hash as u128 * base as u128) % modulus_u128) as u64;
        current_hash = ((current_hash as u128 + new_token as u128) % modulus_u128) as u64;
        
        hash_map.entry(current_hash).or_default().push(start);
    }
    
    // 3. Check for collisions and output the result (unchanged).
    for (_hash, positions) in hash_map {
        if positions.len() > 1 {
            for i in 0..positions.len() {
                for j in i + 1..positions.len() {
                    let p1 = positions[i];
                    let p2 = positions[j];

                    // Prevent out-of-bounds access if p2 is too close to the end
                    if p1 + n > token_sequence.len() || p2 + n > token_sequence.len() {
                        continue;
                    }
                    
                    let seq1 = &token_sequence[p1..p1 + n];
                    let seq2 = &token_sequence[p2..p2 + n];

                    if seq1 == seq2 {
                        let info1 = &token_info[p1];
                        let info2 = &token_info[p2];

                        if info1.file_path == info2.file_path && info1.line_number == info2.line_number {
                            continue;
                        }

                        let tokens_str: Vec<&str> = seq1
                            .iter()
                            .map(|&id| id_to_token.get(&id).unwrap().as_str())
                            .collect();
                        
                        output.push_str(&format!(
                            "Found a duplicate of {} tokens: line {} in file '{}' and line {} in file '{}'.\n  --> Code: \"{}\"\n\n",
                            n, info1.line_number, info1.file_path, info2.line_number, info2.file_path, tokens_str.join(" ")
                        ));
                    }
                }
            }
        }
    }

    if output.is_empty() {
        println!("No duplicates found.");
    }

    Ok(output)
}
