# AddAssist's bundled postal data

The data behind AddAssist, the assisted address entry ([FR-133](../../../../docs/PRD.md#fr-133--assisted-address-entry-where-the-posts-data-is-standardised), [ADR-0051](../../../../docs/adr/0051-addassist-bundled-postal-data.md)). Each table is compiled into the app, checked against the BLAKE3 sum below when it is first used, and **never downloaded at runtime**. `scripts/addassist/fetch-and-pack.sh` rebuilds it. **AddAssist 1.0** holds the tables below: Japan and Korea.

| File | Contents | Source | Terms |
|------|----------|--------|-------|
| `kr.bin` | 173,882 roads, 34,393 postal codes: each road with its 시·도, 시·군·구 and 읍·면, and the runs of building numbers sharing a 5-digit code, so a road and a number give the code and a code its roads | The Ministry of the Interior's 도로명주소 한글 전체분 ([business.juso.go.kr](https://business.juso.go.kr), 주소 DB › 속성; also listed on [data.go.kr](https://www.data.go.kr/data/15050417/fileData.do)), November 2025 (6,413,606 addresses), downloaded by hand | Listed as 「이용허락범위 제한 없음」 (no condition on use) |
| `jp.bin` | 124,527 rows, 120,721 postal codes, 1,892 municipalities: each 7-digit code with its prefecture, municipality and town, in kanji and katakana | [Japan Post](https://www.post.japanpost.jp/service/search/zipcode/download/utf-zip.html) 郵便番号データ, UTF-8 one-record-per-line edition (`utf_ken_all.zip`), data of 2026-10-23 | 「郵便番号データに限っては日本郵便株式会社は著作権を主張しません。自由に配布していただいて結構です。」 ([readme](https://www.post.japanpost.jp/service/search/zipcode/download/utf-readme.html)): no duty; credited by choice |

The placeholder rows are kept with an empty town, matched exactly, never by an ending: 以下に掲載がない場合, a municipality's ○○一円 (a town really named 一円 stays), and ○○の次に番地がくる場合. A note in parentheses (大通西（１〜１９丁目）) is kept apart from the town and never printed by itself. The Latin names are made in the app from the katakana with one Hepburn rule set (`addassist::romaji`), not taken from Japan Post's romaji edition.

## Checksum

| File | Size | BLAKE3 (checked at first use) |
|------|-----:|-------------------------------|
| `kr.bin` | 6,491,049 bytes | `8930992ebc99c7c63df17a518d0c0927a55bc870c3199de6e1942501d3dff7a7` |
| `jp.bin` | 5,469,230 bytes | `7e42b98b6ac3fce948c525f373f5af50a74e0bbf9c9f8dc09a32c74aa187b58a` |

Download used for this table (SHA-256): `utf_ken_all.zip` `db92fcf624bb7280e6f51900aebe03adeafbd168796e59412dbc22a80abb85b9`.

## Format of `jp.bin` (version 1, little-endian)

1. Header: `FADJ`, `u16` version, the data's date as 10 ASCII bytes.
2. Strings: `u32` count, then each `u16` length + UTF-8; string 0 is empty, every name stored once.
3. Municipalities: `u16` count, then each four `u32` string ids: prefecture, municipality, and their katakana.
4. Rows, sorted by code: `u32` count, then each `u32` code, `u16` municipality, `u32` town, `u32` note and `u32` town katakana (string ids).

The reader rejects any table whose checksum, magic, version, counts (at most 400,000 strings and 250,000 rows), ids or order do not match.

## Format of `kr.bin` (version 1, little-endian)

Korea's files are CP949; `scripts/addassist/fetch-and-pack.sh kr` turns them to UTF-8 with `iconv` on the way into the packer. Columns read (from 0): 2 시도, 3 시군구, 4 읍면동 (kept only when it is an 읍 or 면), 10 도로명, 11 지하, 12 건물본번, 16 우편번호. Latin names are made in the app from the Hangul (Revised Romanization, `addassist::romaji::revised`), so the English edition is not bundled.

1. Header: `FADK`, `u16` version, the data's date as 10 ASCII bytes.
2. Strings, as `jp.bin`'s.
3. Areas: `u16` count, then each three `u32` string ids: 시도, 시군구, 읍·면 (empty for a 동).
4. Codes: `u16` count, then each `u32` (at most 99999).
5. Roads, by area then name: `u32` count, then each `u16` area, `u32` name, `u8` 1 if underground, `u16` number of runs.
6. Runs, road after road in number order: `u16` first building number, `u16` code index.

The reader rejects any table whose checksum, magic, version, counts (at most 400,000 strings and roads), ids or lengths do not match.
