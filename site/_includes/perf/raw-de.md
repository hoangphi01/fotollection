### Was ein Sensor wirklich aufzeichnet

Der Sensor einer Kamera ist ein Raster aus Millionen winziger Mulden, den sogenannten Fotodioden. Solange der Verschluss offen ist, sammelt jede das Licht, das sie erreicht, und wandelt es in elektrische Ladung um; schließt er sich, macht ein Wandler aus jeder Ladung eine Zahl. Der Zusammenhang ist **linear**: Doppelt so viel Licht ergibt die doppelte Zahl. Unsere Augen sind überhaupt nicht linear, sie beurteilen Helligkeit im Verhältnis, und deshalb sieht eine RAW-Datei dunkel und flau aus, bis sie entwickelt ist.

Zwei Arten von Sensoren lesen dieses Raster aus:

- **CCD** (Charge-Coupled Device) schiebt die Ladung jeder Fotodiode Zeile für Zeile zu einem einzigen Verstärker am Rand des Chips. Es war der Sensor der ersten Digitalkameras und vieler früher Mittelformat-Rückteile: saubere Bilder, aber langsam ausgelesen und stromhungrig.
- **CMOS** gibt jeder Fotodiode einen eigenen Verstärker und liest viele von ihnen gleichzeitig. Es ist schneller, braucht weniger Strom und ist günstiger herzustellen, daher hat fast jede Kamera und jedes Smartphone heute einen. Die meisten CMOS-Sensoren lesen ihre Zeilen nacheinander aus, weshalb sich ein schnell bewegtes Motiv auf einem Foto mit elektronischem Verschluss neigen kann (der *Rolling Shutter*), und weshalb Stacked-Sensoren, die viel schneller auslesen, sich weniger neigen.

Die Zahlen haben weit mehr Stufen als ein JPEG. Eine Kamera speichert 12 oder 14 Bit pro Fotodiode, 4.096 oder 16.384 Stufen, wo ein JPEG 8 Bit behält, 256 Stufen. Diese zusätzlichen Stufen erlauben es, einen Schatten aufzuhellen oder einen hellen Himmel zurückzunehmen, ohne dass Streifen entstehen. Und kein Licht ist nicht null: Jeder Sensor addiert einen kleinen Versatz, den **Schwarzwert**, damit das Rauschen um Schwarz nicht abgeschnitten wird. Im Beispiel der Nikon D750 liegt Schwarz bei 600, und der hellste Wert, den der Sensor halten kann, der **Weißwert**, bei 15.520. Das Allererste, was jeder RAW-Entwickler tut, ist, diese beiden Zahlen auf 0 und 1 zu bringen, für jede Position des Farbfilters, weil manche Kameras jeder ihren eigenen Schwarzwert geben.

### Farbe kommt von einem Filter über jeder Fotodiode

Eine Fotodiode misst nur, wie viel Licht ankommt, nie seine Farbe. Um Farbe zu sehen, ist der Sensor mit einem **Farbfilter-Array** bedeckt: einem winzigen roten, grünen oder blauen Filter über jeder Fotodiode, sodass jede nur einen Teil des Spektrums zählt.

- **Bayer**, 1976 von Bryce Bayer bei Kodak patentiert, wiederholt ein Quadrat aus 2 × 2: ein Rot, zwei Grün, ein Blau. Die Hälfte der Fotodioden ist grün, weil unsere Augen den größten Teil ihres Sinns für Details aus der Mitte des Spektrums beziehen, so trägt Grün den Großteil der Schärfe. Fast jede Kamera nutzt dieses Muster, in einer von vier Ausrichtungen (RGGB, BGGR, GRBG, GBRG), die die Datei angeben muss.
- **X-Trans**, das Muster von Fujifilm seit 2012, wiederholt ein Quadrat aus 6 × 6 mit Rot, Grün und Blau in jeder Zeile und jeder Spalte. Seine unregelmäßige Anordnung macht Farbmoiré auf feinen, sich wiederholenden Details unwahrscheinlicher, sodass Fujifilm den Weichzeichnungsfilter weglassen kann, den viele Sensoren tragen, um den Preis einer schwierigeren Rekonstruktion.
- **Andere Bauarten:** Monochrom-Sensoren haben gar keinen Filter, so sieht jede Fotodiode volles Detail (die Monochrom-Kameras von Leica); der Foveon von Sigma stapelt drei Schichten, die Farbe in verschiedenen Tiefen des Siliziums aufnehmen; viele Smartphones fassen vier Fotodioden einer Farbe zusammen (*Quad Bayer*) und lesen sie bei wenig Licht als eine.

