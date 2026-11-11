### Warum der Speicher nicht mit dem Ordner wächst

Fotollection behält in seiner Bibliothek ein paar Bytes pro Foto (13 MB für 5.035 Fotos), schreibt Vorschaubilder in einen Cache auf der Festplatte, statt sie festzuhalten, hält eine begrenzte Zahl von Vorschauen im Arbeitsspeicher und bearbeitet nur so viele Fotos in voller Größe gleichzeitig, wie in sein Budget passen. Das Raster auf dem Bildschirm ist virtuell: Nur die Kacheln, die du sehen kannst, existieren. Zum Testen wurden dieselben 95 Fotos 53-mal in einen Ordner mit 5.035 Dateien verlinkt:

| | 95 Fotos | 5.035 Fotos |
|---|---|---|
| Import und EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| Vorschaubilder mit 400 px in den Festplatten-Cache | 47,8 Fotos/s | 46,4 Fotos/s |
| Export, 6 gleichzeitig | 18,0 Fotos/s | 15,4 Fotos/s, 0 fehlgeschlagen |
| Spitzenspeicher des ganzen Durchlaufs | 1,35 GB | 1,45 GB |

53-mal mehr Fotos kosten 0,1 GB mehr. Der lange Export ist pro Foto nur deshalb etwas langsamer, weil er länger als fünf Minuten lief und der Prozessor eines Laptops unter Dauerlast langsamer wird.
