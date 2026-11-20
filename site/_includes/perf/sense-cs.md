### Zátěžový test, ne běžný den

Většina lidí třídí jeden výlet naráz, 2 000 až 3 000 fotek. Aby se ukázalo, zda Fotollection zůstane stabilní i daleko za tím, dostal RollSense najednou 10 044 fotek na běžném notebooku s Windows z roku 2020 (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): JPEG z fotoaparátu, fotky z telefonu, snímky obrazovky a soubory RAW z Canonu a Sony. Přečetl jich 9 639 za 70 minut na všech jádrech, procesor byl celou dobu vytížený a paměť se držela mezi 400 a 850 MB, aniž by rostla.

### Kam jde čas

Každá fotka se jednou dekóduje, zmenší a přečte ji model obsahu. Samotný model zabere na tomto notebooku 128 ms na fotku a celý krok 431 ms, takže asi 70 % času je čtení a dekódování samotné fotky, většinou JPEG a RAW v plném rozlišení. Grafická karta zůstala nečinná, nejvýše 1,5 %: vše běží v Rustu na procesoru.

### Víc vláken a limit notebooku

Dvě vlákna potřebují 614 ms na fotku, čtyři 498 ms a osm 438 ms. Tenký notebook dělí 15 wattů mezi jádra, takže když jich běží všech osm, každé je pomalejší (průměrně 1,8 GHz proti 3,4 GHz v klidu), a čtyři z osmi vláken sdílejí jádro s ostatními čtyřmi.

| | Tento notebook (měřeno) |
|---|---|
| Výlet o 2 500 fotkách | 18 min |
| Galerie telefonu s 15 000 fotkami | 108 min |
| Fotka znovu z mezipaměti | 1,3 ms |

Měřeno 20. 11. 2026 nástrojem `sort_pass_bench`, data každou sekundu.
