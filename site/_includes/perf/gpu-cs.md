### Deset posuvníků za cenu jednoho

Každý barevný posuvník (jas, kontrast, teplota, sytost, vyblednutí, tónování stínů a světel, černobílá) se složí do jediné tabulky 33 × 33 × 33 barev, 35 937 bodů, které říkají, kam se každá barva posune. Použít ji na fotku znamená najít pro každý pixel jeho buňku v tabulce a interpolovat mezi čtyřmi jejími rohy (tetraedrická interpolace), takže deset posuvníků stojí tolik co jeden. Sestavení tabulky se všemi posuvníky nastavenými trvá asi 2,6 ms; její použití na náhled o 1 600 px méně než 2,2 ms; na export s 24 MP asi 24 ms.

### Proč se tažení pořád zdálo pomalé a co změnila grafická karta

Při měření krok po kroku stál jeden pohyb posuvníku asi 26 ms a samotná barva z toho byla jen 2 ms: zbytek zabralo kódování náhledu do JPEG (7 až 11 ms) a jeho opětovné dekódování v okně (9 až 15 ms). Grafická karta odstraní obojí. Když tažení začne, holý náhled a maska toho, kde je fotka (ne datum na fotce, rámeček ani okraj), jdou do grafické karty jednou; při každém kroku za nimi putuje už jen nová tabulka a malý program ve WebGL2 ji interpoluje přesně tak jako jádro: 1 ms na krok ve WebKitu na macOS, 2 ms v Chromiu. Když posuvník pustíš, nahradí ho přesný render jádra. Obojí se porovnalo na výrazném looku se všemi posuvníky posunutými: liší se v průměru o 0,13 ΔE2000 a o 0,77 na 99. percentilu, hluboko pod 1, kterou oko pozná při srovnání vedle sebe.

Žádný přepínač není potřeba. Při prvním tažení si to Fotollection ověří sám: pokud chybí WebGL2, renderer je softwarový nebo zkušební krok trvá 8 ms či déle, zůstane po celé sezení na cestě přes procesor. Softwarový renderer v testu potřeboval 16,5 ms na krok, takže zůstat tam na procesoru je správně.

### Exporty na všech jádrech

Dekódování a kódování fotky běží každé na jednom jádru, takže export je rychlý, když se zpracovává několik fotek najednou. Kolik jich to je, záleží na paměti: u fotek s 24 až 26 MP dávaly 2 najednou 4,9 fotky za sekundu, 4 dávaly 9,4, 6 dávalo 11,5 a 8 dávalo 12,5, přičemž paměť rostla z 0,49 na 1,21 GB. Fotollection volí počet podle paměťového rozpočtu 1,6 GB a velikosti největší fotky a zastaví se na 6, protože na fotkách ze skutečné cesty nebylo 8 najednou o nic rychlejší a spotřebovalo o 0,2 GB víc. Exporty grafickou kartu nikdy nepoužívají: tisk musí vyjít stejně na každém počítači a závisí na tom i testy, které exporty porovnávají bajt po bajtu.
