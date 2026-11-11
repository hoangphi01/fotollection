### Dřív: každá fotka se dekódovala až po kliknutí

Až do v0.10.0 vznikal náhled teprve ve chvíli, kdy sis o něj řekl: soubor se načetl, fotka dekódovala (RAW vyvolal), otočila nastojato, zmenšila na 2 048 px a převedla do sRGB, pak se ořízla, obarvila, dostala datum a zakódovala pro obrazovku. U fotky s 20 MP z Lumixu GX9 to trvalo 274 až 278 ms u JPEG a 440 až 531 ms u RAW, pokaždé, když se otevřela fotka, která nebyla mezi posledními osmi. Pro jednu fotku dost rychlé; pro rychlé procházení cesty ne.

### Standardní náhledy, jak je drží Lightroom

Od v0.11.0 se práce, na kterou uživatel čeká, udělá dřív, než si o ni řekne. Jakmile jsou miniatury složky hotové, Fotollection na pozadí vytvoří pro každou fotku kopii o 2 048 px, jednu fotku najednou na vlákno, a uloží ji na disk vedle miniatur jako JPEG o asi 0,66 MB (asi 280 MB pro celou cestu). Kliknutí na kteroukoli fotku, blízkou i vzdálenou, v tomto sezení i v příštím, začíná od této kopie: 27 ms pro JPEG i pro RAW. Zbývá jen vlastní práce náhledu (ořez, změna velikosti, barva, datum na fotce a kódování), stejných 27 ms jako z paměti. Náhled vytvořený z kopie na disku se od čerstvého liší v průměru o 1,35 úrovně z 255, příliš málo na to, aby to bylo vidět.{% include fn.html keys="fn_lightroom" %}

### Paměť využitá dobře, ne co nejméně

Prohlížeč fotek, který na notebooku s 16 GB drží v paměti osm fotek, notebook promrhá. Fotollection teď dává náhledům čtvrtinu paměti počítače (na Macu s 16 GB asi 190 fotek o 2 048 px, 20 vyvolaných RAW uchovaných pro jejich posuvníky a 600 MB pro zoom), počítanou v bajtech a uvolňovanou od nejstarší. Když je fotka na obrazovce, načtou se i další dvě ve směru, kterým se pohybuješ, a jedna za ní, takže šipky nikdy nečekají. Strop 2 GB pro exporty dál platí.

### Fotka na obrazovce má přednost

Najednou se vyvolává jen jeden RAW, aby se dávka RAW fotek vešla do paměti. Dřív bylo pořadí ve frontě věcí náhody: připravovaný náhled, odhad velikosti exportu nebo export mohly nechat čekat fotku, na kterou jsi klikl. Teď každé vyvolání říká, kdo na něj čeká, a práce na pozadí ustoupí pokaždé, když ho potřebuje fotka na obrazovce. Stejná práce odhalila i dvě vzácná zaseknutí, která jsou teď pryč: jedno, kdy všechna vlákna čekala na vyvolání, na které už nezbylo žádné vlákno, a druhé, kdy obrazovka čekala na obrázek, o který si nikdy neřekla.

| Verze | JPEG p50 / p95 | RAW p50 / p95 | Jak |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Dekódováno po kliknutí; RAW přes macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Dekódováno po kliknutí; RAW naším vlastním vyvoláním |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | Ze standardního náhledu na disku nebo z paměti |

Měřeno na majitelově cestě do Prahy (396 JPEG a 26 RAW, 20 MP) na Apple M3 pomocí `measure_browse`: v0.9.0 sestavená z vlastního tagu, v0.10.0 na kódu těsně před touto změnou.
