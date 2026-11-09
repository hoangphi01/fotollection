# Lightroom reference renders for FR-089

These 14 presets are Fotollection's own (written for the bench, not copied from Adobe or
anyone), each moving one group of sliders, the last two several. Rendered by Lightroom on
the same photos, they are the ground truth `preset_bench` tunes the preset mappings against
(ADR-0045, RESULT §39).

## What to export

1. **Import the presets.** In Lightroom Classic's Develop module, Presets panel, **+** →
   *Import Presets…*, choose the 14 `.xmp` files of this folder. They appear in the group
   *Fotollection bench*.
2. **Import the photos**, unedited, from `~/Desktop/PRAGUE 11.26`:
   - JPEG: `P1067819.JPG`, `P1067886.JPG`, `P1067947.JPG`, `P1068013.JPG`, `P1068099.JPG`, `P1068164.JPG`;
   - RAW: `P1067941.RW2`, `P1067947.RW2`, `P1067953.RW2`, `P1067959.RW2` (leave the profile
     as Lightroom sets it, *Adobe Color*).
3. **For each photo and each preset:** reset the photo (*Reset*), apply the preset, export.
   A virtual copy per preset is the quickest way (14 virtual copies a photo, a preset each).
4. **Export settings** (save them as an export preset):
   - JPEG, quality 100, colour space sRGB;
   - no resizing (full size), no output sharpening, no watermark;
   - metadata: all;
   - file name: `{Filename}__{preset number}`, for example `P1067819__01.jpg` (*Custom
     Name* with the virtual copy name set to the preset's number is enough);
   - folder: `~/Desktop/fotollection-lr-refs`.
5. **Also export each photo once with no preset** (`P1067819__00.jpg`): the baseline that
   separates our develop from the preset's own effect.

That is 10 photos × 15 renders = 150 files, about 1.5 GB. Tell Claude the folder when done.

## What happens next

`cargo run --release -p fotollection-core --example preset_bench -- ~/Desktop/fotollection-lr-refs`
renders each photo through Fotollection with the same preset, compares it with Lightroom's
render (mean and p95 ΔE2000), sweeps each mapping's strength, and keeps the best. JPEGs tune
the mappings; RAW files are reported apart, as our develop and Adobe's differ before any
preset.
