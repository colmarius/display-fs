# Plan: fix bugs, remove dead code, correct docs

Scope: minimal, behavior-preserving except for the three bug fixes. The library is consumed only by this repo's binary, so removing unused public API is safe. Keep the explicit `show` subcommand (no default-subcommand clap plumbing); fix the callers/docs instead — less complexity.

## Tasks

- [x] T1. Fix B1: rewrite word truncation to shrink progressively (`pop()` until it fits); add regression test with a long wide-glyph word.
- [x] T2. Fix B3: track the full `NowPlaying` state (including `is_playing` and the not-running case) so pause/resume and "Spotify not running" render correctly.
- [x] T3. Remove legacy duplicate APIs:
  - image.rs: drop `_oriented`/default-dimension variants; rename `_for_display` variants to the plain names taking `(text, font_size, orientation, width, height)`; add a `OnceLock` font accessor; port tests to the retained API.
  - text.rs: same collapse for `split_into_pages`/`wrap_text`/`fits_in_width`/`truncate_to_fit`.
  - protocol.rs: keep one `send_image_to_display(port, config, data, orientation)`; drop ignored `_orientation` param from the header builder; delete legacy 80x160 header/send functions.
  - lib.rs: update exports.
- [x] T4. main.rs cleanup: extract shared connect helper and render+send helper; `detect_display` enumerates ports once; update clap `about` to cover both models.
- [x] T5. Docs: fix justfile `run`/`detect` recipes to use `show`; fix README/AGENTS.md `cargo run` examples; remove "(default command)" claim; update Cargo.toml description; fix site install card to `just install`.
- [x] T6. CI: add `Swatinem/rust-cache@v2`; add a `japanese` feature test step.
- [x] T7. Verification (see below) and AGENTS.md maintenance sync.

## Observed verification evidence (2026-08-16, Amp orb, no USB display attached)

- Regression test `text::tests::test_long_wide_glyph_word_terminates` passes in 0.01s; before the fix the identical input hung until killed by `timeout 30`.
- `cargo test`: 27 passed, 0 failed. `cargo test --features japanese`: 28 passed, 0 failed (this run exposed and fixed a latent font-dependent threshold in `test_auto_fit_long_text_smaller`, which failed at 38px under Noto Sans JP; now a relative assertion).
- `cargo fmt -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo build --release` succeeds.
- `./target/release/display-fs show "Hi"` and `show --detect` run and exit 1 with the "not found" message (expected without hardware); before the justfile/doc fix, the documented forms `display-fs "Hi"` / `display-fs --detect` exited 2 with clap parse errors.
- `just run "Hi"` and `just detect` now reach the device-lookup path (exit 1 no-device) instead of failing argument parsing.
- Net change: −604 lines (src/ from 2282 to 1673 lines) with the same CLI surface.
- Not verified (no hardware in orb): actual frames on a physical display; Spotify AppleScript path (macOS-only). Protocol byte layout and rotation mapping are covered by unit tests, including a new landscape rotation pixel-mapping test.

## Acceptance criteria

- `split_into_pages(&"W".repeat(60), 14.0, …)` terminates and returns non-empty pages (regression test in suite).
- `just run "Hi"` and `just detect` execute the CLI without argument errors (display-not-found failure is expected in the orb).
- Spotify state comparison includes `is_playing` and the not-running case (unit-testable formatting stays covered; state logic verified by review since macOS/Spotify are unavailable in the orb).
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and `cargo test --features japanese` all pass.
- Line count of src/ drops materially with no feature loss (binary supports the same CLI surface).

## Verification

- `just ci` (fmt + clippy + test) and `cargo test --features japanese`.
- Run the built binary: `show --detect`, `show "Hi"`, `presets` — verify graceful no-device behavior and exit codes.
- Hardware send paths cannot be exercised in the orb (no USB display); protocol byte-layout tests cover header/chunking.
