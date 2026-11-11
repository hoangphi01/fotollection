### Előtte: minden fotót a kattintáskor dekódolt

A v0.10.0-ig az előnézet akkor készült el, amikor kérted: a fájl beolvasása, a fotó dekódolása (a RAW kidolgozása), felállítása, 2048 px-esre méretezése és sRGB-be alakítása, majd a vágás, a színezés, a bélyegzés és a képernyőre kódolás. Egy Lumix GX9 20 MP-es fotóján ez JPEG-nél 274–278 ms, RAW-nál 440–531 ms volt, valahányszor olyan fotót nyitottál meg, amely nem volt az utolsó nyolc között. Egy fotóhoz elég gyors; egy út átfutásához nem.

### Standard előnézetek, ahogy a Lightroom tartja őket

A v0.11.0 óta a munka, amelyre a felhasználó vár, elkészül, mielőtt kérné. Amint egy mappa bélyegképei megvannak, a Fotollection a háttérben elkészíti minden fotó 2048 px-es másolatát, szálanként egyszerre egy fotót, és a lemezen, a bélyegképek mellett tartja, kb. 0,66 MB-os JPEG-ként (az egész útra kb. 280 MB). Egy kattintás bármelyik fotóra, közelire vagy távolira, ebben a munkamenetben vagy a következőben, ebből a másolatból indul: 27 ms JPEG-nél és RAW-nál egyaránt. Ami marad, az az előnézet saját munkája (vágás, átméretezés, szín, dátumbélyeg és kódolás), ugyanaz a 27 ms, mint a memóriából. A lemezes másolatból készült előnézet átlagosan 1,35 szinttel tér el (255-ből) egy frissen készülttől, ami túl kevés ahhoz, hogy látszódjon.{% include fn.html keys="fn_lightroom" %}

### Jól használt memória, nem a lehető legkevesebb

Egy fotónézegető, amely egy 16 GB-os laptopon nyolc fotót tart a memóriában, pazarolja a laptopot. A Fotollection most a gép memóriájának negyedét adja az előnézetnek (egy 16 GB-os Macen kb. 190 fotót 2048 px-en, 20 RAW-kidolgozást a csúszkáik kedvéért, és 600 MB-ot a nagyításhoz), bájtban számolva, és a legrégebbit szabadítja fel először. Amikor egy fotó a képernyőn van, a mozgásod irányában a következő kettő és a mögötte lévő egy is betöltődik, így a nyílbillentyűk sosem várnak. A 2 GB-os plafon az exportálásra továbbra is érvényes.

### A képernyőn lévő fotó az első

Egyszerre csak egy RAW kidolgozása fut, hogy egy köteg RAW-fotó beférjen a memóriába. Korábban a sor rendje a szerencsén múlt: egy épp készülő előnézet, az export méretbecslése vagy egy exportálás megvárakoztathatta a fotót, amelyre kattintottál. Most minden kidolgozás megmondja, ki vár rá, és a háttérmunka félreáll, valahányszor a képernyőn lévő fotónak szüksége van egyre. Ugyanez a munka két ritka elakadást is felszínre hozott, amelyek mára megszűntek: az egyikben minden szál egy olyan kidolgozásra várt, amelynek már nem maradt szála a futáshoz, a másikban a képernyő egy olyan képre várt, amelyet sosem kért.

| Verzió | JPEG p50 / p95 | RAW p50 / p95 | Hogyan |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Kattintáskor dekódolva; a RAW a macOS-en át |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Kattintáskor dekódolva; a RAW a saját kidolgozásunkkal |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | A lemezen lévő standard előnézetből vagy a memóriából |

Mérve a tulajdonos prágai útján (396 JPEG és 26 RAW, 20 MP) egy Apple M3-on a `measure_browse` segítségével: a v0.9.0 a saját címkéjéből építve, a v0.10.0 a kódján közvetlenül e változtatás előtt.
