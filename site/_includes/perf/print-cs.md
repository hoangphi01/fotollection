### Proč tisky z webového editoru vypadají ploše

Test exportuje 20 fotek z fotoaparátu a 5 verzí v Display P3 (širší barvy novějších telefonů a Maců) skutečným exportem Fotollection, pak oba soubory načte nezávislým dekodérem, převede je přes jejich barevné profily do CIELAB a porovná pixel po pixelu s originálem. Stejným porovnáním projde i typický export z webového editoru: profil zahozený, zmenšeno na 2 000 px a pro tisk zase zvětšeno, uloženo v kvalitě 80 s polovičním rozlišením barev (4:2:0).

| Medián | Fotollection, sRGB | Webový editor | Fotollection, Display P3 | Webový editor |
|---|---|---|---|---|
| ΔE2000 (barevná chyba, pod 1 je neviditelná) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Zachovaná sytost | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Zachované jemné detaily | 99,8 % | 13 % | 100 % | 18 % |

Plochý vzhled vysvětlují dvě příčiny. Zmenšení na 2 000 px a nové zvětšení pro fotolab zahodí 87 % jemných detailů, díky kterým tisk působí ostře. A zahození barevného profilu přečte fotku v Display P3, jako by byla v sRGB, což jí vezme 13 % sytosti. Fotollection profil zachová (25 z 25 souborů), exportuje v plném rozlišení fotky s plnými barvami (4:4:4) v kvalitě 95 (tehdy měřeno; od v0.13.0 je výchozí 90, stále pod viditelným rozdílem) a jeho větší chyby leží jen na ostrých hranách, kde se každý JPEG nadře nejvíc. Ještě přijde: skutečné soubory HEIC z iPhonu a tisky změřené spektrofotometrem.
