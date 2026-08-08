# Project: Display FS V1 (0.96 inch + 3.5 inch)

## Overview

Standalone CLI application to interact with the Display FS V1 family (0.96 inch + 3.5 inch), detect if it's connected, and display content.
Spotify now-playing output is width-aware on both display sizes.

## Tech Stack

- Language: Rust 2021
- CLI: Clap
- Display I/O: `serialport`
- Image and text rendering: `image`, `imageproc`, and `ab_glyph`
- Task runner: `just`
- Testing and quality: Cargo tests, rustfmt, and Clippy

## Hardware

- **Device (small):** WeAct Studio Display FS V1 (0.96 inch IPS LCD)
  - **Resolution:** 80x160 pixels (portrait orientation)
  - **Communication:** USB CDC/serial (UART), 115200 baud (current CLI)
  - **USB Chip:** CH340/CH341 USB-Serial converter
  - **Known VID/PID:** CH340 (1A86:7523), CH341 (1A86:5523)
- **Device (large):** WeAct Studio Display FS V1 (3.5 inch IPS LCD)
  - **Resolution:** 320x480 pixels (portrait orientation)
  - **Communication:** USB2.0 FS (CDC); reference Python app uses 1,152,000 baud
  - **Sensors:** Humidity + Temperature

## Workflow

```text
Request or change
├─ Self-contained ───────────▶ Plan and execute in this conversation ─▶ Verify and report
└─ Continuity has value ─────▶ Work Item → Context as needed → Plan → Execute → Verify
                                                                    ├─ Hand off when useful
                                                                    └─ Promote → Commit snapshot → Remove
```

Keep small, self-contained planning and execution in the current conversation. Create a work item when resumption, coordination, handoff, auditability, durable decisions, or an explicit request justifies repository context. Implement in the current thread by default and hand off only when another worker or environment genuinely helps.

Durable work lives under `.agents/work/<category>/<slug>/`. Follow `.agents/work/AGENTS.md` for the canonical artifact, status, handoff, and completion contract. Existing files under `.agents/plans/` predate this workflow; preserve them as legacy project records, but use `agent-work` for new durable work.

## Project Structure

```text
display-fs/
├── AGENTS.md                    # This file - project instructions
├── Cargo.toml                   # Rust project configuration
├── .amp/
│   └── services.yaml            # Supervised static-site portal
├── .agents/
│   ├── setup                    # Fresh-orb dependency setup
│   ├── resume                   # Fast orb wake-up checks
│   ├── work/                    # Durable work items and canonical state
│   ├── research/                # Reusable cross-work research
│   ├── references/              # Local reference checkouts (gitignored)
│   ├── plans/                   # Legacy project plans (preserved)
│   ├── scripts/                 # dot-agents sync helpers
│   └── skills/                  # Reusable agent workflows
│       ├── adapt/               # Refresh project guidance
│       ├── agent-browser/       # Browser workflow discovery
│       ├── agent-work/          # Durable work management
│       └── research/            # Technical research workflow
├── .github/
│   └── workflows/               # GitHub Actions workflows (Pages, CI)
├── site/                        # Static site for GitHub Pages
│   ├── index.html               # Landing page
│   └── styles.css               # Site styles
├── src/                         # Rust source modules
│   ├── main.rs                  # CLI entry point
│   ├── lib.rs                   # Library exports
│   ├── port.rs                  # COM port detection and connection
│   ├── image.rs                 # Image creation and RGB565 conversion
│   ├── protocol.rs              # Display command protocol
│   ├── spotify.rs               # Spotify now-playing integration (macOS)
│   └── text.rs                  # Text wrapping and pagination
└── assets/
    └── fonts/                   # Font files for text rendering
        └── DejaVuSans.ttf       # Embedded in Rust binary
```

## Using Skills

| Command | Effect |
| --- | --- |
| `Run adapt` | Refresh `AGENTS.md` from verified project facts |
| `Use agent-browser to verify ...` | Discover and run the installed browser workflow |
| `Create a new work item for ...` | Create durable context under `.agents/work/` |
| `Research [topic]` | Save work-local or reusable technical findings |
| `Create/execute a plan for ...` | Plan and implement in the current thread |
| `Write a handoff prompt for ...` | Produce a paste-ready transition prompt |

Skills are loaded through natural-language requests. Their procedures live under `.agents/skills/`; do not copy those details into this file.

## Commands

### Quick Commands (using `just`)

```bash
# Install just (if not installed)
brew install just  # or: cargo install just

# Show available commands
just

# Development workflow
just check         # Fast type-check (no codegen)
just check-jp      # Type-check with Japanese/CJK support
just lint          # Run clippy lints
just fmt           # Format code
just test          # Run tests
just ci            # Full check: fmt + lint + test

# Build and run
just build         # Build release binary
just build-jp      # Build release binary with Japanese/CJK support
just install       # Build and update ./display-fs
just install-jp    # Build and update ./display-fs with Japanese/CJK support
just run "Hi"      # Run with custom text
just detect        # Detect display
just docs-open     # Open docs site in browser
just docs-serve    # Serve docs at http://localhost:8000
```

### Direct Cargo Commands

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux (Ubuntu/Debian): install libudev for serial port support
sudo apt-get update && sudo apt-get install -y libudev-dev

# Build
cargo build              # Debug build
cargo build --release    # Release build

# Run
cargo run -- "Hello World!"
cargo run -- --detect

# Quality checks
cargo fmt
cargo clippy -- -D warnings
cargo test
```

### Amp Orb Lifecycle

Fresh orbs run `.agents/setup` to install Rust, native build dependencies, `just`, and locked Cargo dependencies. Resumed orbs run `.agents/resume`, which ensures the supervised static-site portal is available.

```bash
# Reconcile the services declared in .amp/services.yaml
amp orb services ensure
```

### Display Orientation

Use `--orientation` to pick landscape or portrait and `--flip` to rotate output 180° (useful for upside-down installs).

## Git Workflow

Use plain git commands for version control.

```bash
git status
git add -A
git commit -m "Description of changes"
git log --oneline
git push
```

### Commit Guidelines

- Write clear, descriptive commit messages
- Reference the active work item in commits when it helps preserve context
- Commit after each logical step; do not wait until unrelated phases accumulate
- Do not push unless the user explicitly requests it

## Maintenance

After making changes to the codebase, always:

1. **Update AGENTS.md** - Keep project structure and commands current
2. **Update README.md** - Reflect user-facing changes (new features, usage examples)
3. **Update durable work** - Keep the active work item's index, plan, and progress evidence synchronized when applicable
4. **Run tests** - Verify changes with `cargo test`

## Conventions

- Follow Rust 2021 idioms and the repository's `.rustfmt.toml`.
- Keep reusable device behavior in the library modules under `src/`; keep argument parsing and command orchestration in `src/main.rs`.
- Add focused unit tests beside the module they exercise.
- Keep default builds Latin-only; gate Japanese/CJK font support behind the `japanese` Cargo feature.

## Architecture Notes

- `port.rs` detects display models and opens serial connections.
- `image.rs` renders display-sized images and converts them to RGB565 bytes.
- `protocol.rs` sends encoded image data using the device protocol.
- `text.rs` owns wrapping and pagination; `spotify.rs` owns macOS now-playing integration.
- `main.rs` combines these modules into the `show`, `spotify`, preset, and demo CLI flows.
