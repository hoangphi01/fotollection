### Miért tűnnek laposnak a webes szerkesztőből készült nyomatok

A teszt 20 fényképezőgépes fotót és 5 Display P3-as változatot (a friss telefonok és Macek szélesebb színterét) exportál a Fotollection valódi exportján keresztül, majd mindkét fájlt egy független dekóderrel olvassa be, a színprofiljukon át CIELAB-ba alakítja, és pixelről pixelre összeveti az eredetivel. Egy tipikus webes szerkesztő exportja ugyanezen az összevetésen megy át: a profil elhagyva, 2000 px-re kicsinyítve, majd a nyomtatáshoz visszanagyítva, 80-as minőséggel és felezett színnel (4:2:0) mentve.

| Medián | Fotollection, sRGB | Webes szerkesztő | Fotollection, Display P3 | Webes szerkesztő |
|---|---|---|---|---|
| ΔE2000 (színhiba, 1 alatt láthatatlan) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Megmaradt telítettség | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Megmaradt finom részletek | 99,8 % | 13 % | 100 % | 18 % |

Két ok magyarázza a lapos hatást. A 2000 px-re kicsinyítés és a labornak való visszanagyítás eldobja a finom részletek 87 %-át, amelyektől egy nyomat élesnek látszik. A színprofil elhagyása pedig egy Display P3-as fotót úgy olvas be, mintha sRGB volna, ami elveszi a telítettsége 13 %-át. A Fotollection megtartja a profilt (25 fájlból 25-nél), a fotó teljes felbontásán, teljes színnel (4:4:4) exportál 95-ös minőséggel (akkor így mértük; a v0.13.0 óta alapból 90, ami még mindig a látható különbség alatt van), és a nagyobb hibái csak az éles széleken vannak, ahol minden JPEG a legkeményebben dolgozik. Még hátravan: valódi iPhone HEIC-fájlok, és spektrofotométerrel mért nyomatok.
