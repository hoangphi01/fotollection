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

// Korean: the Revised Romanization of Korean (2000), with the sound changes road signs show
// (종로 → jongno, 신라 → silla, 독립 → dongnip); enough for a search and a Latin address line.

const INITIALS: [&str; 19] = [
    "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s", "ss", "", "j", "jj", "ch", "k", "t", "p",
    "h",
];
const MEDIALS: [&str; 21] = [
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa", "wae", "oe", "yo", "u", "wo", "we",
    "wi", "yu", "eu", "ui", "i",
];
/// A final consonant at the end of a word or before a consonant.
const FINALS: [&str; 28] = [
    "", "k", "k", "k", "n", "n", "n", "t", "l", "k", "m", "l", "l", "l", "p", "l", "m", "p", "p",
    "t", "t", "ng", "t", "t", "k", "t", "p", "t",
];
/// A final consonant carried over to a following silent ㅇ (학여울's ㄱ reads g).
const LINKED: [&str; 28] = [
    "", "g", "kk", "ks", "n", "nj", "n", "d", "r", "lg", "lm", "lb", "ls", "lt", "lp", "r", "m",
    "b", "ps", "s", "ss", "ng", "j", "ch", "k", "t", "p", "",
];

/// (initial, medial, final) of a precomposed Hangul syllable.
fn jamo(c: char) -> Option<(usize, usize, usize)> {
    let i = (c as u32).checked_sub(0xAC00).filter(|i| *i < 11_172)? as usize;
    Some((i / 588, (i % 588) / 28, i % 28))
}

/// Lower-case Revised Romanization of Korean text; other characters kept, digits as they are.
pub fn revised(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    for (k, &c) in chars.iter().enumerate() {
        let Some((ini, med, fin)) = jamo(c) else {
            out.extend(c.to_lowercase());
            continue;
        };
        // The initial as the previous final leaves it: ㄹ after ㄴ/ㄹ reads l, after other finals n.
        let prev_fin = k
            .checked_sub(1)
            .and_then(|p| jamo(chars[p]))
            .map_or(0, |(_, _, f)| f);
        let initial = match (INITIALS[ini], FINALS[prev_fin]) {
            ("r", "n" | "l") => "l",
            ("r", "ng" | "m" | "k" | "p" | "t") => "n",
            (i, _) => i,
        };
        out.push_str(initial);
        out.push_str(MEDIALS[med]);
        if fin != 0 {
            let next = chars.get(k + 1).and_then(|n| jamo(*n));
            let tail = match next {
                // Before a silent ㅇ the final moves over.
                Some((11, _, _)) => LINKED[fin],
                // ㄴ before ㄹ reads l (신라 silla).
                Some((5, _, _)) if FINALS[fin] == "n" => "l",
                // Nasalization before ㄴ, ㅁ or ㄹ: ㄱ → ng, ㄷ → n, ㅂ → m.
                Some((2 | 6 | 5, _, _)) => match FINALS[fin] {
                    "k" => "ng",
                    "t" => "n",
                    "p" => "m",
                    f => f,
                },
                _ => FINALS[fin],
            };
            out.push_str(tail);
        }
    }
    out
}

/// A Korean road or place in Latin letters as signs write it: the road's ending after a hyphen
/// (세종대로 → `Sejong-daero`, 학동로30길 → `Hakdong-ro 30-gil`), other names capitalized whole.
/// A name ending in 대로 is read as a boulevard (대로); the few whose 대 belongs to the name
/// (청와대로, Cheongwadae-ro on the signs) come out as `Cheongwa-daero`.
pub fn korean_place(name: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut rest = name;
    // A road with a number: 학동로30길 → 학동로, 30, 길.
    let split_at = rest.find(|c: char| c.is_ascii_digit());
    if let Some(at) = split_at {
        let (road, tail) = rest.split_at(at);
        words.push(korean_place(road));
        let digits_end = tail
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(tail.len());
        let (num, suffix) = tail.split_at(digits_end);
        let suffix = revised(suffix);
        words.push(if suffix.is_empty() {
            num.to_owned()
        } else {
            format!("{num}-{suffix}")
        });
        return words.join(" ");
    }
    // A metropolitan city is written by its name alone (서울특별시 → Seoul), a special
    // self-governing province as a province (제주특별자치도 → Jeju-do).
    for ending in ["특별자치시", "특별시", "광역시"] {
        if let Some(stem) = rest.strip_suffix(ending).filter(|s| !s.is_empty()) {
            return capitalized(&revised(stem));
        }
    }
    if let Some(stem) = rest.strip_suffix("특별자치도").filter(|s| !s.is_empty()) {
        return format!("{}-do", capitalized(&revised(stem)));
    }
    for ending in ["대로", "로", "길", "구", "군", "시", "도", "읍", "면"] {
        if let Some(stem) = rest.strip_suffix(ending).filter(|s| !s.is_empty()) {
            rest = stem;
            return format!("{}-{}", capitalized(&revised(rest)), revised(ending));
        }
    }
    capitalized(&revised(name))
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

    #[test]
    fn fr_133_hangul_reads_as_revised_romanization() {
        assert_eq!(revised("세종대로"), "sejongdaero");
        assert_eq!(revised("청와대로"), "cheongwadaero");
        assert_eq!(revised("종로"), "jongno");
        assert_eq!(revised("신라"), "silla");
        assert_eq!(revised("독립"), "dongnip");
        assert_eq!(revised("서울"), "seoul");
        assert_eq!(revised("학여울"), "hagyeoul");
        assert_eq!(korean_place("세종대로"), "Sejong-daero");
        assert_eq!(korean_place("테헤란로"), "Teheran-ro");
        // 대로 is read as the boulevard: 청와대로 (the Blue House road) reads Cheongwa-daero,
        // not the signs' Cheongwadae-ro; the search folds both alike.
        assert_eq!(korean_place("청와대로"), "Cheongwa-daero");
        assert_eq!(korean_place("학동로30길"), "Hakdong-ro 30-gil");
        assert_eq!(korean_place("종로구"), "Jongno-gu");
        assert_eq!(korean_place("서울특별시"), "Seoul");
        assert_eq!(korean_place("경기도"), "Gyeonggi-do");
        assert_eq!(korean_place("제주특별자치도"), "Jeju-do");
    }
}
