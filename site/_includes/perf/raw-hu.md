### Mit rögzít valójában egy érzékelő

A fényképezőgép érzékelője apró kutak, úgynevezett fotodiódák (photosite-ok) millióiból álló rács. Amíg a zár nyitva van, mindegyik összegyűjti a rá eső fényt, és elektromos töltéssé alakítja; amikor bezárul, egy átalakító minden töltésből egy számot csinál. Az összefüggés **lineáris**: kétszer annyi fény kétszer akkora számot ad. A szemünk egyáltalán nem lineáris, a fényerőt arányokban ítéli meg, ezért néz ki egy RAW fájl sötétnek és laposnak, amíg ki nem dolgozzák.

Kétféle érzékelő olvassa ki ezt a rácsot:

- A **CCD** (charge-coupled device, töltéscsatolt eszköz) minden fotodióda töltését soronként a chip szélén lévő egyetlen erősítőhöz tolja. Ez volt az első digitális gépek és sok korai középformátumú hátfal érzékelője: tiszta képek, de lassú kiolvasás és nagy energiaigény.
- A **CMOS** minden fotodiódának saját erősítőt ad, és egyszerre sokat olvas ki közülük. Gyorsabb, kevesebb energiát fogyaszt és olcsóbb gyártani, így ma szinte minden fényképezőgépben és telefonban ez van. A legtöbb CMOS-érzékelő egymás után olvassa ki a sorait, ezért dőlhet meg egy gyorsan mozgó téma az elektronikus zárral készült fotón (ez a *rolling shutter*), és ezért dőlnek kevésbé a sokkal gyorsabban kiolvasó, rétegzett (stacked) érzékelők.

A számoknak sokkal több lépcsőjük van, mint egy JPEG-nek. Egy fényképezőgép fotodiódánként 12 vagy 14 bitet tárol, 4096 vagy 16 384 szintet, ahol egy JPEG 8 bitet, 256 szintet tart meg. Ezek a plusz szintek teszik lehetővé, hogy sávosodás nélkül emelj meg egy árnyékot vagy húzz vissza egy világos eget. És a fény hiánya sem nulla: minden érzékelő hozzáad egy kis eltolást, a **feketeszintet**, hogy a fekete körüli zaj ne vágódjon le. A Nikon D750 mintájában a fekete 600, a legnagyobb érték pedig, amelyet az érzékelő tárolni tud, a **fehérszint**, 15 520. Bármely RAW-kidolgozó legelső dolga, hogy ezt a két számot 0-ra és 1-re hozza, a színszűrő minden pozíciójára külön, mert egyes gépek mindegyiknek saját feketeszintet adnak.

### A szín az egyes fotodiódák fölötti szűrőből jön

Egy fotodióda csak azt méri, mennyi fény érkezik, a színét soha. Hogy színt lásson, az érzékelőt egy **színszűrő-tömb** fedi: minden fotodióda fölött egy apró piros, zöld vagy kék szűrő, így mindegyik csak a színkép egy részét számolja.

- A **Bayer**-minta, amelyet Bryce Bayer szabadalmaztatott a Kodaknál 1976-ban, egy 2 × 2-es négyzetet ismétel: egy piros, két zöld, egy kék. A fotodiódák fele azért zöld, mert a szemünk a részletek érzékelésének nagy részét a színkép közepéből veszi, így a zöld hordozza az élesség nagy részét. Szinte minden fényképezőgép ezt a mintát használja, négy irány egyikében (RGGB, BGGR, GRBG, GBRG), amelyet a fájlnak meg kell adnia.
- Az **X-Trans**, a Fujifilm 2012 óta használt mintája, egy 6 × 6-os négyzetet ismétel, minden sorban és minden oszlopban pirossal, zölddel és kékkel. Szabálytalan elrendezése miatt a finom, ismétlődő részleteken kevésbé valószínű a színes moaré, így a Fujifilm elhagyhatja azt a lágyító szűrőt, amelyet sok érzékelő hordoz, egy nehezebb rekonstrukció árán.
- **Más megoldások:** a monokróm érzékelőkön egyáltalán nincs szűrő, így minden fotodióda a teljes részletet látja (a Leica Monochrom gépei); a Sigma Foveonja három réteget rak egymásra, amelyek a szilícium különböző mélységeiben nyelik el a színeket; sok telefon négy azonos színű fotodiódát csoportosít (*Quad Bayer*), és gyenge fényben egyként olvassa ki őket.

