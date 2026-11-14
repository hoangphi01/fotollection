// SPDX-License-Identifier: GPL-3.0-only
//! Japanese place names in Latin letters, made from Japan Post's katakana with one Hepburn
//! rule set (PRD FR-130: not Japan Post's romaji edition, whose names are cut short and spelt
//! inconsistently). Long vowels are written without macrons, as road signs write them
//! (トウキョウ → Tokyo, オオサカ → Osaka). The search folds what is typed the same way.

/// The Hepburn syllable of one full-size katakana, or `None` for anything else.
fn syllable(c: char) -> Option<&'static str> {
    Some(match c {
        'ア' => "a",
        'イ' | 'ヰ' => "i",
        'ウ' => "u",
        'エ' | 'ヱ' => "e",
        'オ' | 'ヲ' => "o",
        'カ' => "ka",
        'キ' => "ki",
        'ク' => "ku",
        'ケ' | 'ヶ' => "ke",
        'コ' => "ko",
        'ガ' => "ga",
        'ギ' => "gi",
        'グ' => "gu",
        'ゲ' => "ge",
        'ゴ' => "go",
        'サ' => "sa",
        'シ' => "shi",
        'ス' => "su",
        'セ' => "se",
        'ソ' => "so",
        'ザ' => "za",
        'ジ' | 'ヂ' => "ji",
        'ズ' | 'ヅ' => "zu",
        'ゼ' => "ze",
        'ゾ' => "zo",
        'タ' => "ta",
        'チ' => "chi",
        'ツ' => "tsu",
        'テ' => "te",
        'ト' => "to",
        'ダ' => "da",
        'デ' => "de",
        'ド' => "do",
        'ナ' => "na",
        'ニ' => "ni",
        'ヌ' => "nu",
        'ネ' => "ne",
        'ノ' => "no",
        'ハ' => "ha",
        'ヒ' => "hi",
        'フ' => "fu",
        'ヘ' => "he",
        'ホ' => "ho",
        'バ' => "ba",
        'ビ' => "bi",
        'ブ' => "bu",
        'ベ' => "be",
        'ボ' => "bo",
        'パ' => "pa",
        'ピ' => "pi",
        'プ' => "pu",
        'ペ' => "pe",
        'ポ' => "po",
        'マ' => "ma",
        'ミ' => "mi",
        'ム' => "mu",
        'メ' => "me",
        'モ' => "mo",
        'ヤ' => "ya",
        'ユ' => "yu",
        'ヨ' => "yo",
        'ラ' => "ra",
        'リ' => "ri",
        'ル' => "ru",
        'レ' => "re",
        'ロ' => "ro",
        'ワ' => "wa",
        'ン' => "n",
        'ヴ' => "vu",
        _ => return None,
    })
}

/// Hiragana to katakana, full-width ASCII to ASCII; anything else as it is.
pub(crate) fn normalize_char(c: char) -> char {
    match c {
        'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        '！'..='～' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '　' => ' ',
        _ => c,
    }
}

/// Lower-case Hepburn for katakana; full-width digits and letters become ASCII, other
/// marks a space.
pub fn hepburn(kana: &str) -> String {
    let mut out = String::with_capacity(kana.len());
    let mut double_next = false;
    for c in kana.chars().map(normalize_char) {
        match c {
            'ッ' => double_next = true,
            'ャ' | 'ュ' | 'ョ' => {
                let y = match c {
                    'ャ' => "a",
                    'ュ' => "u",
                    _ => "o",
                };
                // ki + ャ → kya; shi, chi, ji + ャ → sha, cha, ja.
                if out.ends_with("shi")
                    || out.ends_with("chi")
                    || out.ends_with('j')
                    || out.ends_with("ji")
                {
                    out.pop();
                    out.push_str(y);
                } else if out.ends_with('i') {
                    out.pop();
                    out.push('y');
                    out.push_str(y);
                } else {
                    out.push('y');
                    out.push_str(y);
                }
            }
            'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ' => {
                let v = match c {
                    'ァ' => 'a',
                    'ィ' => 'i',
                    'ゥ' => 'u',
                    'ェ' => 'e',
                    _ => 'o',
                };
                // fu + ァ → fa, te + ィ → ti, shi + ェ → she, vu + ァ → va.
                if matches!(out.chars().last(), Some('a' | 'i' | 'u' | 'e' | 'o')) {
                    out.pop();
                }
                out.push(v);
            }
            'ー' => {
                if let Some(v) = out.chars().last().filter(|v| "aiueo".contains(*v)) {
                    out.push(v);
                }
            }
            _ => match syllable(c) {
                Some(s) => {
                    if double_next {
                        // ッチ → tchi, ッカ → kka.
                        let first = if s.starts_with("ch") {
                            't'
                        } else {
                            s.chars().next().unwrap_or('t')
                        };
                        if first.is_ascii_alphabetic() && !"aiueo".contains(first) {
                            out.push(first);
                        }
                        double_next = false;
                    }
                    out.push_str(s);
                }
                None if c.is_ascii_alphanumeric() => out.push(c.to_ascii_lowercase()),
                None => out.push(' '),
            },
        }
    }
    shorten(&out)
}

/// Long vowels without macrons: ou, oo → o; uu → u.
fn shorten(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let last = out.chars().last();
        let long = matches!((last, c), (Some('o'), 'u' | 'o') | (Some('u'), 'u'));
        if !long {
            out.push(c);
        }
    }
    out
}

