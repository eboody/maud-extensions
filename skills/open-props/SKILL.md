---
name: open-props
description: Source-grounded guidance for Open Props CSS variables, imports, and token catalogs. Use when implementing Open Props in CSS/JS, choosing token names or values, wiring CDN or NPM imports, or verifying available props against upstream documentation snapshots in this skill.
---

# Objective

Use authoritative Open Props references in this skill to make correct integration and token-selection changes without inventing props.

## Primary References

1. `/home/eran/code/maud-extensions/skills/open-props/references/source-snapshot.md`
2. `/home/eran/code/maud-extensions/skills/open-props/references/docsite-section-index.md`
3. `/home/eran/code/maud-extensions/skills/open-props/references/upstream/README.md`
4. `/home/eran/code/maud-extensions/skills/open-props/references/upstream/docsite/index.html`
5. `/home/eran/code/maud-extensions/skills/open-props/references/upstream/open-props.resolver.json`
6. `/home/eran/code/maud-extensions/skills/open-props/references/upstream/package.json`
7. `/home/eran/code/maud-extensions/skills/open-props/references/upstream/CHANGELOG.md`

Load only the files needed for the current task.

## Workflow

1. Identify integration target: CDN stylesheet usage, NPM CSS import, JS/TS module usage, or design-token export.
2. Verify import path from `package.json` `exports` before writing code.
3. Resolve token names and exact values from `open-props.resolver.json` when precision matters.
4. Use `docsite/index.html` plus `docsite-section-index.md` for section-level examples and guidance.
5. Check `CHANGELOG.md` and `package.json` version for version-sensitive behavior.
6. Propose minimal imports: module-specific file first, full bundle only when necessary.

## Guardrails

- Never invent token names; confirm in docs or resolver.
- Keep naming consistent with upstream token conventions (`--color-*`, `--size-*`, `--font-size-*`, and similar families).
- Prefer stable export paths from `package.json` over guessed filesystem paths.
- Call out when guidance is version-bound and include the exact version.
- If a requested token or category is missing from the snapshot, say so and suggest closest documented alternatives.

## Fast Lookups

- `rg --line-number --fixed-strings -- '--font-size-' /home/eran/code/maud-extensions/skills/open-props/references/upstream/open-props.resolver.json`
- `rg --line-number 'section id=\"(colors|gradients|media-queries|durations)\"' /home/eran/code/maud-extensions/skills/open-props/references/upstream/docsite/index.html`
- `rg --line-number '\"\\./(colors|sizes|media|resolver)\"' /home/eran/code/maud-extensions/skills/open-props/references/upstream/package.json`
