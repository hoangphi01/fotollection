# ADR-0015: Interface localization with bundled i18next resources

- **Status:** Accepted
- **Date:** 2026-10-23
- **Related:** FR-020, NFR-010, ADR-0007, ADR-0013

## Context

The owner and many target users read Vietnamese, so the interface must be available in English (the default) and Vietnamese, with German later ([FR-020](../PRD.md#fr-020--localization)). The app is offline ([SECURITY.md §5](../SECURITY.md#5-privacy)), its UI is React 19 ([ADR-0007](0007-ui-react-shadcn-lucide.md)), errors come from Rust with a stable `code` and an English `message`, and about 110 strings were hard-coded in 18 components.

## Options considered

1. **Hand-rolled typed dictionary.** No dependency and the smallest bundle. But plural rules, interpolation and fallback would all be ours to write and test, for every new language.
2. **`i18next` + `react-i18next` (MIT).** The de facto standard: CLDR plural rules through `Intl.PluralRules`, interpolation, fallback, a `useTranslation` hook, and typed keys through `CustomTypeOptions`. Cost: about 64 kB minified (21 kB gzipped) of JavaScript.
3. **Lingui or FormatJS (MIT).** ICU messages and compile-time extraction. Better for very large apps, but they need a build step or a macro plugin that adds tooling to the Vite build.

## Decision

Use **`i18next` + `react-i18next`**, with every locale **bundled as JSON** in `ui/src/locales/<lang>.json` and imported at build time. No HTTP backend or language detector is installed, so nothing is fetched at runtime.

- Keys are typed from `en.json`, so a wrong key fails `tsc`. `ui/scripts/check-i18n.mjs`, run by `npm run lint`, fails when a locale lacks a key, has an empty value, or when a component still contains hard-coded visible text.
- The language is chosen in the settings panel and stored per machine in `settings.json` (`ui.language`, schema v3, [DATABASE.md §6](../DATABASE.md#6-app-settings)). English is the default. It is never stored in a project file.
- Rust errors are translated in the UI by `code` (`errors.<code>`), and the English `message` is the fallback. Rust keeps a single language.
- Numbers use `Intl.NumberFormat` with the UI language (`18.6` / `18,6`). Dates of photos stay `YYYY-MM-DD HH:MM`, which is unambiguous in both languages.
- `interpolation.escapeValue` is off, because React escapes every interpolated value and nothing is rendered as HTML ([SECURITY.md](../SECURITY.md)). Translations never contain markup.
- Text printed on photos (the info frame and the date stamp) does not follow the UI language: it is print content.

## Consequences

- Adding a language means adding `<lang>.json` and a row in `LANGUAGES` (`ui/src/i18n/index.ts`). The check script enforces completeness.
- Every new UI string goes through `t()` in the same change ([STYLE_RULE.md §3](../STYLE_RULE.md#3-typescript-ui)), and new Rust error codes get an `errors.*` entry in every locale.
- The bundle grows by about 64 kB ([OPTIMIZATION.md §5](../OPTIMIZATION.md#5-benchmark-log)). It is read from disk, but cold start ([NFR-010](../PRD.md#nfr-010--cold-start)) must stay measured.
- The native menu (FR-023) is built in Rust and needs its own small string table in the shell.
