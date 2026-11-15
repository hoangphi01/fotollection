// SPDX-License-Identifier: GPL-3.0-only
//! The postcard address book of `AddAssist` 1.1 (FR-133, owner 2026-11-15): recipients the user
//! saves under a name, for any card (a European address as typed, a Japanese or Korean one as
//! `AddAssist` holds it), and the Japanese and Korean recipients last used on this machine.
//! This machine's own: kept in its settings, never in a project; checked here, never trusted
//! from the interface (SECURITY.md §2). One bad entry read from a file is dropped alone, never
//! the whole settings (LESSONS, NFR-011).

use serde::{Deserialize, Serialize};
use specta::Type;

use super::address::Address;
use crate::settings::{PostcardPart, validate_postcard};

/// The most recipients the book keeps.
pub const MAX_SAVED: usize = 100;
/// The most characters of a saved recipient's name in the book.
pub const MAX_LABEL_CHARS: usize = 60;
/// How many Japanese or Korean recipients a machine remembers, per country.
pub const MAX_RECENT_PER_COUNTRY: usize = 8;

/// A card's recipient: the European address as typed, or a Japanese or Korean one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum Recipient {
    /// A European card's address, as typed (up to 5 lines).
    Free(String),
    /// A Japanese or Korean card's address, as `AddAssist` holds it.
    Assisted(Address),
}

impl Recipient {
    /// Whether it may be saved: within the limits of its card, and not blank.
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Free(text) => {
                !text.trim().is_empty() && validate_postcard(PostcardPart::Address, text)
            }
            Self::Assisted(address) => address.is_valid() && !address.is_blank(),
        }
    }

    /// The same recipient, whatever filled it: the `AddAssist` version is not part of who
    /// the card goes to.
    fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Free(a), Self::Free(b)) => a.trim() == b.trim(),
            (Self::Assisted(a), Self::Assisted(b)) => same_address(a, b),
            _ => false,
        }
    }
}

/// Whether `draft` is `full` while it was being typed: the same country, and every field it has
/// filled the same in `full` (`AddAssist` saves each field as it is left).
fn draft_of(draft: &Address, full: &Address) -> bool {
    let fields = |a: &Address| {
        [
            a.code.clone(),
            a.region.clone(),
            a.city.clone(),
            a.town.clone(),
            a.road.clone(),
            a.line.clone(),
            a.name.clone(),
        ]
    };
    draft.country == full.country
        && fields(draft)
            .iter()
            .zip(fields(full).iter())
            .all(|(d, f)| d.is_empty() || d == f)
}

fn same_address(a: &Address, b: &Address) -> bool {
    let mut a = a.clone();
    a.filled_by.clone_from(&b.filled_by);
    a == *b
}

/// A recipient saved under a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedAddress {
    pub id: u32,
    pub label: String,
    pub recipient: Recipient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BookError {
    #[error("the address book holds {MAX_SAVED} recipients already")]
    Full,
    #[error("a name is 1 to {MAX_LABEL_CHARS} characters, without control characters")]
    Label,
    #[error("the address is empty or over its card's limits")]
    Address,
    #[error("no saved recipient has that id")]
    Unknown,
}

impl BookError {
    /// The key of `crate::errors::CODES` (`ERROR_CONTROL.md`).
    pub fn key(self) -> &'static str {
        match self {
            Self::Full => "address_book.full",
            Self::Label => "address_book.label_invalid",
            Self::Address => "address_book.address_invalid",
            Self::Unknown => "address_book.unknown",
        }
    }
}

fn checked_label(label: &str) -> Result<String, BookError> {
    let label = label.trim();
    if label.is_empty()
        || label.chars().count() > MAX_LABEL_CHARS
        || label.chars().any(char::is_control)
    {
        return Err(BookError::Label);
    }
    Ok(label.to_owned())
}

