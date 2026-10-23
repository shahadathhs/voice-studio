# Localization

The interface is translated; the engine's output is not. Today: English (the reference) and Spanish.

## How it works

- Messages live in `desktop/src/locales/<code>/`, split by screen (`common`, `transcribe`, `library`, `models`,
  `speak`). Keys are flat and dotted (`batch.start`), values are plain strings with `{name}` placeholders.
- Components call `t("batch.remove", { name })` from `lib/i18n.svelte.ts`. Never write user-facing text in markup.
- **Plurals:** give a `count` and define `key.one` and `key.other` (add `few`, `many`… for languages that need
  them). `t("batch.ready", { count })` picks the form with `Intl.PluralRules`.
- **Names, dates and numbers** come from `Intl` (`Intl.DisplayNames` for language names), so they follow the
  language with nothing to translate.
- **Choice:** Settings → Language, or the system language by default (`es-MX` matches `es`). The choice is kept
  in `localStorage`; `<html lang>` follows it.

## Add a language

1. Copy `locales/es/` to `locales/<code>/` and translate the values. Do not change keys.
2. Export it from `locales/<code>/index.ts` typed `Record<CatalogKey, string>`.
3. Add `{ code, name, messages }` to `LOCALES` in `lib/i18n.svelte.ts`. `name` is the language's own name.

`make typecheck` fails if a language is missing a key or has an extra one, so a catalog cannot silently fall
behind English. A missing message at run time falls back to English, then to the key.

## What is not translated

- Text that comes from Rust: error messages, voice descriptions, and model names. Model _descriptions_ can be
  translated with a `modelInfo.<model id>` message; without one the catalog's English is shown.
- The command-line tool and its `--help`.
- Transcripts, which stay in the language that was spoken.

The Spanish catalog was written without a native review; corrections are welcome.
