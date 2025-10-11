# lock-rs

Yet another paranoid lock - zeroidize your personal pixels before they hit the cloud.

## Overview

It just encrypts images only using AES-256-GCM before uploading to cloud storage providers.

## Installation

### Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/elly-hacen/lock-rs/main/install.sh | bash
```

### Manual Install

```bash
# Download and run installer
curl -sSL https://raw.githubusercontent.com/elly-hacen/lock-rs/main/install.sh -o install.sh
chmod +x install.sh
./install.sh

# Or install to custom directory
./install.sh --install-dir ~/.local/bin
```

### Build from Source

```bash
git clone https://github.com/elly-hacen/lock-rs.git
cd lock-rs
cargo build --release
sudo cp target/release/lock /usr/local/bin/
```

### Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Create release
./release.sh v0.1.0
```

## Usage

### Generate Encryption Key

```bash
# Generate plaintext key
lock keygen -o lock.key

# Generate passphrase-protected key
lock keygen -o lock.key --passphrase-prompt
```

### Encrypt Files

```bash
# Encrypt all images in directory
lock encrypt -i ./photos -o ./encrypted --key-file lock.key

# Include hidden files
lock encrypt -i ./photos -o ./encrypted --key-file lock.key --include-hidden
```

### Decrypt Files

```bash
# Decrypt all .lock files
lock decrypt -i ./encrypted -o ./decrypted --key-file lock.key
```

### Upload to Cloud

```bash
# Upload encrypted files to GitHub
lock upload github -i ./encrypted --github-repo owner/repo --github-branch main

# Verbose output with detailed progress
lock upload github -i ./encrypted --github-repo owner/repo --github-branch main --verbose
```

### Inspect Files

```bash
# Inspect encrypted file
./lock inspect encrypted-file.lock

# Inspect key file
./lock inspect lock.key

# JSON output
./lock inspect encrypted-file.lock --json
```

## Configuration

### Environment Variables

- `GITHUB_TOKEN`: GitHub personal access token for uploads
- Make sure to generate fine grain toke with Content (Read and Write permission and allow all repose or 1st create one and allow only one)

### Key File Format

lock-rs supports two key file formats:

1. **Plaintext**: 64-character hex string (32 bytes)
2. **Protected**: Binary format with passphrase protection using Argon2id

## Security

- **Encryption**: AES-256-GCM with 96-bit nonces
- **Key Derivation**: Argon2id for passphrase protection
- **Authentication**: Built-in authentication tags prevent tampering
- **Memory Safety**: Automatic key zeroization on drop

## File Formats

### Encrypted Files (.lock)

```
MAGIC(5) | NONCE(12) | CIPHERTEXT + AUTH_TAG(16)
```

### Key Files (.key)

**Plaintext Format:**
```
64-character hex string
```

**Protected Format:**
```
LKEY1 | SALT(16) | NONCE(12) | AES-256-GCM(PT=32B key, AAD=header) | TAG(16)
```

## Supported File Types

Lock automatically detects and processes image files:
- JPEG, PNG, GIF, BMP, TIFF, WebP, AVIF
- HEIC, HEIF (iPhone formats)
- RAW formats: DNG, CR2, CR3, NEF, ARW, RAF, RW2, ORF, SR2, PEF, RAW

## Examples

### Complete Workflow

```bash
# 1. Generate key
lock keygen -o mykey.key --passphrase-prompt

# 2. Encrypt photos
lock encrypt -i ./photos -o ./encrypted --key-file mykey.key

# 3. Upload to GitHub
lock upload github -i ./encrypted --github-repo user/backup --github-branch main

# 4. Download and decrypt (on another machine)
lock decrypt -i ./encrypted -o ./photos --key-file mykey.key
```