---
name: maud-extensions
description: Source-grounded guidance for the maud-extensions proc-macro crate (`css!`, `js!`, `inline_css!`, `inline_js!`, `font_face!`, `font_faces!`). Use when adding or fixing macro behavior, parser/validator logic, compile-time errors, generated Maud markup, or README examples in this repository.
---

# Objective

Ship correct updates to `maud-extensions` without breaking call-site ergonomics or macro output.

## Primary Sources

1. `/home/eran/code/maud-extensions/src/lib.rs`
2. `/home/eran/code/maud-extensions/README.md`
3. `/home/eran/code/maud-extensions/Cargo.toml`

Prefer source-first answers and avoid claims that are not verified in these files.

## Workflow

1. Identify which macro is being changed (`css!`, `js!`, `inline_css!`, `inline_js!`, `font_face!`, `font_faces!`).
2. Preserve parsing shape:
   - `LitStr` input path when a string literal is provided.
   - token-stream fallback path when raw tokens are provided.
3. Preserve validation behavior:
   - CSS parsing through `validate_css` (`cssparser`).
   - JS parsing through `validate_js` (`swc_ecma_parser`).
4. Keep compile-time failures actionable using `syn::Error::new(...).to_compile_error()`.
5. Run `cargo check` (and `cargo test` when behavior changed).
6. Update README examples when macro behavior or API surface changed.

## Guardrails

- Keep macro outputs compatible with `maud::html!` and `maud::Markup` usage.
- Avoid introducing runtime behavior that belongs in consuming apps.
- Do not silently change token-to-string spacing rules without explicit intent.
- Keep `font_face!`/`font_faces!` extension detection and data URL behavior consistent unless intentionally refactoring.
- Keep public macro names stable unless a breaking change is explicitly requested.

## Common Changes

- Add a new macro input form:
  - Extend `syn::Parse` input enum.
  - Reuse existing validation pattern.
  - Emit Maud-compatible output via `quote!`.
- Improve diagnostics:
  - Return precise parser/validator messages.
  - Keep errors tied to call-site spans.
- Modify generated markup:
  - Verify output type remains `maud::Markup`-compatible.
  - Check README snippets still match generated behavior.
