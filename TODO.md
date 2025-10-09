## **Base Command**

```
lock <COMMAND> [OPTIONS]
```

## **Commands**

### **1. `encrypt`**

Encrypt all image files in a directory using AES-256-GCM (default).

```
lock encrypt --in <input_dir> --out <output_dir> [OPTIONS]
```

**Options**

| Option                    | Description                                              |
| ------------------------- | -------------------------------------------------------- |
| `--key-file <path>`       | Use an existing key file (plain or passphrase-protected) |
| `--passphrase-prompt, -p` | Prompt securely for passphrase (Argon2id key derivation) |
| `--overwrite`             | Overwrite existing `.lock` files (default: skip)         |
| `--include-hidden`        | Include hidden files and directories                     |
| `--jobs, -j <N>`          | Number of parallel worker threads (default: CPU count)   |
| `--verbose, -v`           | Verbose output (disable progress bar, log per file)      |

---

### **2. `decrypt`**

Decrypt `.lock` files back to their original form.

```
lock decrypt --in <input_dir> --out <output_dir> [OPTIONS]
```

**Options**

| Option                    | Description                                              |
| ------------------------- | -------------------------------------------------------- |
| `--key-file <path>`       | Use an existing key file (plain or passphrase-protected) |
| `--passphrase-prompt, -p` | Prompt for passphrase                                    |
| `--include-hidden`        | Include hidden files                                     |
| `--jobs, -j <N>`          | Number of parallel worker threads (default: CPU count)   |
| `--verbose, -v`           | Verbose output (disable progress bar, log per file)      |

---

### **3. `keygen`**

Generate a new encryption key file.

```
lock keygen --out <key_file> [OPTIONS]
```

**Options**

| Option                    | Description                                                       |
| ------------------------- | ----------------------------------------------------------------- |
| `--algo <aes>`            | Key algorithm (default: AES-256-GCM)                              |
| `--force, -f`             | Overwrite existing key file                                       |
| `--passphrase-prompt, -p` | Prompt for passphrase to protect the key (Argon2id + AES-256-GCM) |

---

### **4. `inspect`**

Display header information from a `.lock` encrypted file or key file (`.key`) without decrypting.
This reveals **safe metadata only** — version, algorithm, salt, and nonce.

```
lock inspect <file> [--json]
```

**Options**

| Option   | Description                    |
| -------- | ------------------------------ |
| `--json` | Output metadata in JSON format |

**Examples**

Human-readable:

```
lock inspect vault/photo.jpg.lock
```

JSON output:

```
lock inspect lock.key --json
```

---

### **5. `version` / `help`**

Show program version or help.

```
lock --version
lock help [command]
```


## **Project Layout**

```
lock/
 ├─ Cargo.toml
 └─ src/
     ├─ main.rs              # Entry point
     ├─ cli.rs               # Top-level CLI parser and command dispatch
     │
     ├─ commands/
     │   ├─ mod.rs
     │   ├─ encrypt.rs       # AES-256-GCM encryption logic
     │   ├─ decrypt.rs       # AES-256-GCM decryption logic
     │   ├─ keygen.rs        # Key generation (plain / passphrase-protected)
     │   └─ inspect.rs       # Header inspection (safe metadata)
     │
     ├─ options/
     │   ├─ mod.rs
     │   ├─ encrypt.rs
     │   ├─ decrypt.rs
     │   ├─ keygen.rs
     │   └─ inspect.rs
     │
     ├─ cry.rs               # Key loading, Argon2id KDF, AES-GCM primitives
     ├─ walk.rs              # File discovery and planning utilities
     ├─ utils.rs             # Helpers for hidden files, image detection, etc.
     ├─ constants.rs         # Magic constants and header format
```
