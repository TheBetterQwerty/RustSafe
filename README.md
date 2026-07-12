# Rust-Safe [1.8.2]

**Rust-Safe** is a secure, lightweight, command-line password manager written in Rust. It is designed for simplicity, local control, and robust cryptographic practices. Passwords are encrypted and stored in a local JSON file, which can be safely exported or imported as needed. It supports profiles with different master passwords as of current version.

---

## Features

| Command                          | Description                                   |
| -------------------------------- | --------------------------------------------- |
| `--version`                      | Display the current RustSafe version          |
| `--init <profile>`               | Initialize the database                       |
| `--logs`                         | Print all saved logs                          |
| `--add <name>`                   | Add a new password entry                      |
| `--get <name>`                   | Retrieve a stored password                    |
| `--list`                         | List all saved entries                        |
| `--edit <name>`                  | Edit an existing password entry               |
| `--rm <name>`                    | Remove an entry                               |
| `--generate <size>`              | Generate a secure random password             |
| `--passwd`                       | Change the master password                    |
| `--import <path>`                | Import passwords from a JSON file             |
| `--export`                       | Export all passwords to a secure JSON file    |
| `--create-profile <name>`        | Create a new profile                          |
| `--edit-profile <name>`          | Rename or modify a profile                    |
| `--delete-profile <name>`        | Delete a profile                              |
| `--list-profiles`                | List all available profiles                   |
| `--set-default-profile <name>`   | Set the default profile                       |
| `--from <name>`                  | Execute a command using the specified profile |

---

## Security Design

* **Secure Password Input**
  Use of a crate preventing passwords from being visible on screen.

* **Master Password Protection**
  Access to stored passwords requires a master password, which is never saved or stored directly.

* **SHA-256 with Salting**
  The master password is salted and hashed using SHA-256 to ensure resistance to dictionary and precomputation attacks.

* **HMAC-Based File Integrity**
  Each data file includes an HMAC (Hash-based Message Authentication Code) to verify that the contents have not been tampered with.

* **Local-Only Storage**
  Passwords are stored only on the local file system in encrypted form. No network access is required or used.

* **Rate-Limiting**
  If user enters incorrect password then they are locked for 5 minutes (customizable)

---

## Data Storage Logic

* Passwords are encrypted and stored in a single JSON file.
* The encryption key is derived from the user's master password and the hex encoded random nonce is used as the salt with the password.
* The password file can be exported and imported securely across systems, provided the same master password is used.
* The master password is required at runtime and never written to disk.

---

## Requirements

* Rust (stable)
* Works on Unix-based systems and Windows

---

## Build and Run

```bash
git clone "https://github.com/TheBetterQwerty/RustSafe.git"
cd RustSafe
make
```

---