Ein Bayer-Sensor mit 24 MP zeichnet also 24 Millionen Zahlen auf, jede für eine einzige Farbe: Zwei der drei Farben jedes Pixels fehlen. Aus diesem Mosaik ein Bild zu machen, ist die Aufgabe des Demosaicing, unten.

### Was eine RAW-Datei enthält

Ein JPEG ist ein fertiges Bild: Die Kamera hat schon jede Entscheidung getroffen. Eine RAW-Datei behält die Zahlen des Sensors von vor diesen Entscheidungen, mit allem, was nötig ist, um sie später zu treffen:

- die Messwerte selbst, einer pro Fotodiode, und unter welcher Filterfarbe jeder lag;
- Schwarz- und Weißwert;
- den **aktiven Bereich** (die Fotodioden, die Licht gesehen haben; ein paar Zeilen und Spalten an den Rändern sind absichtlich abgedeckt, um Schwarz zu messen) und den **Standardzuschnitt** der Kamera;
- den **Weißabgleich**, den die Kamera bei der Aufnahme gewählt hat;
- **Farbmatrizen**, die sagen, wie das eigene Rot, Grün und Blau dieses Sensors mit den Farben zusammenhängt, die Menschen sehen;
- eine **Vorschau**, ein JPEG, das die Kamera selbst gemacht hat, mit eigener Farbe und eigenem Tonwert, dazu die EXIF-Daten.

Jeder Hersteller hat sein eigenes Format, meist auf TIFF aufgebaut und teilweise undokumentiert: CR2 und CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** ist das Format, das Adobe offen veröffentlicht (Version 1.7), genutzt von Ricoh, Leica, Pentax und vielen Smartphones, und dasjenige, dessen Regeln diese Seite folgt.

Fotollection liest DNG mit seinem eigenen Reader, geschrieben nach der Spezifikation, und die anderen Formate werden von rawler entpackt, einer LGPL-Bibliothek, die in Fotollection eingebaut ist. Die Vorschaubilder im Raster berühren die Sensordaten überhaupt nicht: Sie kommen aus der Vorschau in der Datei, in 9 bis 32 ms, und deshalb öffnet sich ein Ordner mit RAW-Dateien so schnell wie ein Ordner mit JPEGs.

### Demosaicing: die zwei fehlenden Farben ermitteln

Die zwei fehlenden Farben jedes Pixels aus seinen Nachbarn wiederherzustellen, heißt **Demosaicing**, und dieser Schritt entscheidet mehr als jeder andere, wie scharf und wie sauber ein RAW-Foto aussieht. Jede Methode steht vor demselben Dilemma: Nachbarn zu mitteln ist in flachen Bereichen sicher, aber über eine Kante hinweg mischt es zwei verschiedene Flächen, die Kunst liegt also darin zu wissen, welche Nachbarn zusammengehören.

- **Bilinear** mittelt einfach die nächsten Fotodioden jeder Farbe. Es ist die Untergrenze, an der jede Methode gemessen wird: weich, und mit farbigen Säumen an jeder scharfen Kante, weil Rot, Grün und Blau über leicht verschiedene Stellen gemittelt werden.
- **Malvar–He–Cutler** (2004) beginnt mit diesem Mittelwert und korrigiert ihn mit dem Detail der Farbe, die die Fotodiode gesehen hat, über feste Filter aus 5 × 5. Es ist schärfer und noch immer sehr schnell. Fotollection hat bis v0.10.0 damit entwickelt.
- **PPG** (Patterned Pixel Grouping) entscheidet zuerst, in welcher Richtung sich das Bild am wenigsten ändert, und interpoliert nur entlang dieser Richtung. rawler nutzt es, und das taten auch die Windows- und Linux-Versionen von Fotollection für die meisten Formate bis v0.10.0.
- **RCD** (Ratio Corrected Demosaicing, von Luis Sanz Rodríguez) arbeitet mit Verhältnissen zwischen den Farben statt mit Differenzen, was in hellen wie dunklen Bereichen besser hält, und ist für wenige Artefakte auf feinem Detail bekannt; darktable und RawTherapee bieten es an.
- **Markesteijn**, nach Frank Markesteijn, ist die für X-Trans gemachte Methode, in einem Durchgang oder, langsamer und sauberer, in drei.
- **Das Demosaicing von Fotollection seit v0.10.0** übernimmt von RCD die Art, die Richtung zu bestimmen, schätzt dann Grün von jeder Seite, korrigiert um die eigene Farbe der Stelle (wie es Hamilton und Adams taten), gewichtet danach, wie stark sich jede Seite ändert. Wir haben zuerst den eigenen Verhältnisschritt von RCD geschrieben, und er schnitt auf unseren Testbildern schlechter ab, also wird die Variante ausgeliefert, die gewonnen hat, unter ihrem eigenen Namen.

