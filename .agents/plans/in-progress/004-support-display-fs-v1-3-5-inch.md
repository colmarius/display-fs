# Plan: Support Display FS V1 (3.5 inch)

Status: IN-PROGRESS

## Goal

Add first-class support for the 3.5-inch Display FS V1 in the CLI, including detection, resolution handling, protocol parity, and documentation, while keeping the 0.96-inch flow intact.

## Tasks

- [ ] **Task 1: Model detection + config baseline**
  - Scope: `src/port.rs`, `src/protocol.rs`, `src/main.rs`
  - Depends on: none
  - Acceptance:
    - CLI can identify which display type is connected (0.96 vs 3.5) using VID/PID or port description
    - A single configuration struct holds model-specific defaults (resolution, baud rate, orientation)
  - Notes: Add a `DisplayModel` enum and extend detection to use port description hints ("AB"/"AD") in addition to VID/PID.

- [ ] **Task 2: Protocol parity for 3.5-inch commands**
  - Scope: `src/protocol.rs`
  - Depends on: Task 1
  - Acceptance:
    - Implement read/write helpers for WHO_AM_I, brightness, and unconnect settings
    - Add humiture report command handling (enable/read) with a typed response
  - Notes: Align command bytes with WeAct protocol v1.1 and keep them model-agnostic where possible.

- [ ] **Task 3: Image pipeline for large display**
  - Scope: `src/image.rs`, `src/text.rs`, `src/protocol.rs`
  - Depends on: Task 1
  - Acceptance:
    - Rendering handles 320x480 and 480x320 orientations without clipping
    - Bitmap writes support chunked writes sized for the active model
  - Notes: Consider optional FastLZ support as follow-up if performance is inadequate.

- [ ] **Task 4: CLI options + UX updates**
  - Scope: `src/main.rs`, `README.md`, `site/index.html`, `site/styles.css`
  - Depends on: Task 1
  - Acceptance:
    - CLI flags allow forcing a model and baud rate override
    - Help text documents the 3.5-inch usage and sensor availability
  - Notes: Keep defaults auto-detect; only require manual selection when detection is ambiguous.

- [ ] **Task 5: Tests + docs refresh**
  - Scope: `src/port.rs`, `README.md`, `AGENTS.md`, `.agents/research/weact-display-fs-v1-3-5-research.md`
  - Depends on: Task 2
  - Acceptance:
    - Unit tests cover model detection paths and protocol command enums
    - Documentation clearly describes both devices and the plan forward
  - Notes: Update plan status to COMPLETED once verification passes.
