# egui-port-manager

A cross-platform desktop GUI port manager built with Rust and egui. Lists open ports and allows filtering. No Tauri or webkit2gtk required.

## Features
- List open ports (TCP/UDP, IPv4/IPv6)
- Filter by any string (port, process, etc.)
- Modern, responsive UI (egui)

## Build & Run (Fedora)
1. Install Rust: https://rustup.rs
2. Install system dependencies (if needed):
   - Fedora: `sudo dnf install pkg-config libxcb-devel`
3. Build and run:
   ```bash
   cargo run --release
   ```

## Project Structure
- `src/main.rs`: egui UI and port listing logic

---
Replace this README with more details as you add features!
