# TUI Port Manager

TUI Port Manager is a terminal user interface tool for interactively viewing and managing network ports and processes on a Linux system. It is built in Rust using the `ratatui` and `procfs` crates, with deep system integration.

## Features

- Interactive terminal user interface (TUI)
- Displays open TCP and UDP ports with associated processes and PIDs
- Search bar for filtering by port, process name, or state
- Vim-style navigation (`j`/`k` for up/down)
- Kill processes with `c`, with confirmation popup
- Real-time refresh of network data
- Uses `/proc` via `procfs` for system integration (no external tools like lsof required)
- **Theming support**: Toggle between default and dark themes with `t`
- **Port sorting**: Cycle sorting by port, process, protocol, or state with `s`
- **Protocol filtering**: Cycle between TCP, UDP, or all ports with `p`
- Filtered port list updates as you type
- Confirmation dialog for killing processes

## Keybindings

- `j` / `k`: Move selection down/up
- `/`: Enter search mode
- `Esc` or `Enter`: Exit search mode
- `Backspace`: Remove last character in search
- `c`: Kill selected process (with confirmation)
- `q`: Quit
- `t`: Toggle theme (default/dark)
- `s`: Cycle port sorting (port, process, protocol, state)
- `p`: Cycle protocol filter (all, TCP, UDP)

## Usage

Run the program in your terminal:

```bash
cargo run
```

## Requirements

- Linux system
- Rust toolchain (`cargo`, `rustc`)

## Screenshots

_Add screenshots here to showcase the UI and features._

# Port Manager Project

This project provides a Rust-based port management tool and API server, with monitoring and visualization via Prometheus and Grafana.

## Features
- Reserve and release ports via API
- Rate limiting (10 requests per IP per minute)
- Prometheus metrics endpoint (`/metrics`)
- Grafana dashboards for monitoring
- Docker Compose setup for Prometheus, Grafana, and Apache
- Posting YAML file for automated API testing

## Structure
```
port-manager/
  src/
    main.rs         # CLI/TUI or main app
    api/            # API server code
      Cargo.toml
      src/
        main.rs
        reservation.rs
  monitoring/
    prometheus/
      prometheus.yml
    docker-compose.yml
  Cargo.toml        # Workspace config
```

## Usage
### Build and Run
```bash
cargo build --workspace
cargo run --manifest-path src/api/Cargo.toml
```

### API Endpoints
- Reserve a port:
  `POST /reserve` with JSON `[port, service]`
- Release a port:
  `POST /release` with JSON `port`
- Check status:
  `GET /status/{port}`
- Metrics:
  `GET /metrics`

### Rate Limiting
Each IP is limited to 10 requests per minute. Exceeding this returns HTTP 429.

### Monitoring
- Start monitoring stack:
  ```bash
  cd src/monitoring
  docker compose up -d
  ```
- Prometheus: http://prometheus:9090
- Grafana: http://localhost:3000

### Testing
- Use Posting tool with `tests/port_reservation_tests.posting.yaml` for automated API tests.

## Advanced Features
- Custom Prometheus metrics for reserve/release requests
- Grafana dashboards (see `port_manager_advanced_dashboard.json`)

## Contributing
PRs and issues welcome!

## License
MIT

