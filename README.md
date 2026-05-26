<div align="center">

<img src="./src/icon/tracetuiicon.png" alt="TraceTUI Logo" width="128" height="128">

# TraceTUI

**Next-Generation Terminal Intelligence for Network & Systems Forensics**

[![Build Status](https://img.shields.io/github/actions/workflow/status/AcoranGonzalezMoray/TraceTUI/rust.yml?style=flat-square)](https://github.com/AcoranGonzalezMoray/TraceTUI/actions)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Website](https://img.shields.io/badge/Official-Website-blue?style=flat-square&logo=edge)](https://acorangonzalezmoray.github.io/TraceTUI/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](CONTRIBUTING.md)
[![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux-lightgrey.svg?style=flat-square)](#-installation)

[Features](#-key-features) •
[Installation](#-installation) •
[Usage](#-quick-start) •
[Architecture](#-architecture) •
[Contributing](#-contributing)

</div>

---

## 📖 Overview

**TraceTUI** is an advanced, high-performance terminal user interface (TUI) designed for deep system forensics and network investigation. Built with **Rust** and **Ratatui**, it empowers power users, sysadmins, and cybersecurity professionals with real-time monitoring of network traffic, comprehensive process management, and rapid suspicious activity analysis—all without leaving the terminal.

<img src="./docs/assets/menu.png" alt="TraceTUI Menu">

## ✨ Key Features

### 🕵️ Real-Time Network Intelligence
- **Deep Visibility**: Track active TCP/UDP connections with sub-second latency.
- **Smart Filtering**: Exclude common ports (e.g., 80/443) to quickly isolate unusual or suspicious traffic.
- **Geospatial Insights**: Live IP geo-location indicators (via ip-api.com) and batch lookup support.
- **Rapid Navigation**: Navigate hundreds of connections instantly via live search (`/`).

### ⚙️ Advanced Process & Container Management
- **System Enumeration**: Complete process visibility, exposing executable paths and command-line arguments.
- **Resource Profiling**: Monitor process-level CPU and memory consumption in real time.
- **Container Integration**: Monitor Docker container lifecycles, resource usage (CPU/Mem), network stats, and execute internal console commands directly from the UI.
- **Secure Actions**: Terminate suspicious processes or drop their existing network connections instantly.

### 🔍 Deep Investigation Suite
- **Comprehensive IP Analysis**: Extract geographic locations, ISPs, ASN info, timezones, and connection types (mobile/hosting).
- **Network Diagnostics**: Built-in DNS resolution, WHOIS lookups, ping latency, and traceroutes with geographic mapping.
- **Automated Risk Scoring**: Heuristic threat evaluation based on domain-to-process mismatches, anonymity usage (VPN/Tor), and latency anomalies.

### 🛡️ Firewall & Policy Management
- **One-Click Blocking**: Select individual endpoints and block them directly via system firewall.
- **Rules Review**: Browse blocked IPs list and execute batch block/unblock operations with ease.

### 💻 Premium User Experience
- **Nerd Font Integration**: Optional enhanced iconography with JetBrains Mono Nerd Font.
- **Global Reach**: 9 built-in localizations (EN, ES, FR, DE, IT, PT, JA, ZH, RU).
- **Adaptive Layout**: Seamlessly resizes to any terminal dimensions without breaking the UI.

---

## 🌐 External Dependencies

TraceTUI connects to the following external services at runtime to provide deep insights:

| Service | URL | Purpose |
|---|---|---|
| **ip-api.com** | `http://ip-api.com/json` | Geo-location of remote IP addresses (city, country, ISP, coordinates). |
| **ip-api.com (Batch)** | `http://ip-api.com/batch` | Bulk geoIP lookups for performance optimization in analysis. |
| **GitHub API** | `https://api.github.com/repos/...` | Startup version checker (verifies local against remote release). |
| **Google Search** | `https://www.google.com/search?q=` | Performs web queries via the "Search Online" feature. |
| **Nerd Fonts** | `https://github.com/ryanoasis/...` | Downloads the JetBrainsMono Nerd Font if the user chooses to install it. |
| **WHOIS** | Various Registries | Queries regional WHOIS registries for network block and domain insights. |

All URLs are securely managed and centralized under `resources/external_urls.json` via compile-time loading.

---

## 🚀 Installation

TraceTUI requires **Rust 1.70+**. Administrator privileges are recommended to fully utilize packet inspection and firewall features.

### 📥 Pre-built Binaries (Recommended)

Get the latest executable from the [Releases page](https://github.com/AcoranGonzalezMoray/TraceTUI/releases) or directly from the [Official Website](https://acorangonzalezmoray.github.io/TraceTUI/).
*(Note: TraceTUI contains a self-updating mechanism to check for newer versions on startup.)*

**Windows (PowerShell - Run as Administrator):**
```powershell
Invoke-WebRequest -Uri "https://github.com/AcoranGonzalezMoray/TraceTUI/releases/latest/download/tracetui-x86_64-pc-windows-gnu.zip" -OutFile "$env:TEMP\tracetui.zip"
Expand-Archive -Path "$env:TEMP\tracetui.zip" -DestinationPath "$env:TEMP\tracetui" -Force
& "$env:TEMP\tracetui\installOrUpdate.ps1"
```

**Linux:**
```bash
curl -L -o /tmp/tracetui.tar.gz "https://github.com/AcoranGonzalezMoray/TraceTUI/releases/latest/download/tracetui-x86_64-unknown-linux-gnu.tar.gz"
tar xzf /tmp/tracetui.tar.gz -C /tmp
chmod +x /tmp/installOrUpdate.sh
sudo sh /tmp/installOrUpdate.sh
```

### 🛠️ Build from Source
```bash
git clone https://github.com/AcoranGonzalezMoray/TraceTUI.git
cd TraceTUI
cargo build --release
./target/release/tracetui
```

---

## ⌨️ Quick Start

Navigate the TUI freely using the following core keybindings:

| Action | Shortcut |
| :--- | :--- |
| **Navigate Panels** | `Tab` / `Shift+Tab` |
| **Move Up / Down** | `↑` `↓` `PgUp` `PgDn` |
| **Deep Investigate** | `Enter` (on a connection/endpoint) |
| **Close / Exit View** | `Q` or `Esc` |
| **Search Bar** | `/` |
| **Toggle Nav Sidebar** | `M` |
| **Kill Process**| `X` |
| **Kill App Connections** | `-` |
| **Toggle Firewall Mode** | `B` |
| **Export to JSON** | `S` |
| **Language Modal** | `L` |

> *Tip: TraceTUI is fully capable of operating in the background. Press `H` to toggle "Hunter Mode" and filter out safe background processes.*

---

## 🏗️ Architecture

TraceTUI is heavily optimized for zero-blocking UI rendering, employing asynchronous systems and comprehensive separation of concerns.

### 🗂️ Project Structure

```text
src/
├── main.rs                 # Entry point
├── app/
│   ├── mod.rs              # App struct (12 state fields), shared methods
│   ├── states/             # 12 state structs for different views
│   ├── services/           # Background polling, investigation, inputs
│   ├── network/            # NetworkAnalyzer, connection parsing
│   ├── process/            # ProcessManager, ProcessInfo
│   └── ui/                 # UI render dispatch (Ratatui modules)
├── config/                 # Constants, thresholds, settings
├── i18n/                   # Translation engine
├── services/               # HTTP client, GeoIP service
└── utils/                  # DB, cache, formatting, WHOIS, rate limiter
test/
├── app/                    # Unit tests
└── E2E/                    # End-to-end integration tests
```

### 🧠 Key Design Decisions

- **Frontend**: [Ratatui](https://github.com/ratatui-org/ratatui) backend renders high-framerate ANSI screens.
- **Concurrency**: [Tokio](https://tokio.rs/) manages non-blocking geospatial queries and threat analysis independent of UI loops.
- **State Storage**: [SQLite](https://www.sqlite.org/) reliably caches investigation paths, metadata, and firewalled endpoints natively.
- **Safety**: Fully structured unit tests (`src/`) and E2E integration tests (`test/E2E/`) validate runtime stability and code safety across environments.

---

## 🤝 Contributing

We want TraceTUI to be the de-facto terminal standard for system network inspection. Contributions of any size—bug reports, feature requests, documentation improvements, or code modifications—are greatly appreciated!

1. Read our [Contributing Guidelines](CONTRIBUTING.md).
2. Fork the repository & create a feature branch.
3. Submit a Pull Request.

Make sure to run linting and tests before submitting:
```bash
cargo fmt && cargo clippy
cargo test
```

## 📜 License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for more information.

---
<div align="center">
  <b>Built with 🦀 Rust</b><br>
  <a href="https://github.com/AcoranGonzalezMoray/TraceTUI/issues">Report a Bug</a> • <a href="https://github.com/AcoranGonzalezMoray/TraceTUI/issues">Request a Feature</a>
</div>
