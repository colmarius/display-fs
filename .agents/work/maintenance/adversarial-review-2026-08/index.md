# Adversarial review: bugs, dedup, docs

Status: completed
Category: maintenance
Updated: 2026-08-16

## Why

User requested an adversarial review of the code, GitHub workflows, features, site documentation, and AGENTS.md, with findings stored durably, improvements planned and implemented, and irrefutable proof of correctness. Goals: end-to-end testing where possible, minimal changes, remove complexity, keep code readable.

## Summary

Review and implementation complete. Fixed a truncation infinite loop (with regression test), broken `just run`/`just detect`/doc commands, and Spotify state tracking; removed all legacy duplicate `_oriented`/default-dimension APIs (−604 lines net); corrected docs/site; CI now caches and tests the `japanese` feature (which exposed and fixed a latent font-dependent test). All checks green: fmt, clippy (all targets), 27 + 28 tests. Findings in `research.md`; plan and observed evidence in `plan.md`.

## Artifacts

- Research: [research.md](research.md)
- PRD: none
- Plan: [plan.md](plan.md)
- Progress: tracked via plan checkboxes
- Decisions: none
- Handoffs: none

## Next Action

- None.

## Open Questions

- None.
