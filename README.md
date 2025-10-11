# Lock

A secure file encryption and cloud upload utility for protecting personal data before cloud storage.

## Overview

Lock provides end-to-end encryption for files using AES-256-GCM before uploading to cloud storage providers. It ensures your data remains encrypted even when stored on third-party services.

## Features

- **AES-256-GCM Encryption**: Military-grade encryption with authenticated encryption
- **Key Management**: Secure key generation with optional passphrase protection
- **Cloud Upload**: Direct upload to GitHub repositories
- **Batch Processing**: Encrypt/decrypt entire directories
- **Progress Tracking**: Real-time progress indicators with verbose output
- **File Inspection**: Analyze encrypted files and key files
- **Cross-Platform**: Works on Unix-like systems

## Installation

### Prerequisites

- Rust 1.70+ with Cargo
- Git (for cloud uploads)

### Build from Source

```bash
git clone <repository-url>
cd lock
cargo build --release
```

The binary will be available at `target/release/lock`.

## Usage

### Generate Encryption Key

```bash
# Generate plaintext key
./lock keygen -o lock.key

# Generate passphrase-protected key
./lock keygen -o lock.key --passphrase-prompt
```

### Encrypt Files

```bash
# Encrypt all images in directory
./lock encrypt -i ./photos -o ./encrypted --key-file lock.key

# Include hidden files
./lock encrypt -i ./photos -o ./encrypted --key-file lock.key --include-hidden
```

### Decrypt Files

```bash
# Decrypt all .lock files
./lock decrypt -i ./encrypted -o ./decrypted --key-file lock.key
```

### Upload to Cloud

```bash
# Upload encrypted files to GitHub
./lock upload github -i ./encrypted --github-repo owner/repo --github-branch main

# Verbose output with detailed progress
./lock upload github -i ./encrypted --github-repo owner/repo --github-branch main --verbose
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

### Key File Format

Lock supports two key file formats:

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

## Performance

- **Parallel Processing**: Multi-threaded encryption/decryption
- **Memory Optimization**: Stack-based buffers and dynamic capacity allocation
- **Progress Tracking**: Real-time progress indicators
- **Efficient I/O**: Optimized file operations

## Supported File Types

Lock automatically detects and processes image files:
- JPEG, PNG, GIF, BMP, TIFF, WebP, AVIF
- HEIC, HEIF (iPhone formats)
- RAW formats: DNG, CR2, CR3, NEF, ARW, RAF, RW2, ORF, SR2, PEF, RAW

## Examples

### Complete Workflow

```bash
# 1. Generate key
./lock keygen -o mykey.key --passphrase-prompt

# 2. Encrypt photos
./lock encrypt -i ./photos -o ./encrypted --key-file mykey.key

# 3. Upload to GitHub
./lock upload github -i ./encrypted --github-repo user/backup --github-branch main

# 4. Download and decrypt (on another machine)
./lock decrypt -i ./encrypted -o ./photos --key-file mykey.key
```

### Batch Operations

```bash
# Encrypt large directory with progress
./lock encrypt -i ./large-photo-collection -o ./encrypted --key-file lock.key

# Upload with verbose progress
./lock upload github -i ./encrypted --github-repo user/backup --verbose
```

## Error Handling

Lock provides detailed error messages and context for troubleshooting:

- File access errors with specific paths
- Encryption/decryption failures with authentication details
- Network errors for cloud uploads
- Key file format validation

## Development

### Build Profiles

- **Development**: `cargo build` (optimized with debug info)
- **Release**: `cargo build --release` (size-optimized binary)

### Testing

```bash
cargo test
```

### Code Quality

- Zero warnings compilation
- Memory-safe operations
- Comprehensive error handling
- Professional commit history

## License

[Add your license information here]

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with proper commit messages
4. Submit a pull request

## Security Notice

- Keep your key files secure and make multiple backups
- Without the key file and passphrase, encrypted data cannot be recovered
- Use strong passphrases for key protection
- Regularly backup your key files to secure locations
