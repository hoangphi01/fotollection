// SPDX-License-Identifier: GPL-3.0-only
//! Subjects routed by the photo's source (FR-141 S6, ADR-0059; owner, 2026-11-20: "split the
//! metadata first, then decide how to apply the model"). The source, read from the metadata
//! before the pixels, sets which subjects CLIP may choose among and the bar a subject must reach:
//! a photo through a lens is never a screenshot or a meme, a screenshot never scenery. Chosen on
//! the owner's labels and argued by three independent reviews (RESULT §42).

use serde::Deserialize;

use super::SourceClass;
use super::clip;

/// The way a photo's subjects are read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubjectRoute {
    /// Taken through a lens: a phone's or another camera, a panorama, a film scan.
    Lens,
    /// A screenshot, certain from its metadata.
    Screen,
    /// Anything else: saved from an app, edited, a guess, or unknown.
    Open,
}

/// One route's subjects (empty: every subject) and the probability its likeliest must reach.
#[derive(Debug, Deserialize)]
pub struct Route {
    pub subjects: Vec<String>,
    pub min_p: f32,
    /// A screenshot goes this way only when every subject's softmax also names one of these
    /// (a file misread as a screenshot is not named "screen" with confidence).
    #[serde(default)]
    pub guard: bool,
}

/// The three routes of `rules.json`.
#[derive(Debug, Deserialize)]
pub struct Routes {
    pub lens: Route,
    pub screen: Route,
    pub open: Route,
}

impl Routes {
    fn of(&self, route: SubjectRoute) -> &Route {
        match route {
            SubjectRoute::Lens => &self.lens,
            SubjectRoute::Screen => &self.screen,
            SubjectRoute::Open => &self.open,
        }
    }

    /// Every route's subjects are known labels and its bar a probability.
    pub fn valid(&self, keys: &[&str]) -> bool {
        [&self.lens, &self.screen, &self.open].iter().all(|r| {
            (0.0..=1.0).contains(&r.min_p) && r.subjects.iter().all(|s| keys.contains(&s.as_str()))
        })
    }
}

/// The route a source takes; an unknown source is open.
pub fn route_of(class: Option<SourceClass>) -> SubjectRoute {
    match class {
        Some(
            SourceClass::Camera
            | SourceClass::Panorama
            | SourceClass::OtherCamera
            | SourceClass::AnotherPhone
            | SourceClass::FilmScan,
        ) => SubjectRoute::Lens,
        Some(SourceClass::Screenshot) => SubjectRoute::Screen,
        _ => SubjectRoute::Open,
    }
}

/// The subjects an embedding shows on its source's route, most likely first, and the bar the
/// likeliest must reach to be named.
pub fn routed(
    routes: &Routes,
    embedding: &[f32],
    class: Option<SourceClass>,
) -> (Vec<(&'static str, f32)>, f32) {
    let all = clip::subjects(embedding);
    let mut route = route_of(class);
    if route == SubjectRoute::Screen && routes.screen.guard {
        let top = all.first().map_or("", |s| s.0);
        if !routes.screen.subjects.iter().any(|s| s == top) {
            route = SubjectRoute::Open;
        }
    }
    let r = routes.of(route);
    if r.subjects.is_empty() {
        return (all, r.min_p);
    }
    // A softmax over some prompts is the summed shares renormalised (the ratios hold).
    let mut kept: Vec<(&'static str, f32)> = all
        .into_iter()
        .filter(|(k, _)| r.subjects.iter().any(|s| s == k))
        .collect();
    let total: f32 = kept.iter().map(|s| s.1).sum();
    if total > 0.0 {
        for s in &mut kept {
            s.1 /= total;
        }
    }
    (kept, r.min_p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routes() -> &'static Routes {
        super::super::subject_routes()
    }

    #[test]
    fn fr_141_the_bundled_routes_name_known_subjects() {
        assert!(routes().valid(clip::subject_keys()));
        let bad: Routes = serde_json::from_str(
            r#"{"lens":{"subjects":["tram"],"min_p":0.5},"screen":{"subjects":[],"min_p":0.3},"open":{"subjects":[],"min_p":1.5}}"#,
        )
        .unwrap_or_else(|e| unreachable!("{e}"));
        assert!(!bad.valid(clip::subject_keys()));
    }

    #[test]
    fn fr_141_a_photo_through_a_lens_is_never_a_screenshot_or_a_meme() {
        let labels = &clip::labels().labels;
        // The text "a screenshot" read as a camera's photo names no screen and no meme.
        let screen = labels["screen"][0].clone();
        let (best, min_p) = routed(routes(), &screen, Some(SourceClass::Camera));
        assert!(best.iter().all(|(k, _)| *k != "screen" && *k != "meme"));
        assert!((best.iter().map(|s| s.1).sum::<f32>() - 1.0).abs() < 1e-4);
        assert!((min_p - routes().lens.min_p).abs() < f32::EPSILON);
        // The same text as a screenshot stays a screen.
        let (best, _) = routed(routes(), &screen, Some(SourceClass::Screenshot));
        assert_eq!(best[0].0, "screen");
        // A guess, an app or no source at all reads every subject.
        for class in [
            Some(SourceClass::ScreenshotCropped),
            Some(SourceClass::App),
            None,
        ] {
            assert_eq!(route_of(class), SubjectRoute::Open);
        }
    }

    #[test]
    fn fr_141_a_screenshot_that_shows_a_landscape_reads_every_subject() {
        // The guard: a "screenshot" whose pixels name scenery goes the open way, so it is named
        // as itself, never forced into screen, document or meme.
        let trees = clip::labels().labels["scenery"][6].clone();
        let (best, min_p) = routed(routes(), &trees, Some(SourceClass::Screenshot));
        assert_eq!(best[0].0, "scenery");
        assert!((min_p - routes().open.min_p).abs() < f32::EPSILON);
    }
}
