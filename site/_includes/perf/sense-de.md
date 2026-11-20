### Ein Lasttest, kein gewöhnlicher Tag

Die meisten sortieren eine Reise nach der anderen, 2.000 bis 3.000 Fotos. Um zu sehen, ob Fotollection weit darüber hinaus stabil bleibt, bekam RollSense 10.044 Fotos auf einmal, auf einem gewöhnlichen Windows-Laptop von 2020 (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): Kamera-JPEGs, Handyfotos, Bildschirmfotos und RAW-Dateien von Canon und Sony. Es las 9.639 davon in 70 Minuten auf allen Kernen, der Prozessor die ganze Zeit ausgelastet, der Speicher zwischen 400 und 850 MB, ohne zu wachsen.

### Wohin die Zeit geht

Jedes Foto wird einmal dekodiert, verkleinert und vom Motivmodell gelesen. Das Modell allein braucht auf diesem Laptop 128 ms pro Foto und der ganze Schritt 431 ms, also sind etwa 70 % der Zeit das Lesen und Dekodieren des Fotos selbst, meist JPEG- und RAW-Dateien in voller Größe. Die Grafikkarte blieb untätig, höchstens 1,5 %: all das läuft in Rust auf dem Prozessor.

### Mehr Threads und die Grenze eines Laptops

Zwei Threads brauchen 614 ms pro Foto, vier 498 ms und acht 438 ms. Ein dünner Laptop teilt 15 Watt auf seine Kerne auf, also läuft jeder langsamer, wenn alle acht arbeiten (im Mittel 1,8 GHz statt 3,4 GHz in Ruhe), und vier der acht Threads teilen sich einen Kern mit den anderen vier.

| | Dieser Laptop (gemessen) |
|---|---|
| Eine Reise mit 2.500 Fotos | 18 Min. |
| Die Mediathek eines Handys mit 15.000 Fotos | 108 Min. |
| Ein Foto erneut aus dem Cache | 1,3 ms |

Gemessen am 20.11.2026 mit `sort_pass_bench`, Messwerte jede Sekunde.
