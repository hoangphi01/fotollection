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
| `ko` | 한국어 | ✓ | ✓ (macOS menu in Apple's words) | ✓ | short (`site/_includes/manual-short-ko.md`), links the English one | `en` | `en` | AI, from the English | — | v0.14.2 |
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

The order in the picker matches the website's: English first, then the others by their own name (Deutsch, Magyar, Tiếng Việt, 한국어, 日本語).

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
10. **Korean** (FR-128, 2026-11-15), as Apple's Korean macOS and Microsoft's Korean style write it:
    - **Actions** are Sino-Korean nouns (저장, 취소, 삭제, 편집, 적용) or native verbs as -기 nouns (열기, 닫기, 내보내기, 가져오기), never -하기; the macOS menu alone keeps Apple's 복사하기.
    - **Hints and messages** end in -합니다 for a statement and -세요 for a request, with a full stop, never 해요체 or -십시오 (Microsoft's Korean style); work under way is "-하는 중…".
    - **Words are spaced** (띄어쓰기), unlike Japanese: a true compound is written whole, the rest as the 국립국어원 dictionary spaces it (미리 보기). The interface sets `word-break: keep-all`, so a line never breaks inside a word.
    - **The develop sliders take Adobe's Korean names** (노출, 대비, 밝은 영역, 어두운 영역, 흰색 계열, 검정 계열, 부분 대비, 디헤이즈, 생동감, 채도, 색온도, 색조, 비네팅, 그레인). The native macOS menu follows Apple's own words (윈도우, 실행 복귀, 별도 저장…, 가리기).
    - Loanwords as the website writes them (폴더, 필터, 프린트, 프리셋); "…" ends a label that opens a dialog. A Hangul syllable is nearly 1 em wide: `npm run text-widths` measures it in the bundled Korean face.

## 4. Glossary

These are the core terms, as the interface shows them. The Hungarian, German, Japanese, Korean and French ones follow the website's `site/_data/i18n/hu.yml`, `de.yml`, `ja.yml`, `ko.yml` and `fr.yml` wherever they use them; German says *du*, French *vous*, as the websites.

| English | Tiếng Việt | Magyar | Deutsch | 日本語 | 한국어 | Français |
|---------|------------|--------|---------|--------|--------|----------|
| photo | ảnh | fotó | Foto | 写真 | 사진 | photo |
| print | bản in | nyomat | Abzug | プリント | 프린트 | tirage |
| print size | khổ in | nyomatméret | Druckformat | プリントサイズ | 프린트 크기 | format d'impression |
| crop | cắt ảnh | vágás | Zuschnitt | トリミング | 자르기 | recadrage |
| straighten | làm thẳng | kiegyenesítés | Geraderichten | 傾き補正 | 수평 맞추기 | redressement |
| date stamp | đóng dấu ngày | dátumbélyeg | Datumsstempel | 日付スタンプ | 날짜 표시 | date incrustée |
| info frame | khung thông tin | adatkeret | Info-Rahmen | 情報フレーム | 정보 프레임 | cadre d'infos |
| look | look | look | Look | ルック | 룩 | look |
| preset | preset | preset | Preset | プリセット | 프리셋 | preset |
| Simple (mode) | Đơn giản | Egyszerű | Einfach | シンプル | 심플 | Simple |
| Collage (mode) | Ghép ảnh | Kollázs | Collage | コラージュ | 콜라주 | Collage |
| Pro (mode) | Pro | Pro | Pro | Pro | Pro | Pro |
| postcard | bưu thiếp | képeslap | Postkarte | ポストカード | 엽서 | carte postale |
| workspace | không gian làm việc | munkaterület | Arbeitsbereich | 作業スペース | 작업 공간 | espace de travail |
| print basket | giỏ in | nyomtatási kosár | Druckkorb | プリントバスケット | 프린트 바구니 | panier d'impression |
| set aside | để riêng | félretett | Zurückgelegt | 保留 | 보류 | mis de côté |
| best shot | ảnh đẹp nhất | legjobb kép | beste Aufnahme | ベストショット | 베스트 샷 | meilleure photo |
| ticked | đã tick | kipipált | markiert | チェック済み | 체크한 | cochée |
| reject | loại | elvetés | Verwerfen (verworfen) | 除外 | 제외 | rejet (rejetée) |
| star rating | sao | csillag | Sterne | レーティング | 별점 | étoiles |
| export | xuất ảnh | exportálás | Exportieren | 書き出し | 내보내기 | exporter |
| develop (RAW) | xử lý RAW | kidolgozás | Entwickeln | 現像 | 현상 | développement |
| undo | hoàn tác | visszavonás | Rückgängig | 取り消し | 실행 취소 | annuler |
| original colour | màu gốc | eredeti színek | Originalfarben | 元の色 | 원본 색 | couleurs d'origine |
| settings | cài đặt | beállítások | Einstellungen | 設定 | 설정 | réglages |
| project | dự án | projekt | Projekt | プロジェクト | 프로젝트 | projet |
| library | thư viện | könyvtár | Bibliothek | ライブラリ | 라이브러리 | bibliothèque |
| folder | thư mục | mappa | Ordner | フォルダー | 폴더 | dossier |

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

   A language whose letters Be Vietnam Pro lacks also bundles a face cut to them: Japanese has Noto Sans JP and Korean Noto Sans KR (OFL 1.1), each cut by `scripts/cjk-font.sh <code>` to every character of its locale and manual, plus every kana or the Hangul compatibility jamo and the CJK punctuation, one variable woff2 each (`ui/src/assets/fonts/`). They sit after Be Vietnam Pro in the interface's font stack, Japanese first, each with its own `unicode-range`, so a face loads only when its script is shown; `check-i18n` fails, naming the character, when a translation uses one its cut lacks, and `scripts/text-widths.mjs` measures with them.
8. The docs:
   - CHANGELOG;
   - the PRD requirement, plus its ROADMAP and STATUS rows;
   - the README feature lists (both);
   - the manuals' language paragraph (both);
   - the website's mention of the app's languages (`site/_data/i18n/*.yml`).

Before a language exists, the pseudo-locale `en-XA` shows what a long one will do: English made 40 % longer and marked (`ui/src/i18n/pseudo.ts`), only in a `VITE_PSEUDO=1` build, run by `scripts/ui-check.sh` (`MEASURE_LANG=en-XA`). On 2026-11-12 it showed that a German-length translation would cut *Retro orange*, *Focal length*, *One colour*, *Fit whole photo* and *Image*.

## 6. Known limits

- **Going back to an older version:** Deutsch and Magyar come with settings v19 (v0.14.1). An older app reads no v19 file at all ([DATABASE.md §6](DATABASE.md#6-app-settings)), whatever the language, so `"de"` and `"hu"` add no new case. 日本語 and 한국어 come with settings v20 (v0.14.2): v0.14.1 meets a v20 file as a newer one, uses its defaults and leaves the file as it is.
- **Japanese or Korean the cut lacks:** a folder's or a place's name in kanji or Hangul the bundled face does not hold is drawn in the system's Japanese face, a little different in width; every such field already fits its text (`FitText`, truncation).
- **Print text in Japanese:** the date stamp, info frame and postcard fonts hold Latin letters only, so their placeholders stay in Latin letters; Japanese text on a print comes with FR-130's はがき; Korean, later.

## 7. What differs by market

Some things change with the language or the country, not only the words, as a phone sold in one country may take a physical SIM where others take an eSIM. This table is where each such difference is listed; the requirement behind it holds the detail. A new difference gets its row here in the same commit.

| | English | Tiếng Việt | Čeština, Deutsch, Magyar, Polski, Français | 한국어 | 日本語 |
|---|---|---|---|---|---|
| **The app's interface** | ✓ | ✓ | Deutsch and Magyar (cs, pl and fr on the website only) | ✓ | ✓ |
| **Face** | Be Vietnam Pro | Be Vietnam Pro | Be Vietnam Pro | + bundled Noto Sans KR | + bundled Noto Sans JP |
| **Print face for the card** | Xanh Mono, Patrick Hand | as English | as English | as English (Hangul on a print later) | Klee One, a handwritten face, bundled for the はがき (FR-130) |
| **Line breaking** | normal | normal | normal | between words only (`keep-all`) | kinsoku (`line-break: strict`) |
| **Postcard (FR-073, FR-130, FR-132)** | European card; a switch, off by default, turns it into a はがき for a card sent to Japan | as English | European card, no はがき switch | the 10 × 15 card with Korea Post's address side (FR-132, planned), no はがき switch | the はがき alone, upright or landscape, no European card |
| **AddAssist, assisted address entry (FR-133, planned)** | — | — | Hungary (KSH), Czech Republic (ČÚZK RÚIAN), Germany (GeoNames, to check): the code fills the town | the code or a search fills 시·도 to road | the code fills prefecture, city and town, searched in romaji too |
| **Postal code on the card** | free text | free text | free text | five boxes below the address (FR-132) | 〒 and seven boxes in the app; printed `248-0024` when typed, seven vermilion boxes when left to the hand (FR-130) |
| **Greetings for the occasion (FR-131, planned)** | Christmas and others, later | Tết and others, later | later | 설날, 어버이날, 스승의 날, the year's end | 年賀, 寒中, 暑中, 残暑, 喪中 |
| **Print sizes in the app** | Europe's first, Japan's and Korea's folded | as English | as English | Korea's first (FR-132, planned) | Japan's convenience stores first (FR-129, planned) |
| **Kiosk sizes the website names** | dm, Rossmann | dm, Rossmann | dm, Rossmann; Français: CEWE at E.Leclerc, Intermarché, Fnac, Kodak at Carrefour | Printing Box (CU, GS25): 4 × 6 and 6 × 8 in | 7-Eleven, Lawson, FamilyMart: L, 2L, two squares (FR-129) |
| **Website: support** | Ko-fi (card) | VietQR | Ko-fi (card) | Ko-fi (card) | Ko-fi (card), its English page explained |
| **Website: community** | AlternativeTo, GitHub | Vietnam Linux Family, CaramOS | AlternativeTo, GitHub | AlternativeTo, GitHub | AlternativeTo, GitHub |
| **Website: screenshots and photos** | the English app, Prague | the Vietnamese app, Prague | the English app, Prague | the Korean app, photos of Seoul by @9adox and @imyourctus (planned) | the Japanese app, the owner's Kamakura (planned) |

The website's switches live in `site/_data/languages.yml` (`kiosk`, `support`, `community`, `aliases`), passed to the pages as `pg.*`; the app's in `LANGUAGES` (`ui/src/i18n/index.ts`) and, for the print, in the requirement named in the row.