Egy 24 MP-es Bayer-érzékelő tehát 24 millió számot rögzít, mindegyiket egyetlen színre: minden pixel három színéből kettő hiányzik. Ebből a mozaikból képet csinálni a demozaikolás dolga, lásd lent.

### Mit tartalmaz egy RAW fájl

A JPEG kész kép: a gép már minden döntést meghozott. A RAW fájl az érzékelő számait e döntések előttről őrzi meg, mindazzal együtt, ami ahhoz kell, hogy később meghozzuk őket:

- magukat a mintákat, fotodiódánként egyet, és hogy melyik színszűrő alatt ült mindegyik;
- a fekete- és a fehérszintet;
- az **aktív területet** (azokat a fotodiódákat, amelyek fényt kaptak; a szélein néhány sor és oszlop szándékosan le van takarva, hogy a feketét mérjék) és a gép **alapértelmezett vágását**;
- a **fehéregyensúlyt**, amelyet a gép a fotó készítésekor választott;
- a **színmátrixokat**, amelyek megmondják, hogyan viszonyul ennek az érzékelőnek a saját pirosa, zöldje és kékje az ember által látott színekhez;
- egy **előnézetet**, egy JPEG-et, amelyet maga a gép készített a saját színével és tónusával, plusz az EXIF-adatokat.

Minden gyártónak saját formátuma van, többnyire TIFF-alapú és részben dokumentálatlan: CR2 és CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). A **DNG** az a formátum, amelyet az Adobe nyilvánosan közzétesz (1.7-es verzió), és amelyet a Ricoh, a Leica, a Pentax és sok telefon használ; ennek a szabályait követi ez az oldal.

A Fotollection a DNG-t a saját, a specifikációból megírt olvasójával olvassa, a többi formátumot pedig a rawler bontja ki, egy LGPL-es könyvtár, amely a Fotollectionbe van építve. A rács bélyegképei egyáltalán nem nyúlnak az érzékelő adataihoz: a fájlban lévő előnézetből jönnek, 9–32 ms alatt, ezért nyílik meg egy RAW-mappa olyan gyorsan, mint egy JPEG-mappa.

### Demozaikolás: a két hiányzó szín kiszámítása

Minden pixel két hiányzó színének újraépítését a szomszédaiból **demozaikolásnak** hívják, és minden más lépésnél jobban eldönti, mennyire éles és tiszta egy RAW-fotó. Minden módszer ugyanazzal a dilemmával néz szembe: a szomszédok átlagolása sík területeken biztonságos, de egy élen át két különböző felületet kever össze, így a művészet abban rejlik, hogy tudjuk, mely szomszédok tartoznak össze.

- A **bilineáris** módszer egyszerűen átlagolja az egyes színek legközelebbi fotodiódáit. Ez az alap, amelyhez minden módszert mérnek: lágy, és minden éles él mentén színes szegélyeket hagy, mert a pirosat, a zöldet és a kéket kissé eltérő helyeken átlagolja.
- A **Malvar–He–Cutler** (2004) ebből az átlagból indul, és rögzített 5 × 5-ös szűrőkkel korrigálja annak a színnek a részleteivel, amelyet a fotodióda látott. Élesebb, és még mindig nagyon gyors. A Fotollection a v0.10.0-ig ezzel dolgozott ki.
- A **PPG** (patterned pixel grouping) először eldönti, melyik irányban változik a legkevésbé a kép, és csak abban az irányban interpolál. A rawler ezt használja, és a v0.10.0-ig a legtöbb formátumnál a Fotollection Windowsos és Linuxos változata is.
- Az **RCD** (ratio corrected demosaicing, Luis Sanz Rodrígueztől) a színek közötti arányokkal dolgozik, nem a különbségekkel, ami világos és sötét területeken egyaránt jobban tart, és arról ismert, hogy finom részleteken kevés hibát hagy; a darktable és a RawTherapee is kínálja.
- A **Markesteijn**, Frank Markesteijn után, az X-Transra készült módszer, egy menetben vagy, lassabban és tisztábban, háromban.
- **A Fotollection demozaikolása a v0.10.0 óta** az RCD-től veszi az irány eldöntésének módját, majd a zöldet mindkét oldalról becsüli meg, az adott pont saját színével korrigálva (ahogy Hamilton és Adams tették), aszerint súlyozva, mennyire változik az egyes oldal. Először az RCD saját arány-lépését írtuk meg, és az rosszabbul teljesített a tesztábráinkon, így a nyertes változat az, amelyik kiadásra került, a saját nevén.

