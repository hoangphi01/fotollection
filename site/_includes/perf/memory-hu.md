### Miért nem nő a memória a mappával

A Fotollection fotónként néhány bájtot tart a könyvtárában (5035 fotóra 13 MB), a bélyegképeket nem a memóriában tartja, hanem egy lemezes gyorsítótárba írja, korlátozott számú előnézetet tart a memóriában, és egyszerre csak annyi teljes méretű fotón dolgozik, amennyi belefér a keretébe. A képernyőn lévő rács virtuális: csak a látható csempék léteznek. A teszthez ugyanazt a 95 fotót 53-szor linkeltük be egy 5035 fájlos mappába:

| | 95 fotó | 5035 fotó |
|---|---|---|
| Importálás és EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| 400 px-es bélyegképek a lemezes gyorsítótárba | 47,8 fotó/s | 46,4 fotó/s |
| Exportálás, 6 egyszerre | 18,0 fotó/s | 15,4 fotó/s, 0 sikertelen |
| A teljes futás csúcsmemóriája | 1,35 GB | 1,45 GB |

53-szor több fotó 0,1 GB-tal több memóriába került. A hosszú exportálás fotónként csak azért kicsit lassabb, mert több mint öt percig futott, és egy laptop processzora tartós terhelés alatt lelassul.
