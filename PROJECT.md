# PROJECT.md — keystone

**What:** Contract linter — validates agent behavior against .agent-contract.toml guardrails.

**Status:** MVP complete, published to github.com/MPfeifer33/keystone

## Architecture
- `src/cli.rs` — Clap 4 CLI: `init`, `lint`, `show`
- `src/contract.rs` — TOML-based .agent-contract.toml with ProjectMeta, ZoneRule (safe/protected), ValidationRule, OwnershipRule. Default contract includes standard safe zones, protected zones, and validation commands.
- `src/lint.rs` — Checks required files exist, flags modified protected files via git status, validates tool availability. matches_pattern() with glob support. 3 unit tests.
- `src/report.rs` — Lint results and contract display (text + JSON)
- `src/main.rs` — Standard error handling

## Usage
```bash
# Initialize a default contract
keystone init

# Lint the repo against its contract
keystone lint

# Show the current contract
keystone show

# JSON output
keystone lint --format json
```

## Design Decisions
- TOML contract format (human-readable, easy to edit)
- Default contract is sensible out of the box (src/tests/docs safe, Cargo.toml/.env protected)
- Lint is non-destructive — reports findings, doesn't block
- Glob pattern matching for file zone rules
- Severity levels: Error (fails), Warning (passes with notes), Info

## Last Updated
June 22, 2026 — Initial MVP
