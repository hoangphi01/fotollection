// SPDX-License-Identifier: GPL-3.0-only
//! Times each phase of our own RAW decoder on one file (ADR-0030), median of 3: parse and
//! decode, demosaic and colour, the embedded JPEG, fitting the camera's look, finishing.
//!
//! `cargo run --release -p fotollection-core --example raw_phases -- <raw>`

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation
)]

use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("a RAW file");
    let bytes = std::fs::read(path).expect("read");
    let codec = fotollection_core::codec::default_codec();
    for run in 0..3 {
        let t = Instant::now();
        let parsed = fotollection_rawcore::dng::parse(&bytes).expect("parse");
        let parse = t.elapsed();
        let rendering = fotollection_rawcore::color::rendering(&parsed.color).expect("colour");
        let ms = |t: Instant| t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let mut plane = fotollection_rawcore::demosaic::balance(&parsed.mosaic, rendering.gains);
        let balance = ms(t);
        let t = Instant::now();
        fotollection_rawcore::prepare::hot_pixels(&mut plane);
        let hot = ms(t);
        let t = Instant::now();
        fotollection_rawcore::prepare::green_balance(&mut plane);
        let green = ms(t);
        let t = Instant::now();
        let _malvar = fotollection_rawcore::demosaic::malvar(&plane, 1.0);
        let malvar = ms(t);
        let t = Instant::now();
        let mut camera = fotollection_rawcore::directional::directional(&plane, 1.0);
        let directional = ms(t);
        let t = Instant::now();
        fotollection_rawcore::falsecolour::suppress(&mut camera);
        let false_colour = ms(t);
        println!(
            "run {run}: parse + decompress {:.0} ms · balance {balance:.0} · hot pixels {hot:.0} · green {green:.0} · (Malvar {malvar:.0}) · directional {directional:.0} · false colour {false_colour:.0} ms",
            parse.as_secs_f64() * 1e3,
        );
        let t = Instant::now();
        let linear = fotollection_rawcore::develop(&bytes).expect("develop");
        let develop = t.elapsed();
        let t = Instant::now();
        let jpeg = codec
            .decode(fotollection_rawcore::dng::embedded_jpeg(&bytes).expect("a JPEG"))
            .expect("decode");
        let decode = t.elapsed();
        let t = Instant::now();
        let reference = fotollection_rawcore::Reference {
            width: jpeg.image.width as usize,
            height: jpeg.image.height as usize,
            rgb: &jpeg.image.data,
        };
        let look = fotollection_rawcore::match_look(&linear, &reference).expect("look");
        let fit = t.elapsed();
        let t = Instant::now();
        let pic = fotollection_rawcore::finish_look(&linear, &look);
        let finish = t.elapsed();
        println!(
            "run {run}: develop {:.0} ms · embedded JPEG {:.0} ms · look {:.0} ms · finish {:.0} ms ({}×{})",
            develop.as_secs_f64() * 1e3,
            decode.as_secs_f64() * 1e3,
            fit.as_secs_f64() * 1e3,
            finish.as_secs_f64() * 1e3,
            pic.width,
            pic.height
        );
    }
}
