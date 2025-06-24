// host/src/main.rs
use clap::Parser;
use colored::*;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use sha2::{Digest, Sha256};
use std::fs;
use anyhow::{Context, ensure};
use methods::{METHOD_ELF, METHOD_ID};

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
        verify_proof(&args.file)
    } else {
        view_and_prove(&args.file, args.redact)
    }
}

fn view_and_prove(file_path: &str, redact_lines: Option<String>) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)
        .context("Failed to read input file")?;

    let redact_indices: Vec<usize> = redact_lines
        .map(|s| s.split(',').filter_map(|n| n.parse().ok()).collect())
        .unwrap_or_default();

    for (i, line) in content.lines().enumerate() {
        if redact_indices.contains(&i) {
            println!("{}", "***REDACTED***".red());
        } else {
            println!("{}", line.green());
        }
    }

    let full_hash = Sha256::digest(content.as_bytes());
    let full_hash_array: [u8; 32] = full_hash.into();

    let env = ExecutorEnv::builder()
        .write(&(content, redact_indices.clone()))?
        .build()?;

    let prover = default_prover();
    let prove_info = prover.prove(env, METHOD_ELF)
        .context("Proof generation failed")?;

    let receipt = prove_info.receipt;

    receipt.verify(METHOD_ID)
        .context("Proof verification failed")?;

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

    let proof_file = format!("{}.proof", file_path);
    fs::write(&proof_file, bincode::serialize(&receipt)?)
        .context("Failed to save proof file")?;

    println!("\n{} Proof generated and verified!", "✓".green());
    println!("- Full file SHA-256 hash: {}", hex::encode(journal_full));
    println!("- Redacted file SHA-256 hash: {}", hex::encode(journal_redacted));
    println!("- Redacted line indices: {:?}", journal_indices);
    println!("Proof saved to: {}", proof_file);

    Ok(())
}

fn verify_proof(proof_path: &str) -> anyhow::Result<()> {
    let proof_data = fs::read(proof_path)
        .context("Failed to read proof file")?;

    let receipt: Receipt = bincode::deserialize(&proof_data)
        .context("Failed to deserialize proof")?;

    receipt.verify(METHOD_ID)
        .context("Proof verification failed")?;

    let (full_hash, redacted_hash, indices): ([u8; 32], [u8; 32], Vec<usize>) = 
        receipt.journal.decode()?;

    println!("{} Proof verified successfully!", "✓".green());
    println!("- Full file SHA-256 hash: {}", hex::encode(full_hash));
    println!("- Redacted file SHA-256 hash: {}", hex::encode(redacted_hash));
    println!("- Redacted line indices: {:?}", indices);

    Ok(())
}