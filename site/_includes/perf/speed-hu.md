### Mi történik, amikor megnyitsz egy mappát

Egy fotómappát a Fotollection nem olvas be egészben. Minden fájlnál az EXIF-adatait (dátum, gép, hely) és azt a kis bélyegképet olvassa be, amelyet minden fényképezőgép a fájlba ment, mást nem: 1000 fotó 0,12 s, szemben azzal az 1 s-os kerettel, amelyet a kód megírása előtt szabtunk meg. Ezután a háttérben élesebb, 400 px-es bélyegképek készülnek, két szálon, hogy a Fotollection gyors maradjon, és egy lemezes gyorsítótárba kerülnek, így egy újra megnyitott mappa azonnal ott van. 5035 fotónál az importálás 383 ms-ig, a beágyazott bélyegképek 236 ms-ig tartottak.

### Hová megy egy előnézet 17 ms-a

Egy beállítás módosítása az előnézetet a fotó memóriában tartott 2048 px-es másolatából rajzolja újra, sosem a teljes fájlból. Az ehhez szükséges 16,7 ms-ból a vágás és a Lanczos-szűrős átméretezés 9,7 ms, a dátumbélyeg 0,1 ms alatt, az előnézet JPEG-be kódolása a képernyőhöz pedig 7,0 ms. Ezt a kódolót méréssel hangoltuk: az export beállításaival (teljes színfelbontás és optimalizált táblák) 16,2 ms-ig tartott; a képernyőnek ezekre nincs szüksége, ezért az előnézet felezett színt és sima táblákat használ, az exportok viszont teljes színnel (4:4:4) készülnek. Kipróbáltunk egy olcsóbb átméretező szűrőt is, de ezen a processzoron lassabbnak bizonyult (12,5 ms), így maradt a Lanczos.

### Natív kód egy böngészővel szemben

Ugyanazt a feladatot (dekódolás, 2:3-ra vágás, dátumbélyeg, JPEG 95-ös minőséggel) ugyanazon a 95 fotón futtattuk le a Fotollectionben és Chromiumban, magonként egy workerrel. Az exportálás sebessége ugyanannyi lett, 18,0 a 17,9 fotóval szemben másodpercenként, ami megcáfolta az első feltevésünket: a böngésző nem lassú. A natív kód melletti érvek máshol vannak: a Fotollection teljes színfelbontású 4:4:4-es fájlokat ír, ahol a böngésző kódolója felezi a színt, megtartja minden fotó színprofilját, a fájlokat közvetlenül a lemezre írja, és negyedével kevesebb memóriát használt (1,35 GB az 1,78 GB-tal szemben). Azóta a nyomtatási kódoló még gyorsabb lett: 16 MP-en másodpercenként 23,5 fotó.