/// Saves `recipient` under `label`; the same recipient saved again only takes the new name.
/// Returns its id.
pub fn save(
    book: &mut Vec<SavedAddress>,
    label: &str,
    recipient: &Recipient,
) -> Result<u32, BookError> {
    let label = checked_label(label)?;
    if !recipient.is_valid() {
        return Err(BookError::Address);
    }
    let recipient = match recipient {
        Recipient::Free(text) => Recipient::Free(text.trim().to_owned()),
        Recipient::Assisted(address) => Recipient::Assisted(address.sanitized()),
    };
    if let Some(kept) = book.iter_mut().find(|s| s.recipient.same_as(&recipient)) {
        kept.label = label;
        return Ok(kept.id);
    }
    if book.len() >= MAX_SAVED {
        return Err(BookError::Full);
    }
    let id = book
        .iter()
        .map(|s| s.id)
        .max()
        .map_or(1, |m| m.saturating_add(1));
    book.push(SavedAddress {
        id,
        label,
        recipient,
    });
    Ok(id)
}

/// Gives a saved recipient a new name.
pub fn rename(book: &mut [SavedAddress], id: u32, label: &str) -> Result<(), BookError> {
    let label = checked_label(label)?;
    let kept = book
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or(BookError::Unknown)?;
    kept.label = label;
    Ok(())
}

/// Takes a recipient out of the book.
pub fn forget(book: &mut Vec<SavedAddress>, id: u32) -> Result<(), BookError> {
    let before = book.len();
    book.retain(|s| s.id != id);
    if book.len() == before {
        return Err(BookError::Unknown);
    }
    Ok(())
}

/// The book as read from a file: each entry checked on its own (a bad one is dropped alone),
/// ids distinct, at most [`MAX_SAVED`].
pub fn sanitized_book(book: &[SavedAddress]) -> Vec<SavedAddress> {
    let mut kept: Vec<SavedAddress> = Vec::new();
    for entry in book {
        let Ok(label) = checked_label(&entry.label) else {
            continue;
        };
        if !entry.recipient.is_valid()
            || kept
                .iter()
                .any(|k| k.id == entry.id || k.recipient.same_as(&entry.recipient))
        {
            continue;
        }
        kept.push(SavedAddress {
            id: entry.id,
            label,
            recipient: entry.recipient.clone(),
        });
        if kept.len() == MAX_SAVED {
            break;
        }
    }
    kept
}

/// Puts a Japanese or Korean recipient first among those its country offers on the next
/// cards, once, and keeps the newest few of each country. Blank or invalid ones are not kept,
/// nor a draft of it already kept (LESSONS: a field saved while typed hands its drafts on).
pub fn remember(recent: &mut Vec<Address>, address: &Address) {
    if !address.is_valid() || address.is_blank() {
        return;
    }
    let address = address.sanitized();
    recent.retain(|a| !same_address(a, &address) && !draft_of(a, &address));
    recent.insert(0, address);
    let mut seen = [0usize; 2];
    recent.retain(|a| {
        let n = &mut seen[a.country as usize];
        *n += 1;
        *n <= MAX_RECENT_PER_COUNTRY
    });
}

