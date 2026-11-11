### Vorher: jedes Foto beim Klick dekodiert

Bis v0.10.0 wurde eine Vorschau erst gemacht, wenn du sie angefordert hast: Datei gelesen, Foto dekodiert (ein RAW entwickelt), aufgerichtet, auf 2.048 px gebracht und nach sRGB umgewandelt, dann zugeschnitten, gefärbt, gestempelt und für den Bildschirm kodiert. Bei einem 20-MP-Foto aus einer Lumix GX9 dauerte das 274 bis 278 ms für ein JPEG und 440 bis 531 ms für ein RAW, jedes Mal, wenn ein Foto geöffnet wurde, das nicht unter den letzten acht war. Schnell genug für ein Foto, nicht aber, um durch eine Reise zu blättern.

### Standardvorschauen, wie Lightroom sie anlegt

Seit v0.11.0 ist die Arbeit, auf die man wartet, erledigt, bevor man fragt. Sobald die Vorschaubilder eines Ordners fertig sind, macht Fotollection im Hintergrund von jedem Foto eine Kopie mit 2.048 px, ein Foto pro Thread nach dem anderen, und legt sie neben den Vorschaubildern auf der Festplatte ab, als JPEG von etwa 0,66 MB (etwa 280 MB für die ganze Reise). Ein Klick auf irgendein Foto, nah oder fern, in dieser Sitzung oder der nächsten, beginnt mit dieser Kopie: 27 ms für ein JPEG wie für ein RAW. Übrig bleibt die eigene Arbeit der Vorschau (Zuschnitt, Größenänderung, Farbe, Datumsstempel und Kodierung), dieselben 27 ms wie aus dem Arbeitsspeicher. Eine Vorschau aus der Kopie auf der Festplatte weicht von einer frisch gemachten im Mittel um 1,35 von 255 Stufen ab, zu wenig, um es zu sehen.{% include fn.html keys="fn_lightroom" %}

### Speicher gut genutzt, nicht so wenig wie möglich

Ein Fotobetrachter, der auf einem Laptop mit 16 GB acht Fotos im Speicher hält, verschwendet den Laptop. Fotollection gibt der Vorschau jetzt ein Viertel des Arbeitsspeichers (auf einem Mac mit 16 GB etwa 190 Fotos mit 2.048 px, 20 RAW-Entwicklungen, die für ihre Regler bereitgehalten werden, und 600 MB für den Zoom), in Bytes gezählt und die ältesten zuerst freigegeben. Ist ein Foto auf dem Bildschirm, werden auch die nächsten zwei in der Richtung, in die du dich bewegst, und das eine dahinter geladen, so warten die Pfeiltasten nie. Die Obergrenze von 2 GB gilt für Exporte weiterhin.

### Das Foto auf dem Bildschirm geht vor

Es wird immer nur ein RAW gleichzeitig entwickelt, damit ein Stapel RAW-Fotos in den Speicher passt. Früher war die Reihenfolge der Warteschlange Glückssache: Eine Vorschau in Vorbereitung, die Größenschätzung des Exports oder ein Export konnten das angeklickte Foto warten lassen. Jetzt sagt jede Entwicklung, wer auf sie wartet, und Arbeit im Hintergrund tritt zur Seite, wann immer das Foto auf dem Bildschirm eine braucht. Dieselbe Arbeit brachte auch zwei seltene Hänger ans Licht, die jetzt beseitigt sind: einen, bei dem jeder Thread auf eine Entwicklung wartete, für die kein Thread mehr frei war, und einen, bei dem der Bildschirm auf ein Bild wartete, das er nie angefordert hatte.

| Version | JPEG p50 / p95 | RAW p50 / p95 | Wie |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Beim Klick dekodiert; RAW über macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Beim Klick dekodiert; RAW über unsere eigene Entwicklung |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | Aus der Standardvorschau auf der Festplatte oder aus dem Arbeitsspeicher |

Gemessen an der Prag-Reise des Autors (396 JPEG und 26 RAW, 20 MP) auf einem Apple M3 mit `measure_browse`: v0.9.0 aus seinem eigenen Tag gebaut, v0.10.0 auf seinem Code direkt vor dieser Änderung.