Ha egy módszer rosszul tippel, az négy jellegzetes módon látszik:

- **cipzár:** váltakozó világos és sötét pixelek sora egy éles él mentén, ahol a módszer mindkét oldalról vett szomszédokat;
- **labirintus:** finom, féregszerű minta sík területeken, amikor egy Bayer-négyzet két zöldje nem egészen ugyanazt olvassa (eltérő szomszédú sorokban ülnek, és sok érzékelőn egy-két százalékkal eltérnek);
- **álszín:** színes pöttyök vagy szegélyek valójában szürke részleteken, például szöveten, hajon, téglafalon vagy egy távoli kerítésen;
- **moaré:** színes hullámok nagyon finom, ismétlődő mintákon, amelyek finomabbak annál, amit a színszűrő mintavételezni tud.

**Hogyan ítélünk meg egyet.** A szemünket könnyen becsapja egy élesebbnek tűnő eredmény, ezért minden módszert olyan képeken pontozunk, amelyeknek pontosan ismert a valósága. Egy számítógép öt tesztábrát rajzol 768 px-en: egy zónalemezt (a széle felé finomodó gyűrűk, egészen a legfinomabb részletig, amelyet a zöld fotodiódák meg tudnak tartani), egy 72 küllős Siemens-csillagot, telített színsávokat szürkén három irányban, sima színátmeneteket, és egy sík szürkét, amelyben a két zöld 2 %-kal eltér. Mindegyiket átalakítjuk arra, amit egy Bayer-érzékelő rögzítene, demozaikoljuk, és összevetjük az eredetivel. A PSNR decibelben méri a teljes hibát (a több a jobb; minden 6 dB felezi); az álszín a szürke ábrákon megjelenő átlagos szín; az élhiba az él 2 px-es körzetén belüli átlagos hiba.

| Ábra (768 px) | Bilineáris | Malvar (a miénk a v0.10.0-ig) | A miénk a v0.10.0 óta |
|---|---|---|---|
| Zónalemez: PSNR, a több a jobb | 13,0 dB | 17,3 dB | 16,5 dB |
| Zónalemez: álszín, a kevesebb a jobb | 65,9 | 43,0 | 18,7 |
| Siemens-csillag: PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Siemens-csillag: álszín | 9,4 | 8,4 | 0,16 |
| Telített színsávok: hiba az élek mentén, a kevesebb a jobb | 11,6 | 21,6 | 10,5 |
| Sima átmenetek: PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Sík szürke zöld eltéréssel: labirintus, a kevesebb a jobb | 0,38 | 0,38 | 0 |
| Textúra beragadt pixelekkel: ennyi marad 10 000-enként, a kevesebb a jobb | 35,8 | 66,5 | 0 |

A Malvar kb. 4 dB-lel jobb a bilineárisnál a finom szürke részleteken, ahogy a cikke ígéri, de rosszabb az erős színek közötti éleken, mert a korrekciója az egyik szín részleteit kölcsönzi a többinek, és feltételezi, hogy együtt változnak, amit egy szürke falon lévő piros tábla nem tesz. Egyik sem tünteti el a labirintust, amely nem demozaikolási probléma, hanem a két zöld közötti különbség, amelyet előtte kell kiegyenlíteni. Ezeket a hiányokat zárja be a láncunk a v0.10.0 óta: a beragadt pixeleket és a zöld egyensúlyhiányát akkor javítjuk, amikor még minden pont csak azt tartalmazza, amit mért, az irányfüggő demozaikolás újraépíti a színeket, és egy kis medián a színkülönbségeken eltünteti a maradék álszínt. A tesztábrákon mindenhol nyer, kivéve a zónalemez teljes PSNR-jét, amely a legfinomabb részletek közelében 0,8 dB-lel alacsonyabb a Malvarénál, ahol minden módszer aliasol; az álszíne még ott is a felére esik. A kilenc valódi fájlon minden Bayer-gép kidolgozását ugyanolyan közel vagy közelebb hozza a gép saját JPEG-jéhez, és 24 MP-en kb. 180 ms-ba kerül, szemben a Malvar 40 ms-ával: fotónként néhány tized másodperccel több, tisztább részletű nyomatokért.

