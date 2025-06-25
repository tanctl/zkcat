// host/src/main.rs
use clap::Parser;
use colored::*;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Instant;
use anyhow::{Context, ensure};
use methods::{METHOD_ELF, METHOD_ID};
use serde_json::json;

#[derive(Parser)]
#[command(name = "zkcat")]
#[command(about = "Zero-knowledge file viewer with redaction proofs", version)]
struct Cli {

    file: String,
    
    #[arg(short, long)]
    redact: Option<String>,
    
    #[arg(short, long)]
    verify: bool,
    
    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    stats: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    
    if args.verify {
        verify_proof(&args.file, args.json)
    } else {
        view_and_prove(&args.file, args.redact, args.output, args.json, args.stats)
    }
}

fn view_and_prove(
    file_path: &str, 
    redact_lines: Option<String>, 
    output_file: Option<String>,
    json_output: bool,
    show_stats: bool
) -> anyhow::Result<()> {
    let start_time = Instant::now();
    
    let content = fs::read_to_string(file_path)
        .context("Failed to read input file")?;

    let redact_indices: Vec<usize> = redact_lines
        .map(|s| s.split(',').filter_map(|n| n.parse().ok()).collect())
        .unwrap_or_default();

    let mut redacted_lines = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if redact_indices.contains(&i) {
            redacted_lines.push("***REDACTED***".to_string());
        } else {
            redacted_lines.push(line.to_string());
        }
    }
    let redacted_content = redacted_lines.join("\n");

    if !json_output {
        for (i, line) in content.lines().enumerate() {
            if redact_indices.contains(&i) {
                println!("{}", "***REDACTED***".red());
            } else {
                println!("{}", line.green());
            }
        }
    }

    if let Some(output_path) = &output_file {
        fs::write(output_path, &redacted_content)
            .context("Failed to write redacted content to output file")?;
        
        if !json_output {
            println!("\nRedacted content saved to: {}", output_path);
        }
    }

    let full_hash = Sha256::digest(content.as_bytes());
    let full_hash_array: [u8; 32] = full_hash.into();

    let proof_start = Instant::now();

    let env = ExecutorEnv::builder()
        .write(&(content.clone(), redact_indices.clone()))?
        .build()?;

    let prover = default_prover();
    let prove_info = prover.prove(env, METHOD_ELF)
        .context("Proof generation failed")?;

    let proof_time = proof_start.elapsed();

    let receipt = prove_info.receipt;

    let verify_start = Instant::now();
    receipt.verify(METHOD_ID)
        .context("Proof verification failed")?;
    let verify_time = verify_start.elapsed();

    let (journal_full, journal_redacted, journal_indices): ([u8; 32], [u8; 32], Vec<usize>) =
        receipt.journal.decode()?;

    ensure!(
        journal_full == full_hash_array,
        "Full file hash mismatch between host and guest"
    );
    ensure!(
        journal_indices == redact_indices,
        "Redaction indices mismatch between host and guest"
    );

    let proof_file = format!("{}.zkproof", file_path);
    fs::write(&proof_file, bincode::serialize(&receipt)?)
        .context("Failed to save proof file")?;

    let total_time = start_time.elapsed();

    if json_output {
        let result = json!({
            "success": true,
            "full_file_hash": hex::encode(journal_full),
            "redacted_file_hash": hex::encode(journal_redacted),
            "redacted_line_indices": journal_indices,
            "proof_file": proof_file,
            "output_file": output_file,
            "statistics": show_stats.then(|| json!({
                "total_time_ms": total_time.as_millis(),
                "proof_generation_time_ms": proof_time.as_millis(),
                "verification_time_ms": verify_time.as_millis(),
                "file_size_bytes": content.len(),
                "lines_total": content.lines().count(),
                "lines_redacted": redact_indices.len()
            }))
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n{} Proof generated and verified!", "✓".green());
        println!("- Full file SHA-256 hash: {}", hex::encode(journal_full));
        println!("- Redacted file SHA-256 hash: {}", hex::encode(journal_redacted));
        println!("- Redacted line indices: {:?}", journal_indices);
        println!("Proof saved to: {}", proof_file);
        
        if show_stats {
            println!("\n{} Statistics:", "📊".blue());
            println!("- Total time: {:.2}s", total_time.as_secs_f64());
            println!("- Proof generation: {:.2}s", proof_time.as_secs_f64());
            println!("- Verification: {:.3}s", verify_time.as_secs_f64());
            println!("- File size: {} bytes", content.len());
            println!("- Total lines: {}", content.lines().count());
            println!("- Redacted lines: {}", redact_indices.len());
        }
    }

    Ok(())
}

fn verify_proof(proof_path: &str, json_output: bool) -> anyhow::Result<()> {
    let start_time = Instant::now();
    
    let proof_data = fs::read(proof_path)
        .context("Failed to read proof file")?;
    let receipt: Receipt = bincode::deserialize(&proof_data)
        .context("Failed to deserialize proof")?;

    receipt.verify(METHOD_ID)
        .context("Proof verification failed")?;

    let verify_time = start_time.elapsed();

    let (full_hash, redacted_hash, indices): ([u8; 32], [u8; 32], Vec<usize>) =
        receipt.journal.decode()?;

    if json_output {
        let result = json!({
            "success": true,
            "verified": true,
            "full_file_hash": hex::encode(full_hash),
            "redacted_file_hash": hex::encode(redacted_hash),
            "redacted_line_indices": indices,
            "verification_time_ms": verify_time.as_millis()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} Proof verified successfully!", "✓".green());
        println!("- Full file SHA-256 hash: {}", hex::encode(full_hash));
        println!("- Redacted file SHA-256 hash: {}", hex::encode(redacted_hash));
        println!("- Redacted line indices: {:?}", indices);
        println!("- Verification time: {:.3}s", verify_time.as_secs_f64());
    }

    Ok(())
}