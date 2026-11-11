### Was Facebook mit einem Foto macht und was wir vorher tun

Facebook verkleinert jedes Foto auf höchstens 2.048 px, komprimiert es neu mit etwa Qualität 71 und halbierter Farbe und ignoriert Farbprofile. Der Export zum Teilen erledigt diese Arbeit sauber vor dem Hochladen, so bleibt Facebook nichts mehr zu verkleinern: Das Foto wird über sein Profil nach sRGB umgewandelt, mit einem Lanczos-Filter in linearem Licht verkleinert, leicht geschärft und mit Qualität 88 gespeichert. Jede Einstellung wurde durch Messen gewählt: Stärkeres Schärfen (0,35 statt 0,2) schnitt schlechter ab, und Qualität 84 oder 80 verlor mehr, als sie sparte.

| Median | Originalgröße | Facebook, 2.048 px | Instagram, 1.080 px |
|---|---|---|---|
| Dateigröße (Kamera-JPEGs mit 3,08 MB) | 1,42 MB | 0,51 MB | 0,24 MB |
| SSIM gegenüber der idealen Verkleinerung (1 ist identisch) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

Nach Facebooks eigener Neukomprimierung bleibt das Ergebnis näher am Foto als auf dem üblichen Weg (SSIM 0,965 gegenüber 0,961), und ein Foto in Display P3 behält 99,6 % seiner Farbe statt 86 %, weil sein Profil in echte sRGB-Farben umgerechnet und nicht verworfen wurde. Die verkleinerten Vorgaben bleiben über ΔE 1 wegen der halbierten Farbe, die jede Plattform ohnehin anwendet; *Originalgröße* behält jedes Pixel bei etwa der halben Datei, unter ΔE 1.