/// The remembered recipients as read from a file: valid, distinct, the newest few per country.
pub fn sanitized_recent(recent: &[Address]) -> Vec<Address> {
    let mut kept = Vec::new();
    for address in recent.iter().rev() {
        remember(&mut kept, address);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addassist::Country;

    fn tower() -> Address {
        let mut a = Address::new(Country::Jp);
        a.code = "1050011".into();
        a.region = "東京都".into();
        a.city = "港区".into();
        a.town = "芝公園".into();
        a.line = "4-2-8".into();
        a.name = "山田太郎".into();
        a.filled_by = "1.1".into();
        a
    }

    #[test]
    fn fr_133_saved_address_keeps_its_name_once_and_within_limits() {
        let mut book = Vec::new();
        let id = save(&mut book, "  Tokyo Tower ", &Recipient::Assisted(tower())).unwrap_or(0);
        assert_eq!(id, 1);
        assert_eq!(book[0].label, "Tokyo Tower");
        // The same recipient, filled under another version, takes only the new name.
        let mut again = tower();
        again.filled_by = "1.0".into();
        assert_eq!(
            save(&mut book, "Yamada", &Recipient::Assisted(again)),
            Ok(1)
        );
        assert_eq!((book.len(), book[0].label.as_str()), (1, "Yamada"));
        assert_eq!(
            save(&mut book, "", &Recipient::Free("Praha".into())),
            Err(BookError::Label)
        );
        assert_eq!(
            save(&mut book, &"x".repeat(61), &Recipient::Free("Praha".into())),
            Err(BookError::Label)
        );
        assert_eq!(
            save(&mut book, "Bad", &Recipient::Free("   ".into())),
            Err(BookError::Address)
        );
        assert_eq!(
            save(&mut book, "Bad", &Recipient::Free("x\n".repeat(6))),
            Err(BookError::Address)
        );
        assert_eq!(
            save(
                &mut book,
                "Museum",
                &Recipient::Free("Národní muzeum\n110 00 Praha 1".into())
            ),
            Ok(2)
        );
    }

    #[test]
    fn fr_133_saved_address_book_is_capped_renamed_and_forgotten() {
        let mut book = Vec::new();
        for n in 0..MAX_SAVED {
            assert!(
                save(
                    &mut book,
                    &format!("N{n}"),
                    &Recipient::Free(format!("Street {n}"))
                )
                .is_ok()
            );
        }
        assert_eq!(
            save(&mut book, "One more", &Recipient::Free("Street x".into())),
            Err(BookError::Full)
        );
        assert_eq!(rename(&mut book, 3, "Home"), Ok(()));
        assert_eq!(book[2].label, "Home");
        assert_eq!(rename(&mut book, 999, "Home"), Err(BookError::Unknown));
        assert_eq!(forget(&mut book, 3), Ok(()));
        assert_eq!(forget(&mut book, 3), Err(BookError::Unknown));
        assert_eq!(book.len(), MAX_SAVED - 1);
    }

    #[test]
    fn fr_133_a_bad_entry_drops_alone() {
        let good = SavedAddress {
            id: 1,
            label: "Tower".into(),
            recipient: Recipient::Assisted(tower()),
        };
        let bad_label = SavedAddress {
            id: 2,
            label: "\u{7}".into(),
            recipient: Recipient::Free("Praha".into()),
        };
        let blank = SavedAddress {
            id: 3,
            label: "Blank".into(),
            recipient: Recipient::Free(" ".into()),
        };
        let twin_id = SavedAddress {
            id: 1,
            label: "Twin".into(),
            recipient: Recipient::Free("Brno".into()),
        };
        let museum = SavedAddress {
            id: 4,
            label: "Museum".into(),
            recipient: Recipient::Free("Národní muzeum".into()),
        };
        let kept = sanitized_book(&[good.clone(), bad_label, blank, twin_id, museum.clone()]);
        assert_eq!(kept, vec![good, museum]);
    }

    #[test]
    fn fr_133_version_1_1_stamps_new_fills() {
        // The interface stamps a fill with the version `countries()` gives (addassist.tsx).
        let versions: Vec<String> = crate::addassist::countries()
            .map(|c| c.into_iter().map(|i| i.version).collect())
            .unwrap_or_default();
        assert_ne!(versions, Vec::<String>::new());
        assert!(versions.iter().all(|v| v == "1.1"));
    }

    #[test]
    fn fr_133_recent_recipients_are_per_country_newest_first_and_few() {
        let mut recent = Vec::new();
        for n in 0..10 {
            let mut a = tower();
            a.line = format!("4-2-{n}");
            remember(&mut recent, &a);
        }
        let mut seoul = Address::new(Country::Kr);
        seoul.code = "03172".into();
        seoul.name = "김민준".into();
        remember(&mut recent, &seoul);
        remember(&mut recent, &Address::new(Country::Kr)); // blank: not kept
        assert_eq!(recent.len(), MAX_RECENT_PER_COUNTRY + 1);
        assert_eq!(recent[0], seoul);
        assert_eq!(recent[1].line, "4-2-9");
        // Used again, it comes first once.
        let mut old = tower();
        old.line = "4-2-5".into();
        remember(&mut recent, &old);
        assert_eq!(recent[0].line, "4-2-5");
        assert_eq!(recent.iter().filter(|a| a.line == "4-2-5").count(), 1);
        assert_eq!(sanitized_recent(&recent), recent);
        // Typed field by field, only the whole one stays.
        let mut typed = Vec::new();
        let mut draft = Address::new(Country::Jp);
        draft.code = "1500001".into();
        remember(&mut typed, &draft);
        draft.name = "佐藤".into();
        remember(&mut typed, &draft);
        draft.line = "1-2-3".into();
        remember(&mut typed, &draft);
        assert_eq!(typed, vec![draft]);
    }
}