Wenn eine Methode falsch rät, zeigt sich das auf vier typische Arten:

- **Zipper:** eine Reihe abwechselnd heller und dunkler Pixel entlang einer scharfen Kante, wo die Methode Nachbarn von beiden Seiten gewählt hat;
- **Labyrinth:** ein feines, wurmartiges Muster in flachen Bereichen, wenn die zwei Grün eines Bayer-Quadrats nicht ganz gleich lesen (sie liegen auf Zeilen mit verschiedenen Nachbarn und weichen auf vielen Sensoren um ein oder zwei Prozent ab);
- **Falschfarbe:** farbige Punkte oder Säume auf Detail, das eigentlich grau ist, etwa Stoff, Haar, eine Backsteinmauer oder ein ferner Zaun;
- **Moiré:** farbige Wellen über sehr feinen, sich wiederholenden Mustern, feiner, als der Farbfilter abtasten kann.

**Wie wir eine Methode beurteilen.** Unsere Augen lassen sich von einem schärfer wirkenden Ergebnis leicht täuschen, deshalb wird jede Methode an Bildern bewertet, deren Wahrheit genau bekannt ist. Ein Computer zeichnet fünf Testbilder mit 768 px: eine Zonenplatte (Ringe, die zum Rand hin feiner werden, bis zum feinsten Detail, das die grünen Fotodioden halten können), einen Siemensstern mit 72 Speichen, Balken gesättigter Farbe auf Grau in drei Richtungen, weiche Farbverläufe und ein flaches Grau, dessen zwei Grün um 2 % abweichen. Jedes wird in das umgewandelt, was ein Bayer-Sensor aufzeichnen würde, demosaiciert und mit dem Original verglichen. PSNR misst den Gesamtfehler in Dezibel (höher ist besser; alle 6 dB halbieren ihn); Falschfarbe ist die mittlere Farbe, die auf den grauen Testbildern erscheint; der Kantenfehler ist der mittlere Fehler innerhalb von 2 px um eine Kante.

| Testbild (768 px) | Bilinear | Malvar (unseres bis v0.10.0) | Unseres seit v0.10.0 |
|---|---|---|---|
| Zonenplatte: PSNR, höher ist besser | 13,0 dB | 17,3 dB | 16,5 dB |
| Zonenplatte: Falschfarbe, niedriger ist besser | 65,9 | 43,0 | 18,7 |
| Siemensstern: PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Siemensstern: Falschfarbe | 9,4 | 8,4 | 0,16 |
| Gesättigte Farbbalken: Fehler an den Kanten, niedriger ist besser | 11,6 | 21,6 | 10,5 |
| Weiche Verläufe: PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Flaches Grau mit Grün-Abweichung: Labyrinth, niedriger ist besser | 0,38 | 0,38 | 0 |
| Textur mit Hotpixeln: verbleibend pro 10.000, niedriger ist besser | 35,8 | 66,5 | 0 |

Malvar ist auf feinem grauem Detail etwa 4 dB besser als bilinear, wie sein Paper verspricht, aber schlechter an Kanten zwischen kräftigen Farben, weil seine Korrektur das Detail einer Farbe für die anderen leiht und annimmt, dass sie sich gemeinsam ändern, was ein rotes Schild an einer grauen Wand nicht tut. Keine der beiden beseitigt das Labyrinth, das kein Problem des Demosaicing ist, sondern ein Unterschied zwischen den zwei Grün, der vorher ausgeglichen werden muss. Das sind die Lücken, die unsere Kette seit v0.10.0 schließt: Hotpixel und das Grün-Ungleichgewicht werden behoben, solange jede Stelle nur hält, was sie gemessen hat, das richtungsabhängige Demosaicing stellt die Farben wieder her, und ein kleiner Median auf den Farbdifferenzen entfernt die verbleibende Falschfarbe. Auf den Testbildern gewinnt es überall außer beim Gesamt-PSNR der Zonenplatte, 0,8 dB unter Malvar nahe dem feinsten Detail, wo jede Methode Aliasing zeigt; selbst dort halbiert sich seine Falschfarbe. Bei den neun echten Dateien bringt es die Entwicklung jeder Bayer-Kamera genauso nah oder näher an das eigene JPEG der Kamera und kostet bei 24 MP etwa 180 ms, gegenüber 40 bei Malvar: ein paar Zehntelsekunden mehr pro Foto, für Abzüge mit saubererem Detail.

