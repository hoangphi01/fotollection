### Proč paměť neroste se složkou

Fotollection drží v knihovně jen pár bajtů na fotku (13 MB pro 5 035 fotek), miniatury zapisuje do mezipaměti na disku, místo aby je držel v paměti, v paměti uchovává omezený počet náhledů a v plné velikosti zpracovává najednou jen tolik fotek, kolik se vejde do jeho rozpočtu. Mřížka na obrazovce je virtuální: existují jen dlaždice, které vidíš. Pro test bylo stejných 95 fotek nalinkováno 53krát do složky s 5 035 soubory:

| | 95 fotek | 5 035 fotek |
|---|---|---|
| Import a EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| Miniatury 400 px do mezipaměti na disku | 47,8 fotky/s | 46,4 fotky/s |
| Export, 6 najednou | 18,0 fotky/s | 15,4 fotky/s, 0 selhalo |
| Nejvyšší spotřeba paměti za celý běh | 1,35 GB | 1,45 GB |

53krát víc fotek stojí o 0,1 GB víc. Dlouhý export je na fotku o něco pomalejší jen proto, že běžel přes pět minut, a procesor notebooku se při trvalé zátěži zpomalí.
