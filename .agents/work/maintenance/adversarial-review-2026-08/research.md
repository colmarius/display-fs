# Adversarial Review Findings (2026-08-16)

All findings verified against commit 21e6a98 in an Amp orb (Linux, stable Rust). `cargo build`, `cargo test` (38 pass), fmt, and clippy were green before review — the issues below are not caught by the existing suite.

## Confirmed bugs (reproduced)

### B1. Infinite loop in `truncate_to_fit` / `truncate_to_fit_for_display` (src/text.rs)

The loop truncates to a **fixed** `limit - 1` characters each iteration instead of shrinking progressively:

```rust
while !result.is_empty() && !fits_in_width(&result, font_size) {
    result = result.chars().take(limit.saturating_sub(1)).collect();
}
```

`limit` derives from `calculate_max_chars_per_line`, which estimates width from the `'x'` glyph advance. A long word of wider glyphs (e.g. `"W".repeat(60)`) still exceeds the display width at `limit - 1` chars, so the loop never terminates.

**Repro:** `split_into_pages(&"W".repeat(60), 14.0)` hangs; test killed by `timeout 30`. Any user running `display-fs show <long-URL-or-token>` can hang the CLI.

### B2. Documented default command does not exist (src/main.rs, justfile, README.md, AGENTS.md)

`Cli.command` is `Option<Commands>` with no default-subcommand wiring, so:

- `display-fs "Hi"` → `error: unrecognized subcommand 'Hi'` (exit 2)
- `display-fs --detect` → `error: unexpected argument '--detect'` (exit 2)

Broken callers: `just run`, `just detect`, README/AGENTS.md `cargo run -- "Hello World!"` and `cargo run -- --detect`, and the clap doc-comment "Display text on the screen (default command)".

### B3. Spotify state changes are missed (src/main.rs `run_spotify`)

`last_track` compares only `(track, artist)`, so play→pause on the same track never refreshes the ♪/|| prefix. Also, when Spotify is not running, `current == last_track == None` on the first iteration, so "Spotify not running" is never displayed and a non-loop invocation exits silently.

## Dead / duplicated code

- **src/image.rs (704 lines):** every `*_for_display` function has a near-identical legacy `*_oriented` / default-dimension twin (`create_text_image`, `calculate_auto_fit_size`, `calculate_max_chars_per_line`, `calculate_max_lines`, `image_to_rgb565_bytes`, `create_blank_image`, …). `main.rs` uses only the `_for_display` variants. The legacy set hard-codes 80x160 and exists only for tests.
- **src/text.rs (356 lines):** `split_into_pages`/`wrap_text`/`fits_in_width`/`truncate_to_fit` fully duplicated as `_for_display` variants (same bug B1 in both copies).
- **src/protocol.rs (222 lines):** `create_bitmap_header`, `create_bitmap_header_oriented`, `send_image_to_display`, `send_image_to_display_oriented` unused by the binary; `create_bitmap_header_for_display_oriented` takes an `_orientation` param it ignores.
- **Font parsing repeated:** `FontRef::try_from_slice(FONT_DATA)` re-parsed in 7+ functions per call. A `OnceLock` accessor removes the duplication and repeated cost.
- **main.rs:** the find-port → override-config → open-connection block and the render→convert→send block are each copy-pasted three times (`run_demo`, `run_spotify`, `display_text`). `detect_display` enumerates ports twice (`is_display_connected()` then `find_display_port()`).
- **Tautological tests:** e.g. `assert!(result == true || result == false)` in port.rs; several tests assert only "returns without panicking".

## Docs / site / workflow issues

- **Cargo.toml `description`** and **clap `about`**: say "0.96 inch" only; project supports the 3.5" model.
- **site/index.html install card:** `just build` then `./display-fs show "Hello"` — the binary lands in `target/release/`; the working command is `just install`.
- **README/AGENTS.md:** `cargo run -- "Hello World!"` and `cargo run -- --detect` broken (B2).
- **CI (.github/workflows/ci.yml):** no Rust caching (full rebuild each run); the `japanese` feature is never compiled in CI even though the NotoSansJP font is committed and a `#[cfg(feature = "japanese")]` test exists.
- **pages.yml:** fine (path-scoped trigger, correct permissions, concurrency group).

## Non-issues checked

- RGB565 conversion, rotation index math for all four orientations, and the large-display bitmap header math are correct (verified by reading the index mapping and existing tests).
- `--once` flag is a no-op but documents default behavior and `conflicts_with = "loop"`; kept for CLI compatibility.
- `.agents/setup` / `.amp/services.yaml` / resume scripts are consistent with AGENTS.md.
