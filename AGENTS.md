# Project: Display FS V1 (0.96 inch + 3.5 inch)

## Overview

Standalone CLI application to interact with the Display FS V1 family (0.96 inch + 3.5 inch), detect if it's connected, and display content.
Spotify now-playing output is width-aware on both display sizes.

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
│   ├── research/                # Research and reference material
│   ├── plans/                   # Implementation plans
│   │   ├── todo/                # Planned but not started
│   │   ├── in-progress/         # Currently being worked on
│   │   └── completed/           # Finished and verified
│   └── skills/                  # Agent skills
│       ├── ralph/               # Autonomous implementation loops
│       └── research/            # Deep research workflow
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

## Plan Management

Plans in `.agents/plans/` follow this workflow:

| Status | Description |
|--------|-------------|
| **TODO** | Planned but not started |
| **IN-PROGRESS** | Currently being worked on |
| **COMPLETED** | Finished and verified |

Each plan file has a `Status:` field at the top to track progress.

### Writing Ralph-Ready Plans

Plans intended for autonomous execution with the `ralph` skill must use this task format:

```markdown
- [ ] **Task N: Short descriptive title**
  - Scope: `path/to/affected/files` or module name
  - Depends on: Task M (or "none")
  - Acceptance:
    - Specific, verifiable criterion 1
    - Specific, verifiable criterion 2
  - Notes: Optional implementation hints
```

**Task sizing rule:** If you can't describe the task in 2-3 sentences, split it.

**Task ordering:** Dependencies flow downward. Common order: Schema → Service → API → CLI → Tests

**Task markers:**

| Marker | Meaning |
|--------|---------|
| `- [ ]` | Not started |
| `- [x]` | Completed |
| `- [ ] (blocked)` | Blocked, needs intervention |
| `- [ ] (manual-verify)` | Requires manual verification |

**Running a plan:**

```bash
# Start fresh
Run ralph on .agents/plans/in-progress/003-auto-fit-text.md

# Resume from specific task
Continue ralph from Task 3 on .agents/plans/in-progress/003-auto-fit-text.md
```

## Commands

### Quick Commands (using `just`)

```bash
# Install just (if not installed)
brew install just  # or: cargo install just

# Show available commands
just

# Development workflow
just check         # Fast type-check (no codegen)
just lint          # Run clippy lints
just fmt           # Format code
just test          # Run tests
just ci            # Full check: fmt + lint + test

# Build and run
just build         # Build release binary
just install       # Build and update ./display-fs
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
- Reference plan numbers in commits (e.g., "Plan 001: Initialize Rust project")
- **Commit after each logical step** - Don't wait until everything is done; commit when a phase or meaningful unit of work is complete

## Maintenance

After making changes to the codebase, always:

1. **Update AGENTS.md** - Keep project structure and commands current
2. **Update README.md** - Reflect user-facing changes (new features, usage examples)
3. **Update plan status** - Mark plans as COMPLETED when finished
4. **Run tests** - Verify changes with `cargo test`
