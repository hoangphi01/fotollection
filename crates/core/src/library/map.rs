// SPDX-License-Identifier: GPL-3.0-only
//! RollSense's map (FR-141; owner 2026-11-18): one country of the scope's photos, its outline
//! and a pin for each place with photos, from the facets' own places so the map and the list
//! never disagree.

use super::Library;
use crate::geo_shapes::{CountryMap, country_map};

impl Library {
    /// The country `country` (as the facets name it) of the photos in `folders`, drawn with
    /// its places' pins; `None` when it has no located place or no bundled outline.
    pub fn country_map(&self, folders: &[u16], country: &str) -> Option<CountryMap> {
        let facets = self.facets(folders);
        let places = &facets
            .countries
            .iter()
            .find(|c| c.country == country)?
            .places;
        let mut code = None;
        let points: Vec<(u32, f64, f64, u32)> = places
            .iter()
            .filter_map(|p| {
                let place = crate::geocode::by_id(p.id).ok().flatten()?;
                code.get_or_insert(place.country_code);
                Some((p.id, place.lat, place.lon, p.count))
            })
            .collect();
        country_map(&code?, &points)
    }
}
