# Contributing to TraceTUI

First off, thank you for considering contributing to TraceTUI! It's people like you that make TraceTUI such a great tool for the community.

## Code of Conduct

By participating in this project, you are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check our issue tracker as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

*   **Use a clear and descriptive title** for the issue to identify the problem.
*   **Describe the exact steps which reproduce the problem** in as many details as possible.
*   **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
*   **Explain which behavior you expected to see instead and why.**
*   **Include screenshots** if the problem is visual (TUI layout issues, etc.).

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please:

*   **Use a clear and descriptive title.**
*   **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
*   **Explain why this enhancement would be useful** to most TraceTUI users.

### Pull Requests

1.  **Fork the repository** and create your branch from `main`.
2.  **If you've added code that should be tested, add tests.**
3.  **If you've changed APIs, update the documentation.**
4.  **Ensure the test suite passes.**
5.  **Make sure your code lints** (`cargo clippy`).
6.  **Format your code** (`cargo fmt`).

## Style Guide

### Rust Style

Please follow the standard Rust style guidelines. We use `rustfmt` to enforce a consistent coding style across the project.

### Commit Messages

Every commit **MUST** follow this format:

```
[EMOJI]: MESSAGE
```

The emoji **MUST** be chosen from [gitmoji.dev](https://gitmoji.dev/) based on the type of change:

| Change type | Emoji | Example |
|---|---|---|
| New feature | `:sparkles:` (✨) | `✨: Add continuous analysis with pause/resume toggle` |
| Bug fix | `:bug:` (🐛) | `🐛: Fix mouse offset calculation in dashboard click` |
| Refactor | `:recycle:` (♻️) | `♻️: Extract hardcoded URLs to centralized resources module` |
| Tests | `:test_tube:` (🧪) | `🧪: Add E2E tests for analysis lifecycle and firewall flow` |
| Documentation | `:memo:` (📝) | `📝: Update CONTRIBUTING with commit convention` |
| CI/CD | `:green_heart:` (💚) | `💚: Fix clippy warnings and failing panel_from_x tests` |
| Release | `:rocket:` (🚀) | `🚀: Deployed V0.0.3 for Windows/Linux` |
| Configuration | `:wrench:` (🔧) | `🔧: Add DEFAULT_TERM_WIDTH/HEIGHT config constants` |
| Dependencies | `:heavy_plus_sign:` (➕) | `➕: Add once_cell dependency for lazy static URLs` |

Additionally:

*   Use the present tense ("Add feature" not "Added feature")
*   Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
*   Limit the first line to 72 characters or less
*   Reference issues and pull requests liberally after the first line

### Changelog (`VERSIONS.md`)

Every PR **MUST** update `VERSIONS.md` in the `# UNRELEASED` section with:

```
## YYY-MM-DD VX.X.X
LIST OF CHANGES
```

Each change **MUST** be listed with its gitmoji, matching the commit convention. Do **NOT** include a date — the release pipeline adds it automatically when promoting to `# RELEASED`:
Example
```
## UNRELEASED

## 2026-05-15  V0.0.4
✨: Continuous analysis with pause/resume toggle and background refresh cycle
🧪: E2E tests for analysis lifecycle, firewall flow, and export/investigation
🧪: JSON export test with structure validation and auto-cleanup
```

The release pipeline reads `VERSIONS.md` and updates `Cargo.toml` automatically, but both files must be in sync before merging.

## Setting Up Development Environment

1.  Install Rust (1.70 or newer).
2.  Clone the repository: `git clone https://github.com/AcoranGonzalezMoray/TraceTUI.git`
3.  Build the project: `cargo build`
4.  Run tests: `cargo test`

Thank you for your contributions!