/// What the search compares: lower-case letters and digits only, long vowels shortened, m
/// before b, m or p read as n (Shimbashi and Shinbashi both match).
pub fn fold(text: &str) -> String {
    let letters: String = text
        .chars()
        .map(normalize_char)
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'ō' | 'ô' => 'o',
            'ū' | 'û' => 'u',
            'ā' | 'â' => 'a',
            'ē' | 'ê' => 'e',
            'ī' | 'î' => 'i',
            c => c,
        })
        .filter(char::is_ascii_alphanumeric)
        .collect();
    let short = shorten(&letters);
    let mut out = String::with_capacity(short.len());
    let chars: Vec<char> = short.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let before_lip = matches!(chars.get(i + 1), Some('b' | 'm' | 'p'));
        out.push(if c == 'm' && before_lip { 'n' } else { c });
    }
    out
}

/// The suffix a place's kanji name ends with, how it is read, and how it is written.
const SUFFIXES: [(char, &str, &str); 11] = [
    ('都', "ト", "to"),
    ('道', "ドウ", "do"),
    ('府', "フ", "fu"),
    ('県', "ケン", "ken"),
    ('郡', "グン", "gun"),
    ('市', "シ", "shi"),
    ('区', "ク", "ku"),
    ('町', "マチ", "machi"),
    ('町', "チョウ", "cho"),
    ('村', "ムラ", "mura"),
    ('村', "ソン", "son"),
];

fn capitalized(word: &str) -> String {
    let mut c = word.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// One place in Latin letters, its suffix after a hyphen when the kanji ends with one and the
/// kana with its reading: 鎌倉市 カマクラシ → `Kamakura-shi`, 北海道 → `Hokkaido`; otherwise the
/// whole reading.
pub fn place(kanji: &str, kana: &str) -> String {
    if kanji == "北海道" {
        return "Hokkaido".to_owned();
    }
    let last = kanji.chars().last();
    SUFFIXES
        .iter()
        .filter(|(c, _, _)| Some(*c) == last)
        .find_map(|(_, reading, latin)| {
            kana.strip_suffix(reading)
                .filter(|stem| !stem.is_empty())
                .map(|stem| format!("{}-{latin}", capitalized(&hepburn(stem))))
        })
        .unwrap_or_else(|| capitalized(&hepburn(kana)))
}

/// A municipality in Latin letters. A district's town (足柄下郡箱根町) is cut after 郡 and its
/// reading グン; a designated city's ward (札幌市中央区) after 市, its city read as `city_kana`
/// (the reading its wards share, found by the caller), so each part takes its suffix.
pub fn municipality(kanji: &str, kana: &str, city_kana: Option<&str>) -> String {
    if let (Some((gun, town)), Some(at)) = (kanji.split_once('郡'), kana.find("グン")) {
        let cut = at + "グン".len();
        return format!(
            "{} {}",
            place(&format!("{gun}郡"), &kana[..cut]),
            place(town, &kana[cut..])
        );
    }
    if let (Some(city), Some(at)) = (city_kana, kanji.find('市'))
        && let Some(ward_kana) = kana.strip_prefix(city)
        && kanji.ends_with('区')
    {
        let cut = at + '市'.len_utf8();
        return format!(
            "{} {}",
            place(&kanji[..cut], city),
            place(&kanji[cut..], ward_kana)
        );
    }
    place(kanji, kana)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_133_kana_reads_as_hepburn() {
        assert_eq!(hepburn("カマクラ"), "kamakura");
        assert_eq!(hepburn("トウキョウ"), "tokyo");
        assert_eq!(hepburn("オオサカ"), "osaka");
        assert_eq!(hepburn("シチリガハマ"), "shichirigahama");
        assert_eq!(hepburn("ホッカイドウ"), "hokkaido");
        assert_eq!(hepburn("ハッチョウボリ"), "hatchobori");
        assert_eq!(hepburn("キョウト"), "kyoto");
        assert_eq!(hepburn("シンジュク"), "shinjuku");
        assert_eq!(hepburn("ジュウジョウ"), "jujo");
        assert_eq!(hepburn("ヴァ"), "va");
        assert_eq!(hepburn("１チョウメ"), "1chome");
    }

    #[test]
    fn fr_133_places_take_their_suffix() {
        assert_eq!(place("鎌倉市", "カマクラシ"), "Kamakura-shi");
        assert_eq!(place("神奈川県", "カナガワケン"), "Kanagawa-ken");
        assert_eq!(place("東京都", "トウキョウト"), "Tokyo-to");
        assert_eq!(place("北海道", "ホッカイドウ"), "Hokkaido");
        assert_eq!(place("日立市", "ヒタチシ"), "Hitachi-shi");
        assert_eq!(place("町田市", "マチダシ"), "Machida-shi");
        assert_eq!(place("京都市", "キョウトシ"), "Kyoto-shi");
        assert_eq!(place("多賀町", "タガチョウ"), "Taga-cho");
        assert_eq!(
            municipality("札幌市中央区", "サッポロシチュウオウク", Some("サッポロシ")),
            "Sapporo-shi Chuo-ku"
        );
        assert_eq!(
            municipality("足柄下郡箱根町", "アシガラシモグンハコネマチ", None),
            "Ashigarashimo-gun Hakone-machi"
        );
    }

    #[test]
    fn fr_133_the_search_folds_alike() {
        assert_eq!(fold("Tōkyō"), fold("tokyo"));
        assert_eq!(fold("Shimbashi"), fold("shinbashi"));
        assert_eq!(fold("Oosaka"), "osaka");
        assert_eq!(fold("kama-kura"), "kamakura");
    }
}
