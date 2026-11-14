# AddAssist's bundled postal data

The data behind AddAssist, the assisted address entry ([FR-133](../../../../docs/PRD.md#fr-133--assisted-address-entry-where-the-posts-data-is-standardised), [ADR-0051](../../../../docs/adr/0051-addassist-bundled-postal-data.md)). Each table is compiled into the app, checked against the BLAKE3 sum below when it is first used, and **never downloaded at runtime**. `scripts/addassist/fetch-and-pack.sh` rebuilds it. **AddAssist 1.0** holds the tables below.

| File | Contents | Source | Terms |
|------|----------|--------|-------|
| `jp.bin` | 124,527 rows, 120,721 postal codes, 1,892 municipalities: each 7-digit code with its prefecture, municipality and town, in kanji and katakana | [Japan Post](https://www.post.japanpost.jp/service/search/zipcode/download/utf-zip.html) 郵便番号データ, UTF-8 one-record-per-line edition (`utf_ken_all.zip`), data of 2026-10-23 | 「郵便番号データに限っては日本郵便株式会社は著作権を主張しません。自由に配布していただいて結構です。」 ([readme](https://www.post.japanpost.jp/service/search/zipcode/download/utf-readme.html)): no duty; credited by choice |

The placeholder rows are kept with an empty town, matched exactly, never by an ending: 以下に掲載がない場合, a municipality's ○○一円 (a town really named 一円 stays), and ○○の次に番地がくる場合. A note in parentheses (大通西（１〜１９丁目）) is kept apart from the town and never printed by itself. The Latin names are made in the app from the katakana with one Hepburn rule set (`addassist::romaji`), not taken from Japan Post's romaji edition.

## Checksum

| File | Size | BLAKE3 (checked at first use) |
|------|-----:|-------------------------------|
| `jp.bin` | 5,469,230 bytes | `7e42b98b6ac3fce948c525f373f5af50a74e0bbf9c9f8dc09a32c74aa187b58a` |

Download used for this table (SHA-256): `utf_ken_all.zip` `db92fcf624bb7280e6f51900aebe03adeafbd168796e59412dbc22a80abb85b9`.

## Format of `jp.bin` (version 1, little-endian)

1. Header: `FADJ`, `u16` version, the data's date as 10 ASCII bytes.
2. Strings: `u32` count, then each `u16` length + UTF-8; string 0 is empty, every name stored once.
3. Municipalities: `u16` count, then each four `u32` string ids: prefecture, municipality, and their katakana.
4. Rows, sorted by code: `u32` count, then each `u32` code, `u16` municipality, `u32` town, `u32` note and `u32` town katakana (string ids).

The reader rejects any table whose checksum, magic, version, counts (at most 400,000 strings and 250,000 rows), ids or order do not match.
