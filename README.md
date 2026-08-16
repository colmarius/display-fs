# Display FS V1

CLI tool to interact with the WeAct Studio Display FS V1 family (0.96 inch + 3.5 inch IPS LCDs).

## Features

- **Standalone executable** - No runtime dependencies after installation
- Auto-detect display via USB (CH340/CH341 USB-Serial)
- Display text on the 160x80 or 320x480 pixel screen
- Cross-platform support (Windows, Linux, macOS)

## Website

The GitHub Pages site lives in `site/` and deploys automatically on pushes to `main`.

```bash
# Open the static page or serve it locally
just docs-open
just docs-serve
```

In an Amp orb, `.agents/setup` installs the Rust development environment and starts the site through the supervised service declared in `.amp/services.yaml`. Run `amp orb services ensure` to reconcile it and print its portal URL.

## Agent Workflow

This repository uses [dot-agents](https://dot-agents.dev/) for reusable skills and durable work context. Project-specific instructions live in `AGENTS.md`; durable work items live under `.agents/work/`, while reusable research remains under `.agents/research/`.

```bash
# Preview upstream workflow updates
.agents/scripts/sync.sh --diff

# Sync the latest workflow from the configured main ref
.agents/scripts/sync.sh
```

## Hardware

| Specification | 0.96 inch                          | 3.5 inch                          |
|---------------|------------------------------------|-----------------------------------|
| Device        | WeAct Studio Display FS V1          | WeAct Studio Display FS V1        |
| Resolution    | 80x160 pixels (portrait)            | 320x480 pixels (portrait)         |
| Connection    | USB-C (serial)                      | USB 2.0 FS (CDC)                  |
| Baud Rate     | 115200                              | 1152000 (reference Python app)    |
| USB Chip      | CH340/CH341                          | USB CDC                           |
| Sensors       | None                                | Humidity + Temperature            |

## Model Detection

The CLI auto-detects the display model by VID/PID and USB product strings. If detection is
ambiguous, pass `--model small` or `--model large` and, if needed, override the baud rate with
`--baud-rate`.

## Quick Start

After [installing the CLI](#installation):

```bash
# Display text
display-fs show "Hello World!"

# Auto-fit text to largest readable size
display-fs show --auto "Hi"

# Check if display is connected
display-fs show --detect

# Custom font size
display-fs show -s 20 "Big Text"

# Show current Spotify track (macOS)
display-fs spotify --loop

# Run a preset (clock, git status, etc.)
display-fs preset clock --loop

# Demo mode: cycle through all presets
display-fs demo
```

## Installation

### Install with Cargo (Recommended)

Requires [Rust](https://rustup.rs/):

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Linux (Ubuntu/Debian): install libudev for serial port support
sudo apt-get update && sudo apt-get install -y libudev-dev

# Install the default Latin build from GitHub
cargo install --git https://github.com/colmarius/display-fs
```

Cargo installs `display-fs` into its binary directory (normally `~/.cargo/bin`, which Rustup adds to `PATH`).

### Build from Source

```bash
git clone https://github.com/colmarius/display-fs.git
cd display-fs

# Build a release binary at target/release/display-fs
cargo build --release

# Build with Japanese/CJK support (~6.5 MB)
cargo build --release --features japanese
```

#### Japanese/CJK Font Support

The default build uses DejaVuSans font (~750 KB) for Latin text. For Japanese song titles or CJK characters, build with the `japanese` feature:

```bash
# Install from a source checkout using just
just build-jp       # Build with Japanese support
just install-jp     # Install japanese-enabled binary

# Or install the feature-enabled build from GitHub
cargo install --git https://github.com/colmarius/display-fs \
  --features japanese --force
```

| Build | Font | Binary Size |
|-------|------|-------------|
| Default | DejaVuSans | ~1.5 MB |
| `--features japanese` | Noto Sans JP | ~6.5 MB |

### Driver

Install CH340/CH341 USB-Serial drivers if not automatically detected:

- **Windows:** Usually auto-installed
- **macOS:** [CH340 Driver](https://github.com/adrianmihalko/ch340g-ch34g-ch34x-mac-os-x-driver)
- **Linux:** Usually built into the kernel

## Usage

```text
display-fs show [OPTIONS] [TEXT]

Arguments:
  [TEXT]  Text to display [default: "Hello World!"]

Options:
  -s, --font-size <SIZE>        Font size in pixels; must be finite and positive [default: 14]
  -a, --auto                    Auto-fit text to largest readable size
  -o, --orientation <MODE>      Display orientation: landscape or portrait [default: landscape]
      --flip                    Flip the display 180° (use if the screen is upside down)
  -d, --delay <SECONDS>         Positive finite delay between pages [default: 2.0]
  -l, --loop                    Loop display continuously
      --detect                  Only check if display is connected
      --model <MODEL>           Force display model: small or large
      --baud-rate <BAUD>        Override baud rate (advanced)
  -p, --port <PATH>             Use a specific serial port instead of USB auto-detection
                                (model defaults to small unless --model is given)
  -h, --help                    Print help
```

### Auto-Fit Mode

The `--auto` flag automatically calculates the largest font size that fits your text on the 80x160 pixel display. Great for maximizing readability:

```bash
# Short text displays large
display-fs show --auto "Hi"        # Uses ~70px font

# Longer text uses smaller font to fit
display-fs show --auto "Hello!"    # Uses ~40px font
```

### Orientation Mode

The `--orientation` flag switches between landscape (160x80, default) and portrait (80x160) modes. Add `--flip` to rotate the output 180° in either orientation:

```bash
# Landscape (default) - wider display
display-fs show --auto "Wide text"

# Portrait - taller display, good for multi-line
display-fs show --auto -o portrait $'Line 1\nLine 2\nLine 3'
```

### Examples

```bash
# Default message
display-fs show

# Custom message
display-fs show "Hello from Rust!"

# Auto-fit (recommended)
display-fs show --auto "Status OK"

# Portrait orientation with auto-fit
display-fs show --auto -o portrait "Tall"

# Landscape orientation but flipped 180°
display-fs show --auto --flip "Upside down"

# Larger font (manual)
display-fs show -s 24 "BIG"

# Just detect display
display-fs show --detect
```

### Spotify Now Playing (macOS)

Display the currently playing Spotify track. The title/artist lines automatically truncate
to the current display width (including large-screen auto-fit):

```bash
# Show once
display-fs spotify

# Live updates (refresh every 2 seconds)
display-fs spotify --loop

# Live updates with flipped display
display-fs spotify --loop --flip

# Faster refresh
display-fs spotify --loop --speed fast
```

### Presets

Run built-in system information presets:

```bash
# List available presets
display-fs presets

# Run a preset
display-fs preset clock
display-fs preset git
display-fs preset ip
display-fs preset whoami

# Loop a preset (live updates)
display-fs preset cpu --loop

# Demo mode: cycle through all presets
display-fs demo
```

Available presets: `clock`, `datetime`, `uptime`, `git`, `ip`, `whoami`, `pwd`, `cpu`, `memory`, `docker`, `spotify`, `fortune`

## Project Structure

```text
display-fs/
├── Cargo.toml             # Rust project configuration
├── .amp/
│   └── services.yaml      # Supervised static-site portal for Amp orbs
├── .agents/
│   ├── setup              # Fresh-orb toolchain and dependency setup
│   ├── resume             # Fast orb wake-up checks
│   ├── work/              # Durable agent work items
│   ├── research/          # Reusable research notes
│   ├── references/        # Local reference checkouts (gitignored)
│   ├── scripts/           # dot-agents sync helpers
│   └── skills/            # Reusable agent workflows
├── .github/                # GitHub Actions workflows
│   └── workflows/          # GitHub Pages deploy workflow
├── site/                   # Static GitHub Pages site
│   ├── index.html          # Landing page
│   └── styles.css          # Site styles
├── src/                   # Rust source code
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library exports
│   ├── port.rs            # USB port detection
│   ├── image.rs           # Image creation & RGB565
│   ├── protocol.rs        # Display protocol
│   ├── spotify.rs         # Spotify now-playing (macOS)
│   └── text.rs            # Text wrapping & pagination
└── assets/
    └── fonts/             # Font files (embedded in binary)
```

## License

MIT License - see [LICENSE](LICENSE) for details.
