### Mit tesz a Facebook egy fotóval, és mit teszünk mi előbb

A Facebook minden fotót legfeljebb 2048 px-re kicsinyít, kb. 71-es minőséggel és felezett színnel tömöríti újra, és figyelmen kívül hagyja a színprofilokat. A Share export ezt a munkát a feltöltés előtt rendesen elvégzi, így a Facebooknak nem marad mit kicsinyítenie: a fotó a profilján át sRGB-be kerül, lineáris fényben Lanczos-szűrővel méreteződik át, enyhén élesedik, és 88-as minőséggel mentődik. Minden beállítást mérés alapján választottunk: az erősebb élesítés (0,2 helyett 0,35) rosszabb pontszámot kapott, a 84-es vagy 80-as minőség pedig többet veszített, mint amennyit megtakarított.

| Medián | Eredeti méret | Facebook, 2048 px | Instagram, 1080 px |
|---|---|---|---|
| Fájlméret (3,08 MB-os fényképezőgépes JPEG-ek) | 1,42 MB | 0,51 MB | 0,24 MB |
| SSIM az ideális átméretezéshez képest (1 az azonos) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

A Facebook saját újratömörítése után az eredmény közelebb marad a fotóhoz, mint a szokásos úton (SSIM 0,965 a 0,961-gyel szemben), és egy Display P3-as fotó a színe 99,6 %-át tartja meg 86 % helyett, mert a profilját valódi sRGB-színekké alakítottuk, nem dobtuk el. Az átméretezett presetek a felezett szín miatt maradnak ΔE 1 felett, amelyet egyébként is minden platform alkalmaz; az *Original size* minden pixelt megtart nagyjából fele akkora fájlban, ΔE 1 alatt.
