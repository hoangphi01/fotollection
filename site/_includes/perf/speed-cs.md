### Co se stane, když otevřeš složku

Složka fotek se nečte celá. Z každého souboru Fotollection přečte jeho data EXIF (datum, fotoaparát, místo) a malou miniaturu, kterou každý fotoaparát ukládá do souboru, a nic dalšího: 1 000 fotek zabere 0,12 s, proti rozpočtu 1 s stanovenému ještě před napsáním kódu. Ostřejší miniatury o 400 px pak vznikají na pozadí, na dvou vláknech, aby Fotollection zůstal svižný, a ukládají se do mezipaměti na disku, takže znovu otevřená složka je tu hned. U 5 035 fotek trval import 383 ms a vložené miniatury 236 ms.

### Kam jde 17 ms náhledu

Změna nastavení překreslí náhled z kopie fotky o 2 048 px uložené v paměti, nikdy z celého souboru. Z 16,7 ms, které to trvá, zabere ořez a zmenšení Lanczosovým filtrem 9,7 ms, datum na fotce méně než 0,1 ms a kódování náhledu do JPEG pro obrazovku 7,0 ms. Tento kodér byl vyladěn měřením: s nastavením exportu (plné rozlišení barev a optimalizované tabulky) trval 16,2 ms; obrazovka je nepotřebuje, takže náhled používá poloviční rozlišení barev a obyčejné tabulky, zatímco exporty si nechávají plné barvy (4:4:4). Zkusil se i levnější filtr pro změnu velikosti a na tomto procesoru vyšel pomalejší (12,5 ms), takže Lanczos zůstal.

### Nativní kód proti prohlížeči

Stejná úloha (dekódování, ořez na 2:3, datum na fotce, JPEG v kvalitě 95) běžela na stejných 95 fotkách ve Fotollection a v Chromiu s jedním workerem na jádro. Rychlost exportu vyšla stejně, 18,0 proti 17,9 fotky za sekundu, což vyvrátilo náš první odhad: prohlížeč pomalý není. Důvody pro nativní kód jsou jinde: Fotollection zapisuje soubory 4:4:4 v plném rozlišení barev, kde kodér prohlížeče barvy půlí, zachovává barevný profil každé fotky, zapisuje soubory přímo na disk a spotřeboval o čtvrtinu méně paměti (1,35 GB proti 1,78 GB). Od té doby se kodér pro tisk ještě zrychlil: 23,5 fotky za sekundu při 16 MP.
