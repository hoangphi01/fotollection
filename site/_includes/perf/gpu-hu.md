### Tíz csúszka egynek az áráért

Minden színcsúszka (fényerő, kontraszt, melegség, telítettség, fakítás, színezett árnyékok és csúcsfények, fekete-fehér) egyetlen 33 × 33 × 33 színből álló táblába sűrűsödik: 35 937 pontba, amelyek megmondják, hová kerül az egyes szín. Ha ezt egy fotóra alkalmazzuk, minden pixelnek megkeressük a celláját a táblában, és a cella négy sarka között interpolálunk (tetraéderes interpoláció), így tíz csúszka annyiba kerül, mint egy. A tábla felépítése minden csúszkával beállítva kb. 2,6 ms; alkalmazása egy 1600 px-es előnézetre 2,2 ms-nál kevesebb, egy 24 MP-es exportra kb. 24 ms.

### Miért tűnt még lassúnak a húzás, és mit változtatott a videokártya

Lépésenként mérve egy csúszkamozdulat kb. 26 ms-ba került, és ebből maga a szín csak 2 ms volt: a többi az előnézet JPEG-be kódolása (7–11 ms) és az ablakban való újbóli dekódolása (9–15 ms). A videokártya mindkettőt megszünteti. Amikor egy húzás elkezdődik, a sima előnézet és egy maszk arról, hol van a fotó (nem a dátumbélyeg, a keret vagy a szegély), egyszer kerül át a videokártyára; minden lépésnél csak az új tábla követi, és egy kis WebGL2-program interpolálja, pontosan úgy, ahogy a mag: lépésenként 1 ms a macOS WebKitjében, 2 ms Chromiumban. Amikor elengeded, a mag pontos renderje lép a helyére. A kettőt egy erős lookon hasonlítottuk össze, minden csúszkát elmozdítva: átlagosan 0,13-mal térnek el ΔE2000-ben, a 99. percentilisen 0,77-tel, messze az 1 alatt, amelyet a szem egymás mellett még észrevesz.

Semmilyen kapcsolóra nincs szükség. Az első húzáskor a Fotollection maga ellenőrzi: ha nincs WebGL2, ha szoftveres a renderelő, vagy ha egy próbalépés 8 ms-ig vagy tovább tart, a munkamenet végéig a processzor útján marad. Egy szoftveres renderelő a tesztben lépésenként 16,5 ms-ot igényelt, így ott helyes a processzornál maradni.

### Exportálás minden magon

Egy fotó dekódolása és kódolása egy-egy magon fut, így az exportálás akkor gyors, ha egyszerre több fotón dolgozik. Hogy hányon, az a memórián múlik: 24–26 MP-es fotókon 2 egyszerre másodpercenként 4,9 fotót adott, 4 9,4-et, 6 11,5-öt, 8 pedig 12,5-öt, miközben a memória 0,49-ről 1,21 GB-ra nőtt. A Fotollection a számot egy 1,6 GB-os memóriakeretből és a legnagyobb fotó méretéből választja, és 6-nál megáll, mert egy valódi út fotóin 8 egyszerre nem volt gyorsabb, és 0,2 GB-tal több memóriát használt. Az exportálás sosem használja a videokártyát: egy nyomatnak minden gépen ugyanolyannak kell lennie, és az exportokat bájtról bájtra összevető tesztek erre épülnek.
