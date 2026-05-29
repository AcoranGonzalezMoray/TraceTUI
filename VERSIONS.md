# UNRELEASED

## 2026-05-29  V2.0.1
🐛: Fixed export functionality on Linux and kill process action on Windows/Linux

🐧: **Linux Parity** — Native file explorer dialog for export actions (Zenity → KDialog → Python/Tkinter fallback chain) now applied consistently in Networks and Libraries sections

🚀: **Action Loaders** — Visual loading modal while killing processes or closing connections, with i18n status messages

⌨️: **Improved Controls** — `Enter` key now confirms action dialogs across all panels alongside the existing `Y`/`S` keys

🌐: **i18n Polish** — Fully synchronized and translated `status.closing_connections`, `status.killing_process` and `status.please_wait` across all 9 supported languages (EN, ES, FR, DE, IT, PT, RU, ZH, JA)

🛡️: **System Stability** — Fixed process termination logic for Windows (`taskkill /F`) and Linux (`kill -9`) with better access-denied and not-found error messages

🧹: **Code Quality** — Resolved all Clippy lint warnings (unused imports, useless `as_ref`, etc.) and cleaned up `input_service.rs`

🎨: **UI Improvements** — Fixed image visualization in the Linux storage explorer; improved icon extraction feedback

# RELEASED

## 2026-05-29  V2.0.0
✨: **Docker Container Monitoring** — Full lifecycle management (start/stop/restart/pause/unpause), real-time CPU/Memory/Net I/O stats, logs viewer, interactive console inside containers

✨: **Docker Hub Integration** — Search images from Docker Hub, configure ports/env vars, and create containers directly from the TUI

✨: **Docker Service Control** — Start/Stop Docker Desktop (Windows) or Docker daemon (Linux) from within the app

✨: **Storage Explorer** — Disk usage visualization with usage bars, file browser with directory navigation, multiple sort modes (name/size/date)

✨: **File Search** — Recursive file search across directories with extension filters (images, documents, code, archives, audio, video) and abort support

✨: **File Viewer** — View text/log/ANSI files inline, image preview via PowerShell/chafa/catimg/Pillow

✨: **Library Inspection** — Per-process loaded DLL/library listing, digital signature verification (Signed/Unsigned), origin classification (System/ProgramFiles/UserSpace/Temp), risk assessment

✨: **Binary Viewer** — Hex dump and x86 disassembly viewer (iced-x86 engine) for suspicious libraries

✨: **Hash Information** — SHA-256 computation for loaded libraries

✨: **Library Risk Filtering** — Filter libraries by risk level with export to JSON/CSV

✨: **Trends & Analytics View** — Real-time CPU and connection count sparklines, risk distribution chart, top processes by CPU/memory, protocol breakdown, destination country stats

✨: **File Sorter** — Toggle between name, size, and date sort modes in file browser

✨: **Hunter Mode** — `H` key toggle to filter unsigned/unknown processes, auto-hide known-safe signed processes

✨: **Nerd Font Installer** — One-click JetBrains Mono Nerd Font download and installation dialog on first launch

✨: **Installation Module** — In-app installer with sudo password modal (Linux), PowerShell-based install (Windows)

✨: **Update Progress Bar** — Visual download progress during self-update with percentage indicator

✨: **Improved Welcome Dialog** — Multi-step welcome tour shown after version upgrades

✨: **Nav Sidebar** — Collapsible `M` key sidebar with 5 views (Main, Trends, Storage, Libraries, Containers)

✨: **Center Tab Switching** — `[1] Connections`, `[2] Risk Overview`, `[3] Timeline` tabs in center panel

✨: **Enhanced Filtering** — `F` key high-risk filter now works alongside search query

🐛: Fix initial loading state blocking UI before first data pull

🐛: Handle empty docker ps output gracefully (no containers)

🐛: Fix locale not persisting on version upgrade

♻️: Refactor App struct with 12 state modules for separation of concerns

🔧: Update Cargo.toml dependencies — add iced-x86, sysinfo 0.30, lru 0.12

## 2026-05-17  V1.1.2
🔧: PR workflow now also checks that Cargo.toml version is bumped (compared to base branch)
✨: Update success/failure modal now shows ✅/❌ emojis instead of [OK]/[FAIL]