### Von den Farben der Kamera zu deinen

Rot, Grün und Blau der Kamera sind nicht die deines Bildschirms: Die Filter jedes Sensors nutzen eigene Farbstoffe, und sie überlappen anders als die Zapfen unserer Augen. Die Farbmatrizen in der Datei wandeln die Farben der Kamera in **CIE XYZ** um, das 1931 definierte Standardmodell des menschlichen Farbsehens, und von dort in **sRGB**, den Farbraum der meisten Bildschirme und der Fotolabore.

- **Der Weißabgleich** kommt zuerst: eine Verstärkung für Rot und für Blau, damit etwas Weißes in der Szene neutral herauskommt. Die Kamera speichert die Verstärkungen, die sie gewählt hat, und Fotollection wendet sie vor dem Demosaicing an, so bleiben die Farbdifferenzen, auf die sich die Methoden oben stützen, klein.
- Ein DNG trägt meist **zwei Matrizen**, eine unter warmem Glühlampenlicht gemessen und eine unter Tageslicht. Die richtige für dein Foto liegt dazwischen: Der Entwickler ermittelt das Licht der Aufnahme aus ihrem Weißabgleich, als Farbtemperatur, und mischt die beiden. Fotollection macht dasselbe für die anderen Formate, mit den Matrizen, die rawler für jedes Kameramodell kennt.
- All das geschieht in **linearem Licht**, auf der eigenen Skala des Sensors, vor jeder Kurve. Farben in linearem Licht zu mischen, zu skalieren und zu verkleinern ergibt physikalisch richtige Ergebnisse; nach der Kurve getan, verdunkelt es Kanten und verschiebt Farben.

**Der Look der Kamera.** Das eigene JPEG einer Kamera ist nicht neutral: Jeder Hersteller fügt seinen Kontrast, seine Farbe und seine Hauttöne hinzu, und dieser Look ist ein Teil dessen, warum Menschen eine Marke wählen. Seit v0.10.0 öffnet sich ein RAW-Foto mit einer **neutralen** Wiedergabe, treu zur Matrix des Sensors, mit der Vignettierungskorrektur und der Belichtung, die die Kamera gewählt hat; **Like the camera's JPEG** ist einen Klick entfernt in der Karte *RAW colour* eines RAW-Fotos, für ein Foto oder für alle RAW auf einmal. (Bis v0.10.0 wurde jedes RAW an das JPEG der Kamera angepasst; ein damals gespeichertes Projekt behält diesen Look.) Diese Wahl passt das JPEG der Kamera auf beide Arten an, eine Kurve und eine Matrix oder eine Kurve pro Kanal, und behält, was für diese Datei näher kommt: Bei der Nikon D750 halbiert sie den Unterschied, von 2,26 auf 1,16 in ΔE2000. Die Vorschaubilder im Raster behalten die eigene Vorschau der Kamera, wie in Lightroom.

### Tonwert: von linearen Zahlen zu einem Bild

Lineare Zahlen sehen auf einem Bildschirm dunkel und matt aus, weil eine Verdopplung des Lichts, die der Sensor als doppelte Zahl aufzeichnet, für uns wie eine Stufe Helligkeit aussieht. Eine **Tonkurve** macht aus der Skala des Sensors ein Bild: Sie legt das mittlere Grau fest, hebt die Tiefen an, lässt die Lichter sanft auslaufen, statt sie auf einmal abzuschneiden, und gibt dem Bild seinen Kontrast. Das Bild wird dann mit der sRGB-Kurve (seinem „Gamma“) gespeichert, die mehr der 256 Stufen eines JPEG für die Tiefen aufwendet, wo unsere Augen Stufen am meisten bemerken.

Zwei Korrekturen, die eine Kamera in ihrem JPEG vornimmt, stehen nicht in den RAW-Daten, und beide sind für Abzüge wichtig:

