# SFT - Secure File Transfer

**SFT** (Secure File Transfer) is a lightweight, fast, and secure alternative to `scp`. It allows you to transfer files between hosts over **TCP with end-to-end encryption**, using a simple `ssh`-like syntax.

Unlike plain `scp`, SFT is designed to be **minimal, extensible, and self-hosted**, with a modern crypto stack and async Rust internals.

---

## Features

* **End-to-end encryption** using AES-GCM + X25519 key exchange
* **Fast async transfers** with `tokio`
* **Daemon-based architecture** (`sftd`) for persistent connections
* **User\@Host syntax** like SSH (`sft send file.txt user@host`)
* **Config file** (`~/.sft/config`) for host aliases and defaults
* **Resumable transfers** (planned)
* **Directory transfers & compression** (planned)

---

## Getting Started

### 1. Installation

```bash
cargo install sft
```

This provides two binaries:

* `sftd` → the daemon running on the server/receiver
* `sft` → the CLI client for sending and receiving files

### 2. Running the Daemon

On your remote host (e.g. Raspberry Pi, server, VPS):

```bash
sftd --user pepe --port 5555
```

This starts the SFT daemon for user `pepe`, listening on port `5555` (default).

### 3. Sending a File

From your local machine:

```bash
sft send ./secret.txt pepe@192.168.1.32
```

This will:

1. Connect to `192.168.1.32:5555`
2. Authenticate as `pepe`
3. Negotiate a secure session key via Diffie-Hellman
4. Encrypt `secret.txt` with AES-GCM
5. Stream it to the daemon

### 4. Receiving a File (optional)

If you want to pull from a remote machine:

```bash
sft recv pepe@192.168.1.32:/remote/path/file.txt ./local/path/
```

---

## Configuration

SFT supports a configuration file at `~/.sft/config`, inspired by SSH:

```ini
Host pi
    HostName 192.168.1.32
    User pepe
    Port 5555

Host server
    HostName myserver.tld
    User root
```

Then you can simply run:

```bash
sft send ./secret.txt pi
```

---

## Authentication & Security

SFT supports multiple authentication methods:

* **Pre-shared key**: stored in `~/.sft/id_sft`
* **Password-based**: prompt at connection time
* **Public/private keys** (planned): similar to SSH `id_rsa`

### Encryption

* Key exchange: **X25519 Diffie-Hellman**
* Symmetric encryption: **AES-256-GCM**
* Integrity: **HMAC-SHA256**

---

## Roadmap

### v0.1 (MVP)

* [x] Basic client/server (`sft`, `sftd`)
* [x] Default port (5555)
* [x] `user@host` syntax
* [x] AES-GCM encryption with pre-shared key

### v0.2

* [ ] Host aliases (`~/.sft/config`)
* [ ] Better error handling & retries
* [ ] Logging & metrics

### v0.3

* [ ] Public/private key authentication
* [ ] File resume on interruption
* [ ] Parallel chunked transfers

### v1.0

* [ ] Directory transfers (`sft send ./folder user@host`)
* [ ] Compression (LZ4/Zstd)
* [ ] Windows/Mac support
* [ ] Cross-platform release binaries

---

## Protocol Design

SFT defines a lightweight protocol on top of TCP:

1. **Handshake Phase**

   * Client → Hello (user, protocol version)
   * Server → Ack
   * Diffie-Hellman key exchange

2. **Authentication Phase**

   * Client proves knowledge of pre-shared key / password / private key
   * Server validates

3. **Transfer Phase**

   * Client → File metadata (name, size, checksum)
   * Server → Ready
   * Client → Encrypted file stream in chunks
   * Server → Ack + checksum validation

4. **Closure Phase**

   * Both sides close session gracefully

---

## License

MIT License © 2025

---

## Contributing

Contributions are welcome! Some areas you can help with:

* Improving protocol security
* Adding compression & resume support
* Building a TUI for transfer progress
* Packaging for Linux distros

PRs are open 😃
