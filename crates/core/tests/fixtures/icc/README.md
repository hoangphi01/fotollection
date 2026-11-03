# ICC profiles for tests

`DisplayP3-v4.icc` and `AdobeCompat-v4.icc` are from Clinton Ingram's [Compact ICC Profiles](https://github.com/saucecontrol/Compact-ICC-Profiles) (`profiles/`, fetched 2026-11-03), released under **CC0 1.0** (public domain). They are used only by tests (`colour::tests`, `tests/colour_screen.rs`) and are not shipped with the app. Their descriptions are "sP3" and "A98C", which is why `facts::profile_space` tells spaces apart by their primaries first.
