// methods/guest/src/main.rs
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() {
    let (full_content, redact_indices): (String, Vec<usize>) = env::read();
    
    let full_hash = Sha256::digest(full_content.as_bytes());
    let full_hash_array: [u8; 32] = full_hash.into();
    
    let mut lines: Vec<&str> = full_content.lines().collect();
    for &idx in &redact_indices {
        if idx < lines.len() {
            lines[idx] = "***REDACTED***";
        }
    }
    let redacted_content = lines.join("\n");
    
    let redacted_hash = Sha256::digest(redacted_content.as_bytes());
    let redacted_hash_array: [u8; 32] = redacted_hash.into();
    
    env::commit(&full_hash_array);
    env::commit(&redacted_hash_array);
    env::commit(&redact_indices);
}