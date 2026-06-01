# AI / Codex Project Instructions

This project is ChronoOS, a Rust `no_std` x86_64 educational hobby operating system.

The public/product identity is **ChronoOS**. The repository, package, generated image names, and some legacy source text may still use Chronosapian or `chronosapien`; do not rename those unless explicitly asked.

## Core Rules

- Do not claim runtime success unless it was actually tested.
- Clearly separate:
  - implemented in code
  - partially implemented
  - needs runtime verification
  - roadmap/design-only
  - missing
- Keep the project beginner-friendly, terminal-first, and educational.
- Avoid unnecessary dependencies.
- Avoid broad refactors unless explicitly requested.
- Preserve the ChronoOS identity: eras, museum mode, quest/progress feeling, and educational explanations.
- Do not add new OS features during stabilization or documentation tasks unless explicitly requested.

## Required Progress Tracking

- Every time Codex completes a task, update `docs/AI_PROGRESS_LOG.md`.
- Every time Codex changes the roadmap, priority queue, or intended next work, update `docs/NEXT_STEPS.md`.
- Every time Codex modifies files, report the changed files and summarize why each file changed.

## Reporting Expectations

When reporting work, include:

- Files created.
- Files changed.
- What was intentionally avoided.
- Whether runtime behavior was verified.
- What still needs verification.
- The safest next engineering step.
