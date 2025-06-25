# zkcat
A zero-knowledge file viewer with cryptographic redaction proofs.

## Features
- **Zero-Knowledge Proofs**: Generate cryptographic proofs of file redaction without revealing original content
- **Selective Redaction**: Redact specific lines by index while preserving the rest
- **Verification**: Independently verify redaction proofs without access to the original file
- **Multiple Output Formats**: Support for both human-readable and JSON output
- **Performance Metrics**: Optional timing and statistics reporting
- **File Export**: Save redacted content to output files

## Installation

### Prerequisites
- Rust toolchain (1.70+)
- RISC Zero zkVM dependencies

### Install from Git [Recommended]

```bash
cargo install --git https://github.com/tanctl/zkcat
```

### Building From Source

```bash
git clone https://github.com/tanctl/zkcat.git
cd zkcat
cargo build --release
```
The binary will be available at `target/release/zkcat`.


## Usage

### Basic file viewing with redaction

### JSON Output

### Proof Verification

## Architecture

## Example Workflow

## Output Formats

### Standard Output

### JSON Output

## License
This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

## Built with [RISC Zero](https://www.risczero.com/) zkVM