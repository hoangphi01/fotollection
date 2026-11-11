### Co Facebook s fotkou udělá a co uděláme my předem

Facebook zmenší každou fotku nejvýš na 2 048 px, znovu ji zkomprimuje zhruba v kvalitě 71 s polovičním rozlišením barev a barevné profily ignoruje. Export Share tuhle práci udělá pořádně ještě před nahráním, takže Facebook už nemá co zmenšovat: fotka se přes svůj profil převede do sRGB, zmenší Lanczosovým filtrem v lineárním světle, lehce doostří a uloží v kvalitě 88. Každé nastavení bylo zvoleno měřením: silnější doostření (0,35 místo 0,2) dopadlo hůř a kvalita 84 nebo 80 ztratila víc, než ušetřila.

| Medián | Original size | Facebook, 2 048 px | Instagram, 1 080 px |
|---|---|---|---|
| Velikost souboru (JPEG z fotoaparátu o 3,08 MB) | 1,42 MB | 0,51 MB | 0,24 MB |
| SSIM proti ideálnímu zmenšení (1 je totožné) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

I po vlastní rekompresi Facebooku zůstane výsledek blíž fotce než obvyklým postupem (SSIM 0,965 proti 0,961) a fotka v Display P3 si uchová 99,6 % barev místo 86 %, protože její profil byl převeden na skutečné barvy sRGB, a ne zahozen. Presety se zmenšením zůstávají nad ΔE 1 kvůli polovičnímu rozlišení barev, které každá platforma stejně použije; *Original size* zachová každý pixel zhruba v polovině velikosti souboru, pod ΔE 1.
