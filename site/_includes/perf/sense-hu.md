### Terheléses teszt, nem átlagos nap

A legtöbben egyszerre egy utazást rendeznek, 2000–3000 fotót. Hogy kiderüljön, a Fotollection ennél sokkal többnél is stabil marad-e, a RollSense egyszerre 10 044 fotót kapott egy átlagos, 2020-as windowsos laptopon (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): fényképezős JPEG-eket, telefonos fotókat, képernyőképeket, valamint Canon és Sony RAW-fájlokat. 70 perc alatt 9639-et olvasott be minden magon, a processzor végig dolgozott, a memória 400 és 850 MB között maradt, és nem nőtt.

### Hová megy az idő

Minden fotót egyszer dekódol, lekicsinyít, és a tartalomfelismerő modell beolvassa. Maga a modell ezen a laptopon fotónként 128 ms, az egész lépés 431 ms, vagyis az idő kb. 70 %-a magának a fotónak a beolvasása és dekódolása, főleg teljes méretű JPEG és RAW fájloké. A videokártya pihent, legfeljebb 1,5 %: mindez Rustban, a processzoron fut.

### Több szál és a laptop korlátja

Két szállal 614 ms egy fotó, néggyel 498 ms, nyolccal 438 ms. Egy vékony laptop 15 wattot oszt el a magjai között, így ha mind a nyolc szál dolgozik, mindegyik lassabb (átlagosan 1,8 GHz, nyugalomban 3,4 GHz), és a nyolc szálból négy egy magon osztozik a másik néggyel.

| | Ez a laptop (mért) |
|---|---|
| Egy 2500 fotós utazás | 18 perc |
| Egy telefon 15 000 fotós tára | 108 perc |
| Egy fotó újra a gyorsítótárból | 1,3 ms |

Mérve 2026. 11. 20-án a `sort_pass_bench` eszközzel, másodpercenkénti adatokkal.
