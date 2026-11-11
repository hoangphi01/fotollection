### Co Facebook robi ze zdjęciem i co robimy najpierw my

Facebook zmniejsza każde zdjęcie do najwyżej 2048 px, kompresuje je ponownie w jakości około 71 z połową rozdzielczości koloru i ignoruje profile kolorów. Eksport Share wykonuje tę pracę porządnie przed wysłaniem, więc Facebook nie ma już czego zmniejszać: zdjęcie jest konwertowane do sRGB przez swój profil, zmniejszane filtrem Lanczos w świetle liniowym, lekko wyostrzane i zapisywane w jakości 88. Każde ustawienie wybrano na podstawie pomiarów: mocniejsze wyostrzenie (0,35 zamiast 0,2) wypadło gorzej, a jakość 84 czy 80 traciła więcej, niż oszczędzała.

| Mediana | Original size | Facebook, 2048 px | Instagram, 1080 px |
|---|---|---|---|
| Rozmiar pliku (JPEG-i z aparatu po 3,08 MB) | 1,42 MB | 0,51 MB | 0,24 MB |
| SSIM względem idealnego zmniejszenia (1 to identyczne) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

Po własnej ponownej kompresji Facebooka wynik pozostaje bliższy zdjęciu niż przy zwykłym sposobie (SSIM 0,965 wobec 0,961), a zdjęcie w Display P3 zachowuje 99,6 % swojego koloru zamiast 86 %, bo jego profil zamieniono na prawdziwe kolory sRGB, zamiast go usunąć. Presety ze zmniejszeniem pozostają powyżej ΔE 1 z powodu połowy rozdzielczości koloru, którą i tak stosuje każda platforma; *Original size* zachowuje każdy piksel przy mniej więcej połowie rozmiaru pliku, poniżej ΔE 1.
