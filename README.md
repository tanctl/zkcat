# zkcat
A zero-knowledge file viewer with cryptographic redaction proofs

## Features
- **Zero-Knowledge Proofs**: Generate cryptographic proofs of file redaction without revealing original content
- **Selective Redaction**: Redact specific lines by index while preserving the rest
- **Verification**: Independently verify redaction proofs without access to the original file
- **Multiple Output Formats**: Support for both human-readable and JSON output
- **Performance Metrics**: Optional timing and statistics reporting
- **File Export**: Save redacted content to output files
- **Trustless**: No need to trust the redactor - cryptographic proofs ensure integrity

## Installation
### Prerequisites
- **Rust toolchain** (1.70 or later)
- **RISC Zero zkVM** dependencies (automatically handled by cargo)

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

The binary will be available at `target/release/zkcat` or you can install it with:

```bash
cargo install --path host
```

## Usage

### Basic Commands

```bash
# View file with redaction
zkcat <file> --redact <line_indices>

# Generate proof and save to file
zkcat document.txt --redact 1,3,5 --output redacted.txt

# Verify an existing proof
zkcat document.txt.zkproof --verify

# JSON output for programmatic use
zkcat document.txt --redact 2,4 --json

# Show performance statistics
zkcat document.txt --redact 1 --stats
```

### Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--redact` | `-r` | Comma-separated line indices to redact (0-indexed) |
| `--verify` | `-v` | Verify an existing proof file |
| `--output` | `-o` | Save redacted content to file |
| `--json` | | Output results in JSON format |
| `--stats` | | Show performance statistics |
| `--help` | `-h` | Show help information |
| `--version` | | Show version information |

**Note:** Line indices are 0-indexed, meaning the first line is line 0, second line is line 1, etc.

### Basic File Viewing with Redaction

```bash
# Redact lines 1, 3, and 6 from document.txt
zkcat document.txt --redact 1,3,6
```

**Output:**
```
Hello, this is a public note.
***REDACTED***
Nothing to redact in this line, all clear.
***REDACTED***
This file contains sensitive information.
Just a random comment about cats and pizza.
***REDACTED***

✓ Proof generated and verified!
- Full file SHA-256 hash: a1b2c3d4e5f6...
- Redacted file SHA-256 hash: f6e5d4c3b2a1...
- Redacted line indices: [1, 3, 6]
Proof saved to: document.txt.zkproof
```

### JSON Output

```bash
zkcat document.txt --redact 1,3 --json --stats
```

**Output:**
```json
{
  "success": true,
  "full_file_hash": "a1b2c3d4e5f6789abcdef123456789abcdef123456789abcdef123456789abcd",
  "redacted_file_hash": "f6e5d4c3b2a1987654321fedcba987654321fedcba987654321fedcba987654",
  "redacted_line_indices": [1, 3],
  "proof_file": "document.txt.zkproof",
  "output_file": null,
  "statistics": {
    "total_time_ms": 2543,
    "proof_generation_time_ms": 2234,
    "verification_time_ms": 12,
    "file_size_bytes": 287,
    "lines_total": 10,
    "lines_redacted": 2
  }
}
```

### Proof Verification

```bash
zkcat document.txt.zkproof --verify
```

**Output:**
```
✓ Proof verified successfully!
- Full file SHA-256 hash: a1b2c3d4e5f6789abcdef123456789abcdef123456789abcdef123456789abcd
- Redacted file SHA-256 hash: f6e5d4c3b2a1987654321fedcba987654321fedcba987654321fedcba987654
- Redacted line indices: [1, 3]
- Verification time: 0.012s
```

## Architecture

zkcat uses a **host-guest architecture** powered by the RISC Zero zkVM:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│      Host       │    │   RISC Zero     │    │      Guest      │
│   (main app)    │───▶│     zkVM        │───▶│  (computation)  │
│                 │    │                 │    │                 │
│ • File I/O      │    │ • Proof Gen     │    │ • Hash original │
│ • CLI interface │    │ • Verification  │    │ • Apply redact  │
│ • Proof storage │    │ • Isolation     │    │ • Hash redacted │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Technical Components
- **Host Binary** (`host/src/main.rs`): Command-line interface, file I/O, proof verification
- **Guest Program** (`methods/guest/src/main.rs`): Zero-knowledge computation in isolated environment
- **Methods** (`methods/`): Build configuration and method exports for zkVM integration
- **Workspace Configuration**: Optimized build settings for zkVM performance

