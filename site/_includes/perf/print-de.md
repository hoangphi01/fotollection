### Warum Abzüge aus einem Web-Editor flau aussehen

Der Test exportiert 20 Kamerafotos und 5 Fassungen in Display P3 (die weitere Farbe neuerer Smartphones und Macs) über den echten Export von Fotollection, liest dann beide Dateien mit einem unabhängigen Decoder, rechnet sie über ihre Farbprofile nach CIELAB um und vergleicht sie Pixel für Pixel mit dem Original. Ein typischer Export aus einem Web-Editor durchläuft denselben Vergleich: Profil verworfen, auf 2.000 px verkleinert und für den Abzug wieder vergrößert, mit Qualität 80 und halbierter Farbe (4:2:0) gespeichert.

| Median | Fotollection, sRGB | Web-Editor | Fotollection, Display P3 | Web-Editor |
|---|---|---|---|---|
| ΔE2000 (Farbfehler, unter 1 unsichtbar) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Erhaltene Sättigung | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Erhaltenes feines Detail | 99,8 % | 13 % | 100 % | 18 % |

Zwei Ursachen erklären den flauen Eindruck. Das Verkleinern auf 2.000 px und erneute Vergrößern für das Labor wirft 87 % des feinen Details weg, das einen Abzug knackig wirken lässt. Und das Verwerfen des Farbprofils liest ein Foto in Display P3, als wäre es sRGB, was ihm 13 % seiner Sättigung nimmt. Fotollection behält das Profil (25 von 25 Dateien), exportiert in der vollen Auflösung des Fotos mit voller Farbe (4:4:4) bei Qualität 95 (so damals gemessen; seit v0.13.0 standardmäßig 90, noch immer unter einem sichtbaren Unterschied), und seine größeren Fehler liegen nur an scharfen Kanten, wo jedes JPEG am härtesten arbeitet. Noch ausstehend: echte HEIC-Dateien vom iPhone und Abzüge, die mit einem Spektralfotometer gemessen werden.
