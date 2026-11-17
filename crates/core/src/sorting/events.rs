// SPDX-License-Identifier: GPL-3.0-only
//! A roll cut into events (FR-141 S4): `PhotoTOC`'s adaptive gaps (Platt, Czerwinski and
//! Field, 2003). Photos in time order; a gap is a boundary when its logarithm passes the mean
//! of the `2d` log-gaps around it by `ln K`, so a two-hour pause cuts a day of minute-apart
//! shots but not a week of hour-apart ones. A gap of `always_cut_h` or more always cuts (two
//! photos days apart have no neighbours to compare with), and so does a jump in place (two
//! shots in a row further apart than `split_km`).

/// One photo as the events read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moment {
    /// Camera-local seconds.
    pub t: i64,
    pub gps: Option<(f32, f32)>,
}

/// `PhotoTOC`'s window and threshold, the gap that always cuts, and the jump in place.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub struct EventRules {
    pub d: usize,
    pub k: f64,
    pub split_km: f64,
    pub always_cut_h: i64,
}

/// The events, each the indexes of its photos in time order; the events in time order.
pub fn events(moments: &[Moment], rules: &EventRules) -> Vec<Vec<usize>> {
    let mut order: Vec<usize> = (0..moments.len()).collect();
    order.sort_by_key(|&i| (moments[i].t, i));
    if order.len() < 2 {
        return if order.is_empty() {
            Vec::new()
        } else {
            vec![order]
        };
    }
    #[allow(clippy::cast_precision_loss)]
    let logs: Vec<f64> = order
        .windows(2)
        .map(|w| ((moments[w[1]].t - moments[w[0]].t).max(1) as f64).ln())
        .collect();
    // Prefix sums for the window means.
    let mut sums = vec![0.0; logs.len() + 1];
    for (i, l) in logs.iter().enumerate() {
        sums[i + 1] = sums[i] + l;
    }
    let threshold = rules.k.ln();
    let mut out = vec![vec![order[0]]];
    for (i, w) in order.windows(2).enumerate() {
        let (lo, hi) = (i.saturating_sub(rules.d), (i + rules.d + 1).min(logs.len()));
        let others = hi - lo - 1;
        #[allow(clippy::cast_precision_loss)]
        let adaptive =
            others > 0 && logs[i] >= threshold + (sums[hi] - sums[lo] - logs[i]) / others as f64;
        let long = moments[w[1]].t - moments[w[0]].t >= rules.always_cut_h * 3600;
        let jump = match (moments[w[0]].gps, moments[w[1]].gps) {
            (Some(a), Some(b)) => km(a, b) > rules.split_km,
            _ => false,
        };
        if adaptive || long || jump {
            out.push(Vec::new());
        }
        if let Some(last) = out.last_mut() {
            last.push(w[1]);
        }
    }
    out
}

fn km(a: (f32, f32), b: (f32, f32)) -> f64 {
    crate::geocode::distance_km(
        f64::from(a.0),
        f64::from(a.1),
        f64::from(b.0),
        f64::from(b.1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: EventRules = EventRules {
        d: 10,
        k: 17.0,
        split_km: 50.0,
        always_cut_h: 24,
    };

    fn run(start: i64, step: i64, n: usize) -> Vec<Moment> {
        (0..n)
            .map(|i| Moment {
                t: start + step * i64::try_from(i).unwrap_or(0),
                gps: None,
            })
            .collect()
    }

    #[test]
    fn fr_141_a_long_pause_against_short_ones_cuts() {
        let mut m = run(0, 60, 30);
        m.extend(run(30 * 60 + 7200, 60, 30));
        assert_eq!(events(&m, &RULES).len(), 2);
        // The same pause among hour-long gaps is no boundary.
        let mut m = run(0, 3600, 30);
        m.extend(run(30 * 3600 + 7200, 3600, 30));
        assert_eq!(events(&m, &RULES).len(), 1);
    }

    #[test]
    fn fr_141_three_days_out_make_three_events() {
        let day = 86_400;
        let mut m = run(0, 120, 20);
        m.extend(run(3 * day, 120, 20));
        m.extend(run(7 * day, 120, 20));
        let e = events(&m, &RULES);
        assert_eq!(e.iter().map(Vec::len).collect::<Vec<_>>(), vec![20, 20, 20]);
    }

    #[test]
    fn fr_141_a_jump_in_place_cuts() {
        let mut m = run(0, 60, 20);
        for (i, x) in m.iter_mut().enumerate() {
            x.gps = Some(if i < 10 { (47.5, 19.0) } else { (48.2, 16.4) });
        }
        assert_eq!(events(&m, &RULES).len(), 2);
        assert_eq!(events(&[], &RULES), Vec::<Vec<usize>>::new());
        let two_days = [run(0, 1, 1)[0], run(2 * 86_400, 1, 1)[0]];
        assert_eq!(events(&two_days, &RULES).len(), 2);
    }
}
