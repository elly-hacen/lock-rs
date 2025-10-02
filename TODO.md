
# Lock CLI Structure

## Base command

```
lock <COMMAND> [OPTIONS]
```

## Commands

### 1. `encrypt`

Encrypts all images in a directory.

```
lock encrypt --in <input_dir> --out <output_dir> [OPTIONS]
```

**Options**

* `--key-file <path>` → use an existing key file
* `--passphrase` → prompt securely for passphrase (derive key via Argon2id)
* `--algo <aes|chacha>` → choose algorithm (default: chacha)
* `--dry-run` → show plan, don’t write files
* `--concurrency <N>` → number of worker threads
* `--overwrite` → overwrite if output exists (default: error)
* `--include-hidden` → include hidden files
* `--preserve-times` → copy mtime/ctime
* `--max-size <MB>` → skip files larger than this

---

### 2. `decrypt`

Decrypts `.flk` files back into original images.

```
lock decrypt --in <input_dir> --out <output_dir> [OPTIONS]
```

**Options**

* `--key-file <path>` → use an existing key file
* `--passphrase` → prompt for passphrase
* `--overwrite-policy <skip|overwrite|rename>` → handle collisions
* `--concurrency <N>` → threads
* `--dry-run` → show plan only

---

### 3. `keygen`

Generate a new key file.

```
lock keygen --out tlock.key [OPTIONS]
```

**Options**

* `--algo <aes|chacha>` → default key type
* `--force` → overwrite if file exists

---

### 4. `inspect`

Check header info of an encrypted file (safe metadata only).

```
lock inspect <file.flk>
```

Output: version, algo, ext stored, salt present, nonce.

---

### 5. `version` / `help`

* `lock --version`
* `lock help [command]`

---

# Example Usage

```bash
# Encrypt a folder with a generated key
lock keygen --out lock.key
lock encrypt --in ~/Pictures --out ~/Encrypted --key-file lock.key

# Encrypt with passphrase (will prompt)
lock encrypt --in ./images --out ./enc --passphrase

# Decrypt back
lock decrypt --in ./enc --out ./restored --key-file lock.key

# Inspect one encrypted file
lock inspect ./enc/photo.jpg.flk
```

---

**Design Philosophy**

* One key command = one job.
* No hidden magic: everything explicit (`--in`, `--out`, `--key-file`).
* Safe defaults: refuse overwrite unless told.
* `inspect` helps debugging without decryption.

```
lock/
 ├─ Cargo.toml
 └─ src/
     ├─ main.rs            # entrypoint
     ├─ cli.rs             # top-level CLI wiring
     │
     ├─ commands/
     │   ├─ mod.rs
     │   ├─ encrypt.rs
     │   ├─ decrypt.rs
     │   ├─ keygen.rs
     │   └─ inspect.rs
     │
     ├─ options/
     │   ├─ mod.rs
     │   ├─ encrypt.rs
     │   ├─ decrypt.rs
     │   ├─ keygen.rs
     │   └─ inspect.rs
     │
     ├─ crypto.rs          # AEAD, KDF, header
     ├─ fs_utils.rs        # file walker, atomic writes
     ├─ progress.rs        # progress + events
     ├─ errors.rs          # error enum
```