## 2026-05-17  V1.1.1
🐛: Fix self-update failing on Windows (running exe lock) and Linux (permissions)
🐛: Store database in OS app data directory to avoid admin rights requirement
🐛: Fix locale not persisting on first launch — now detects and saves system language
🐛: Fix install script CRLF/BOM issues causing "not found" errors on Linux
🐛: Fix Linux install script Bad substitution and sudo HOME path detection
♻️: Change install paths to user-writable directories (~/.local/bin, %LOCALAPPDATA%)
🔧: Force LF line endings for .sh/.desktop files via .gitattributes
🧪: Add Linux icon extractor tests and fix flaky CI tests due to welcome dialog
💚: Fix clippy warnings in test builds (unused-mut, unused-variable)

## 2026-05-17  V1.1.0
✨: Added installation module with self-update detection and auto-update dialogs
🌐: Update and sync translations across all 11 supported languages
🐛: Fix config bug that reset user locale on every application startup
🐛: Resolve UI freeze issue during initial network analysis by optimizing event polling
💄: Improve Welcome Dialog UI with better text wrapping button styling
🐛: Fix English messages appearing in Spanish locale for update success notifications
🔧: Enhance icon extraction path discovery for both development and production environments
🐛: Fix update notification flow to ensure welcome modal appears after successful update
♻️: Refactor startup logic to handle system locale detection more efficiently

## 2026-05-16  V1.0.0
✨: Real-time Network Intelligence
- Deep Monitoring: Track active TCP/UDP connections with sub-second latency
- Smart Filtering: Exclude common ports (80/443) to focus on unusual traffic
- Geo-Location: Visual indicators for remote connection endpoints using ip-api.com
- Batch GeoIP Lookup: Efficient bulk IP lookups for improved performance
- Sort & Search: Navigate through hundreds of connections with live search (`/` key)
- Filter High Risk: Show only suspicious connections (`F` key)

✨: Advanced Process Management
- System Enumeration: Full process visibility including paths and command lines
- Resource Tracking: Real-time CPU and memory usage per process
- Secure Termination: Kill suspicious processes with multi-step confirmation (`X` key)
- Connection Termination: Kill all connections for a process (`-` key)
- Window Integration: (Windows only) Extract application icons and metadata
- Clipboard Integration: Copy process paths to clipboard (`Ctrl+C` or `C` key)
- Online Search: Search for process information online (`G` key)

✨: Deep Investigation Suite
- IP Investigation: Detailed analysis of remote IPs including:
  - Geographic location (city, country, coordinates)
  - ISP and organization details
  - ASN and network information
  - Timezone and connection type (mobile/proxy/hosting)
- DNS Lookup: Forward and reverse DNS resolution (`nslookup`/`dig`)

- Network Diagnostics: 
  - Ping latency measurement
  - Traceroute with geographic hop mapping
  - WHOIS record lookup

- Risk Assessment: Automated risk scoring based on:
  - Domain/process mismatch detection
  - Network anonymity indicators (proxy, VPN, Tor)
  - Latency anomalies
  - Hosting provider and mobile network detection
  
- Visual Mapping: Interactive map view of connection routes and endpoints

✨: Automated Batch Analysis
- Risk Scoring: Detect suspicious network patterns and orphaned processes
- Heuristic Analysis: Identify threats based on connection frequency and behavior
- Hunter Mode: Auto-filter known-safe signed processes to surface unknowns (`H` key)
- JSON Export: Export full analysis to timestamped JSON files (`S` key or Action #5)
- Pause/Resume: Temporarily halt background analysis (`R` key)
- Manual Refresh: Trigger immediate analysis update (`Ctrl+R` key)

✨: Firewall Management
- Per-Connection Blocking: Select individual connections to block via Windows Firewall
- Blocked IPs Viewer: Review and unblock previously blocked addresses
- Batch Operations: Block/unblock multiple connections at once
- Firewall Mode: Toggle firewall management (`B` key or Action #7)

✨:User Experience
- Full Input Support: Comprehensive keyboard shortcuts and mouse interaction
- Adaptive Layout: Auto-scales panels based on terminal size
- Multi-language: Built-in i18n with 9 locales (EN, ES, FR, DE, IT, PT, JA, ZH, RU) (`L` key or Action #8)
- Nerd Font Support: Optional JetBrains Mono Nerd Font for enhanced iconography
- System Tray: Windows system tray integration for background operation
- Update Checking: Automatic version checks with GitHub releases
- Installation Helpers: Scripts for easy setup and dependency installation

✨: Investigation Panels
- Connections View: Detailed table of network connections with filtering
- Risk Analysis: Process risk scoring and threat indicators
- Timeline View: Historical activity tracking and trends
- Map View: Geographic visualization of connection routes
- Process Details: Executable paths, signatures, and resource usage
- Firewall Management: Connection blocking and IP allowlist/blocklist

✨: Landing page terminal mockup now matches Rust TUI exactly