### A gép színeitől a tieidig

A gép pirosa, zöldje és kékje nem azonos a képernyőd színeivel: minden érzékelő szűrői saját festékeket használnak, és másképp fedik át egymást, mint a szemünk csapjai. A fájlban lévő színmátrixok a gép színeit **CIE XYZ**-be alakítják, az emberi színlátás 1931-ben meghatározott szabványos modelljébe, onnan pedig **sRGB**-be, a legtöbb képernyő és a fotólaborok színterébe.

- Először a **fehéregyensúly** jön: egy erősítés a piroson és a kéken, hogy ami a jelenetben fehér, semlegesnek tűnjön. A gép rögzíti az általa választott erősítéseket, és a Fotollection ezeket a demozaikolás előtt alkalmazza, így a fenti módszerek által használt színkülönbségek kicsik maradnak.
- Egy DNG általában **két mátrixot** hordoz, az egyiket meleg izzólámpafényben, a másikat nappali fényben mérték. A fotódhoz illő a kettő között van: a kidolgozó a fehéregyensúlyból színhőmérsékletként kiszámítja a felvétel fényét, és összekeveri a kettőt. A Fotollection ugyanezt teszi a többi formátumnál, azokból a mátrixokból, amelyeket a rawler minden gépmodellhez ismer.
- Mindez **lineáris fényben** történik, az érzékelő saját skáláján, bármilyen görbe előtt. A színek keverése, skálázása és átméretezése lineáris fényben ad fizikailag helyes eredményt; ha a görbe után történik, az élek sötétebbek lesznek, a színek pedig elcsúsznak.

**A gép saját lookja.** Egy gép saját JPEG-je nem semleges: minden gyártó hozzáadja a saját kontrasztját, színeit és bőrtónusait, és ez a look része annak, amiért az emberek egy márkát választanak. A v0.10.0 óta egy RAW-fotó **semleges** megjelenítéssel nyílik meg, hűen az érzékelő mátrixához, az objektív vignettálásával és a gép által választott expozícióval; a **Like the camera's JPEG** egy kattintásra van egy RAW-fotó *RAW colour* kártyáján, egy fotóra vagy egyszerre minden RAW-ra. (A v0.10.0-ig minden RAW-ot a gép JPEG-jéhez igazítottunk; az akkor mentett projekt megtartja ezt a lookot.) Ez a választás kétféleképpen igazít a gép JPEG-jéhez, egy görbével és egy mátrixszal vagy csatornánként egy görbével, és azt tartja meg, amelyik közelebb jut az adott fájlnál: a Nikon D750-en felezi a különbséget, ΔE2000-ben 2,26-ról 1,16-ra. A rács bélyegképei a gép saját előnézetét tartják meg, ahogy a Lightroomban.

### Tónus: lineáris számokból kép

A lineáris számok sötétnek és fakónak tűnnek a képernyőn, mert a fény megduplázása, amelyet az érzékelő kétszer akkora számként rögzít, nekünk csak egy fokozatnyi világosodásnak látszik. Egy **tónusgörbe** alakítja az érzékelő skáláját képpé: elhelyezi a középszürkét, megemeli az árnyékokat, a csúcsfényeket lágyan lekerekíti ahelyett, hogy egyszerre levágná őket, és megadja a kép kontrasztját. A képet ezután az sRGB-görbével (a „gammájával”) tároljuk, amely egy JPEG 256 szintjéből többet szán az árnyékokra, ahol a szemünk a lépcsőket a legjobban észreveszi.

Két korrekció, amelyet a gép a JPEG-jében elvégez, nincs benne a RAW-adatokban, és mindkettő számít a nyomatoknál:

