# Localization — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-12
>
> This page is the single source of truth for the app's languages. It covers:
>
> - which languages the app speaks, and how much of the app each one covers;
> - what falls back to English;
> - the rules for every translation, and the shared glossary;
> - the checklist for adding a language.
>
> The mechanism is in [ADR-0015](adr/0015-ui-localization-i18next.md) and the requirements are [FR-020](PRD.md#fr-020--localization), [FR-123](PRD.md#fr-123--hungarian-interface) and [FR-127](PRD.md#fr-127--german-interface). The website's languages are a separate system (`site/_data/languages.yml`, [ADR-0049](adr/0049-website-on-eleventy.md)). It shares the glossary below.

## 1. Languages

| Code | Name | Interface | Menu and dialogs | Errors | Manual | Welcome pictures | Update reason | Translated by | Native review | Since |
|------|------|-----------|------------------|--------|--------|------------------|---------------|---------------|---------------|-------|
| `en` | English | ✓ source | ✓ | ✓ | full (`docs/USER_GUIDE.md`) | `en` | `en` | the author | — | v0.1.0 |
| `vi` | Tiếng Việt | ✓ | ✓ | ✓ | full (`docs/USER_GUIDE.vi.md`) | `vi` | `vi` | the owner | ✓ the owner | v0.1.0 |
| `de` | Deutsch | ✓ | ✓ | ✓ | short (`site/_includes/manual-short-de.md`), links the English one | `en` | `en` | AI, from the English | — | v0.14.1 |
| `hu` | Magyar | ✓ | ✓ | ✓ | short (`site/_includes/manual-short-hu.md`), links the English one | `en` | `en` | AI, from the English | — | v0.14.1 |
| `ja` | 日本語 | ✓ | ✓ (macOS menu in Apple's words) | ✓ | short (`site/_includes/manual-short-ja.md`), links the English one | `en` | `en` | AI, from the English | — | v0.14.2 |

Here is what each column covers:

- **Interface:** `ui/src/locales/<code>.json`. English is the source; every other locale has the same keys.
- **Menu and dialogs:** the strings the Rust shell shows itself: the native menu, the window title and the file dialogs (`src-tauri/src/i18n.rs`).
- **Errors:** Rust gives a code. The interface shows `errors.<code>` from the locale, with Rust's English message as the fallback. Rust keeps one language.
- **Manual:** the manual inside the app ([FR-110](PRD.md#fr-110--the-manual-inside-the-app-offline)). The full manual exists in English and Vietnamese. Any other language shows its short guide, the same one the website shows.
- **Welcome pictures:** `ui/public/welcome/<page>-<assets>-<theme>.webp`, the screenshots of the welcome screen.
- **Update reason:** the reason for a safety update, written with the release in English and Vietnamese (`.github/update.json`, [FR-103](PRD.md#fr-103--check-for-updates-at-start-never-download-unasked)).

## 2. One description per language

Each language is described once, in `LANGUAGES` (`ui/src/i18n/index.ts`): its code, its own name, its manual (`full` or `short`) and its assets (`en` or `vi`). The manual, the welcome screen and the update popup read that entry; none of them tests a language code itself.

**Fallback:** a key, a manual, a picture or a reason a language lacks is shown in English.

The order in the picker matches the website's: English first, then the others by their own name (Deutsch, Magyar, Tiếng Việt, 日本語).

The language is stored per machine in `settings.json` (`ui.language`, [DATABASE.md §6](DATABASE.md#6-app-settings)), never in a project.

## 3. Rules for every translation

1. **The app's words, not new ones.** Use the glossary (§4). A term the interface shows is written the same way in the manual and on the website.
2. **Placeholders stay exactly as in English.** That covers `{{n}}`, `{{name}}`, ⌘, the key names and the units (cm, DPI, MB). `check-i18n` fails when a locale's placeholders differ from English.
3. **Plurals follow CLDR.** Each English `_one` / `_other` pair gets the forms the language has. Vietnamese has only `_other`. Hungarian has `_one` and `_other`, and a noun after a number stays singular, so both forms often read the same (*1 fotó*, *5 fotó*). German has `_one` and `_other` as English.
4. **Plain text only.** No markup and no HTML: React escapes every value, and nothing is rendered as HTML ([SECURITY.md](SECURITY.md)).
5. **Print content never follows the interface.** The info frame's line and the date stamp are part of the print (FR-020).
6. **Numbers follow the language,** through `Intl.NumberFormat` (`18.6`, `18,6`). A photo's date stays `YYYY-MM-DD HH:MM` in every language.
7. **Shorter first, then the three steps** ([FR-125](PRD.md#fr-125--long-words-fit-a-step-smaller-then-cut-with-the-whole-in-a-hint)). A translation is kept as short as the English where it can be: `npm run text-widths` lists the labels it makes over 30 % and 20 px wider. What still does not fit is fitted by the app: one step of the type scale smaller for that control only, then cut with an ellipsis and the whole words in the hint (*nagyállomás* → *nagyállom…*). English, the source, always fits whole (UI-TEXT-007); MEASURE lists every word a translation shrank or cut (`MEASURE=1 MEASURE_LANG=<code>`). The layout is never changed for one language ([STYLE_RULE.md §4](STYLE_RULE.md#4-uiux)).
8. **Product names stay as they are:** Fotollection, Lightroom, LUMIX Lab, LUT, RAW, JPEG, Pro.
9. **Japanese** (FR-128, 2026-11-15), as Japanese apps and Apple's and Microsoft's Japanese style write it:
   - **Actions** are Sino-Japanese nouns without する (保存, 削除, 適用, 追加, 完了, 選択); native verbs stay in their dictionary form (開く, 閉じる, 書き出す). **Things and features** borrowed from English are katakana (プリント, プレビュー, フィルター, フレーム).
   - **Loanwords keep their final long vowel**, as Microsoft's style and the Cabinet's notation since 2008: フォルダー, フィルター, プリンター, ユーザー. The native macOS menu alone follows Apple's own words (ウインドウ, しまう, 取り消す).
   - **Labels and titles end in a noun** (体言止め: 写真を書き出し, プリントサイズ); **hints and messages** are です・ます sentences.
   - **The develop sliders take Lightroom Classic's Japanese names** (露光量, ハイライト, シャドウ, 白レベル, 黒レベル, 明瞭度, かすみの除去, 自然な彩度, 色温度, 色かぶり補正), the words Japanese photographers know.
   - **No typed space** between Japanese and Latin or figures (the UI spaces them); figures and units are half-width; "…" ends a label that opens a dialog. A Japanese character is nearly 1 em wide, so a label short in characters can still be wide: `npm run text-widths` measures it in the bundled Japanese face.

## 4. Glossary

These are the core terms, as the interface shows them. The Hungarian, German and Japanese ones follow the website's `site/_data/i18n/hu.yml`, `de.yml` and `ja.yml` wherever they use them; German says *du*, as the website.

| English | Tiếng Việt | Magyar | Deutsch | 日本語 |
|---------|------------|--------|---------|--------|
| photo | ảnh | fotó | Foto | 写真 |
| print | bản in | nyomat | Abzug | プリント |
| print size | khổ in | nyomatméret | Druckformat | プリントサイズ |
| crop | cắt ảnh | vágás | Zuschnitt | トリミング |
| straighten | làm thẳng | kiegyenesítés | Geraderichten | 傾き補正 |
| date stamp | đóng dấu ngày | dátumbélyeg | Datumsstempel | 日付スタンプ |
| info frame | khung thông tin | adatkeret | Info-Rahmen | 情報フレーム |
| look | look | look | Look | ルック |
| preset | preset | preset | Preset | プリセット |
| Simple (mode) | Đơn giản | Egyszerű | Einfach | シンプル |
| Collage (mode) | Ghép ảnh | Kollázs | Collage | コラージュ |
| Pro (mode) | Pro | Pro | Pro | Pro |
| postcard | bưu thiếp | képeslap | Postkarte | ポストカード |
| workspace | không gian làm việc | munkaterület | Arbeitsbereich | 作業スペース |
| print basket | giỏ in | nyomtatási kosár | Druckkorb | プリントバスケット |
| set aside | để riêng | félretett | Zurückgelegt | 保留 |
| best shot | ảnh đẹp nhất | legjobb kép | beste Aufnahme | ベストショット |
| ticked | đã tick | kipipált | markiert | チェック済み |
| reject | loại | elvetés | Verwerfen (verworfen) | 除外 |
| star rating | sao | csillag | Sterne | レーティング |
| export | xuất ảnh | exportálás | Exportieren | 書き出し |
| develop (RAW) | xử lý RAW | kidolgozás | Entwickeln | 現像 |
| undo | hoàn tác | visszavonás | Rückgängig | 取り消し |
| original colour | màu gốc | eredeti színek | Originalfarben | 元の色 |
| settings | cài đặt | beállítások | Einstellungen | 設定 |
| project | dự án | projekt | Projekt | プロジェクト |
| library | thư viện | könyvtár | Bibliothek | ライブラリ |
| folder | thư mục | mappa | Ordner | フォルダー |

The Hungarian website still writes the mode names in English (*Simple*, *Collage*), because its screenshots show the English app. When Hungarian screenshots exist, the site takes the app's names.

## 5. Adding a language

Do all of it in one commit.

1. `ui/src/locales/<code>.json`, translated from `en.json` with the glossary (add a column to §4).
2. A row in `LANGUAGES` and `resources` (`ui/src/i18n/index.ts`).
3. `Language::<Code>` in `crates/core/src/settings.rs`, plus a test that reads it back. The settings' schema row in [DATABASE.md](DATABASE.md) names it.
4. `static <CODE>: Strings` in `src-tauri/src/i18n.rs`, plus its branch in `strings()`.
5. Its manual: the short guide of the website (`site/_includes/manual-short-<code>.md`), or a full `docs/USER_GUIDE.<code>.md` that `check-docs` then keeps in step with English.
6. A row in §1 here: who translated it and whether a native speaker has reviewed it.
7. The checks:
   - `cd ui && npm run lint`: `check-i18n` checks for the same keys, no empty values and the same placeholders;
   - `cargo test`: regenerates `ui/src/ipc/bindings.ts`;
   - `npm run text-widths`: the labels it makes much wider than English, to shorten first;
   - `MEASURE=1 MEASURE_LANG=<code> node site/screenshots/shoot.mjs …`, in both themes: no overflow, and the list of words fitted smaller or cut;
   - its row in `scripts/ui-check.sh`.

   A language whose letters Be Vietnam Pro lacks also bundles a face cut to them: Japanese has Noto Sans JP (OFL 1.1), cut by `scripts/ja-font.sh` to every character of `ja.json` and its manual, every kana and the CJK punctuation, 207 KB as one variable woff2 (`ui/src/assets/fonts/`). It sits second in the interface's font stack with a CJK-only `unicode-range`, so no other language loads it; `check-i18n` fails, naming the character, when a translation uses one the cut lacks, and `scripts/text-widths.mjs` measures with it.
8. The docs:
   - CHANGELOG;
   - the PRD requirement, plus its ROADMAP and STATUS rows;
   - the README feature lists (both);
   - the manuals' language paragraph (both);
   - the website's mention of the app's languages (`site/_data/i18n/*.yml`).

Before a language exists, the pseudo-locale `en-XA` shows what a long one will do: English made 40 % longer and marked (`ui/src/i18n/pseudo.ts`), only in a `VITE_PSEUDO=1` build, run by `scripts/ui-check.sh` (`MEASURE_LANG=en-XA`). On 2026-11-12 it showed that a German-length translation would cut *Retro orange*, *Focal length*, *One colour*, *Fit whole photo* and *Image*.

## 6. Known limits

- **Going back to an older version:** Deutsch and Magyar come with settings v19 (v0.14.1). An older app reads no v19 file at all ([DATABASE.md §6](DATABASE.md#6-app-settings)), whatever the language, so `"de"` and `"hu"` add no new case. 日本語 comes with settings v20 (v0.14.2): v0.14.1 meets a v20 file as a newer one, uses its defaults and leaves the file as it is.
- **Japanese the cut lacks:** a folder's or a place's name in kanji the bundled face does not hold is drawn in the system's Japanese face, a little different in width; every such field already fits its text (`FitText`, truncation).
- **Print text in Japanese:** the date stamp, info frame and postcard fonts hold Latin letters only, so their placeholders stay in Latin letters; Japanese text on a print comes with FR-130's はがき.