- **Vignettierung:** Jedes Objektiv ist zu den Ecken hin dunkler. Fotollection misst aus der eigenen Vorschau der Kamera, wie stark sie die Ecken aufgehellt hat, als radiale Verstärkung. Als unsere DNG-Entwicklung zum ersten Mal mit dem JPEG der Kamera verglichen wurde, war die Mitte etwa 4 L\* zu hell und die Ecken 1,5 zu dunkel; die radiale Verstärkung brachte den Unterschied von ΔE 3,8 auf 2,67, mit jedem Bereich des Fotos innerhalb von ±0,5 L\*.
- **Verzeichnung:** die leichte Tonnen- oder Kissenform eines Objektivs. Micro-Four-Thirds-Kameras (Panasonic, Olympus) sind darauf ausgelegt, dass die Kamera sie korrigiert, und ihre Dateien halten fest, wie. Auf einem Mac werden diese Dateien von macOS entwickelt, das die Korrektur anwendet; sie in unserer eigenen Entwicklung zu korrigieren, ist geplant (FR-085).

### Wie Fotollection ein RAW entwickelt, Schritt für Schritt

Für eine vollständige Entwicklung, die hinter der Druckvorschau, dem Zoom und dem Export steht:

1. **Erkennen** des Formats an den ersten Bytes der Datei, nie an ihrem Namen, so wird auch eine umbenannte Datei oder ein RAW, das wie ein einfaches TIFF aussieht, richtig gelesen.
2. **Lesen** der Zahlen des Sensors: DNG mit unserem eigenen Reader, die anderen Formate von rawler entpackt. Eine zu große oder beschädigte Datei wird abgelehnt, bevor ein Pixel dekodiert wird, mit einem Fehlercode.
3. **Werte:** Schwarz auf 0 und Weiß auf 1, für jede Position des Farbfilters, gezählt ab der Ecke des aktiven Bereichs.
4. **Weißabgleich**, dann **Demosaicing**: zuerst Hotpixel und die Grün-Balance, das richtungsabhängige Demosaicing, der Median gegen Falschfarbe (Markesteijn für X-Trans kommt in v0.15.0).
5. **Farbe:** die Matrizen der Kamera, gemischt für das Licht der Aufnahme, in linearem Licht.
6. **Tonwert und Look**, angepasst an die Vorschau der Kamera, mit ihrer Vignettierungskorrektur.
7. Ab hier ist ein RAW-Foto wie jedes andere: Zuschnitt, Datumsstempel, Rahmen, Druckformat und Farbprofil, exportiert in voller Auflösung in 4:4:4 mit Qualität 95.

Ein paar Regeln gelten durchgehend. Es läuft immer nur eine Entwicklung gleichzeitig, verteilt auf jeden Prozessorkern und nie auf dem eigenen Thread des Fensters, so reagiert Fotollection weiter, während ein 30-MP-Foto entwickelt wird. Exporte laufen immer auf dem Prozessor, so kommt ein Abzug auf jedem Computer gleich heraus; die Grafikkarte beschleunigt nur, was du beim Bearbeiten siehst. Und der Code ist unser eigener, geschrieben nach den veröffentlichten Spezifikationen und Papers, nicht aus anderen Projekten kopiert, und dann geprüft: an den synthetischen Testbildern oben, an neun RAW-Dateien von neun Kameras (CC0, von raw.pixls.us) und durch absichtlich abgeschnittene und beschädigte Dateien, damit eine schlechte Datei ein Fehler ist, nie ein Absturz.

**Seit v0.10.0: unsere eigene Entwicklung für jede Bayer-Kamera.** rawler entpackt nur die Dateien, der Rest ist von Fotollection. Vollständige Entwicklung der neun Beispieldateien, Median aus 3, auf demselben Laptop: unsere Entwicklung mit ihrem saubereren Demosaicing, gegenüber dem einfacheren von rawler und dem von macOS:

| Kamera (Format) | Fotollections eigene | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1.044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1.164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1.647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | als Nächstes | 524 ms | 1.339 ms |

Unsere braucht für ihr saubereres Demosaicing und ihre angepasste Farbe eine Zehntel- bis Viertelsekunde mehr als die von rawler und bleibt bei den meisten Kameras schneller als macOS; sie geschieht einmal pro Foto, und ein Regler stellt danach eine bereitgehaltene Kopie in 4 bis 11 ms neu fertig. Auf einem Mac bleiben die Dateien von Panasonic und Olympus wegen der Objektivkorrektur bei macOS, bis unsere eigene Entwicklung die Verzeichnung korrigiert. Die grauen Felder der S5 kommen neutraler heraus als über rawler (ein Farbstich von 1,42 gegenüber 2,43, wobei 0 vollkommen grau ist). Fujifilm X-Trans folgt mit Markesteijn, und danach die Entwicklung auf der Grafikkarte für die Vorschau, mit dem Ergebnis des Prozessors als Referenz.