### How Zero-Knowledge Proofs Work
1. **Input**: Original file content + redaction indices
2. **Guest Execution**: 
   - Computes SHA-256 hash of original content
   - Applies redaction to specified lines (replaces with "***REDACTED***")
   - Computes SHA-256 hash of redacted content
   - Commits hashes and indices to journal
3. **Proof Generation**: zkVM creates cryptographic proof of correct execution
4. **Verification**: Anyone can verify the proof without seeing original content

### Proof File Format
Proof files (`.zkproof`) contain:
- **Receipt**: Cryptographic proof of computation
- **Journal Data**: Original file hash, redacted file hash, redaction indices
- **Method ID**: Identifier for the guest program that generated the proof
The proof files are serialized using bincode for efficient storage and transmission.

### Security Properties
- **Privacy**: Original content never leaves the host system
- **Integrity**: Proof guarantees redaction was performed correctly
- **Verifiability**: Third parties can verify redaction without seeing original content
- **Non-repudiation**: Cryptographic proof prevents denial of redaction

## Example Workflow
### 1. Create a sample file

```bash
cat > sensitive_doc.txt << EOF
Public information line 1
SECRET: API key yogurt
Public information line 3
CONFIDENTIAL: Password oiia
Public information line 5
EOF
```

### 2. Redact sensitive lines

```bash
zkcat sensitive_doc.txt --redact 1,3 --output clean_doc.txt --stats
```

### 3. Share redacted file and proof

```bash
# Share these files publicly:
# - clean_doc.txt (redacted content)
# - sensitive_doc.txt.zkproof (verification proof)

# Recipients can verify without seeing original:
zkcat sensitive_doc.txt.zkproof --verify
```

### 4. Programmatic integration

```bash
# Use JSON output for automation
zkcat sensitive_doc.txt --redact 1,3 --json > redaction_result.json
```

## Output Formats

### Standard Output
- **Green text**: Non-redacted lines
- **Red text**: Redacted lines shown as `***REDACTED***`
- **Checkmarks**: Success indicators
- **Hashes**: SHA-256 hashes in hexadecimal
- **Performance**: Timing information when `--stats` is used

### JSON Output

```json
{
  "success": boolean,
  "full_file_hash": "hex_string",
  "redacted_file_hash": "hex_string", 
  "redacted_line_indices": [number],
  "proof_file": "string",
  "output_file": "string|null",
  "statistics": {
    "total_time_ms": number,
    "proof_generation_time_ms": number,
    "verification_time_ms": number,
    "file_size_bytes": number,
    "lines_total": number,
    "lines_redacted": number
  }
}
```


## Advanced Usage

### Batch Processing

```bash
# Process multiple files
for file in *.txt; do
    zkcat "$file" --redact 0,2,4 --output "clean_$file" --json
done
```

### Integration with Scripts

```bash
#!/bin/bash
# Automated redaction pipeline
RESULT=$(zkcat document.txt --redact 1,3,5 --json)
SUCCESS=$(echo "$RESULT" | jq -r '.success')

if [ "$SUCCESS" = "true" ]; then
    echo "Redaction completed successfully"
    PROOF_FILE=$(echo "$RESULT" | jq -r '.proof_file')
    echo "Proof saved to: $PROOF_FILE"
else
    echo "Redaction failed"
    exit 1
fi
```

### Performance Tuning

```bash
# For large files, monitor performance
zkcat large_file.txt --redact 10,20,30 --stats --json | jq '.statistics'
```

### Code Structure

```
zkcat/
├── host/                    # Main application
│   ├── src/main.rs         # CLI interface and proof handling
│   └── Cargo.toml          # Host dependencies
├── methods/                 # zkVM integration
│   ├── guest/              # Guest program (runs in zkVM)
│   │   ├── src/main.rs     # Zero-knowledge computation
│   │   └── Cargo.toml      # Guest dependencies
│   ├── src/lib.rs          # Method exports
│   ├── build.rs            # Build configuration
│   └── Cargo.toml          # Methods dependencies
└── Cargo.toml              # Workspace configuration
```

## License
This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments
#### Built with [RISC Zero](https://www.risczero.com/) zkVM
This project uses the [RISC Zero](https://www.risczero.com/) zkVM, which is licensed under the Apache License 2.0.

---

*zkcat - Trustless file redaction with zero-knowledge proofs*