- **Vignettálás:** minden objektív sötétebb a sarkok felé. A Fotollection a gép saját előnézetéből méri meg, mennyire világosította ki őket a gép, sugárirányú erősítésként. Amikor a DNG-kidolgozásunkat először vetettük össze a gép JPEG-jével, a közép kb. 4 L\*-gal volt túl világos, a sarkok pedig 1,5-del túl sötétek; a sugárirányú erősítés a különbséget ΔE 3,8-ról 2,67-re vitte, a fotó minden részét ±0,5 L\*-on belül tartva.
- **Torzítás:** az objektív enyhe hordó- vagy párnaformája. A Micro Four Thirds gépeket (Panasonic, Olympus) úgy tervezték, hogy a gép korrigálja, és a fájljaik rögzítik, hogyan. Macen ezeket a fájlokat a macOS dolgozza ki, amely alkalmazza a korrekciót; a saját kidolgozásunkban való korrigálása tervben van (FR-085).

### Hogyan dolgoz ki a Fotollection egy RAW-ot, lépésről lépésre

Teljes kidolgozásnál, amely a nyomtatási előnézet, a nagyítás és az exportálás mögött áll:

1. **Felismerés:** a formátumot a fájl első bájtjaiból ismeri fel, sosem a nevéből, így egy átnevezett fájl vagy egy sima TIFF-nek látszó RAW is helyesen olvasódik be.
2. **Beolvasás:** az érzékelő számai, a DNG a saját olvasónkkal, a többi formátum a rawler kibontásával. A túl nagy vagy sérült fájlt elutasítja, mielőtt egyetlen pixelt dekódolna, egy hibakóddal.
3. **Szintek:** a fekete 0-ra, a fehér 1-re, a színszűrő minden pozíciójára, az aktív terület sarkától számolva.
4. **Fehéregyensúly**, majd **demozaikolás**: először a beragadt pixelek és a zöld egyensúlya, aztán az irányfüggő demozaikolás, végül az álszín-medián (az X-Trans Markesteijnje a v0.15.0-ban jön).
5. **Szín:** a gép mátrixai, a felvétel fényéhez keverve, lineáris fényben.
6. **Tónus és look**, a gép előnézetéhez igazítva, a vignettálásával együtt.
7. Innentől egy RAW-fotó olyan, mint bármely másik: vágás, dátumbélyeg, keret, nyomtatási méret és színprofil, teljes felbontásban, 4:4:4-ben, 95-ös minőséggel exportálva.

Néhány szabály végig érvényes. Egyszerre csak egy kidolgozás fut, minden processzormagra szétosztva, és sosem az ablak saját szálán, így a Fotollection akkor is válaszol, amikor egy 30 MP-es fotó kidolgozása zajlik. Az exportálás mindig a processzoron fut, így egy nyomat minden gépen ugyanolyan lesz; a videokártya csak azt gyorsítja, amit szerkesztés közben látsz. És a kód a sajátunk, a közzétett specifikációkból és cikkekből írva, nem más projektekből másolva, majd ellenőrizve: a fenti szintetikus ábrákon, kilenc gép kilenc RAW fájlján (CC0, a raw.pixls.us oldalról), és fájlok szándékos megcsonkításával és elrontásával, hogy egy rossz fájlból hiba legyen, sosem összeomlás.

**A v0.10.0 óta: saját kidolgozás minden Bayer-gépre.** A rawler csak kibontja a fájlokat, a többi a Fotollectioné. A kilenc mintafájl teljes kidolgozása, 3 futás mediánja, ugyanazon a laptopon: a mi kidolgozásunk a tisztább demozaikolásával, szemben a rawler egyszerűbb megoldásával és a macOS-ével:

| Gép (formátum) | A Fotollection sajátja | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | következik | 524 ms | 1339 ms |

A miénk egy tized–negyed másodperccel tovább tart a rawlerénél a tisztább demozaikolás és az igazított szín miatt, és a legtöbb gépen gyorsabb marad a macOS-nél; fotónként egyszer történik meg, utána egy csúszka 4–11 ms alatt dolgozza újra a megtartott másolatot. Macen a Panasonic és az Olympus fájljai a macOS-nél maradnak az objektívkorrekció miatt, amíg a saját kidolgozásunk nem korrigálja a torzítást. Az S5 szürke foltjai semlegesebbek lesznek, mint a rawleren át (1,42-es színeltolódás a 2,43-mal szemben, ahol a 0 a tökéletes szürke). A Fujifilm X-Trans a Markesteijnnel következik, utána pedig az előnézet kidolgozása a videokártyán, a processzor eredményével mint referenciával.
