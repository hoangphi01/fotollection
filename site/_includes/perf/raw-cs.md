### Co snímač doopravdy zaznamená

Snímač fotoaparátu je mřížka milionů drobných jamek, kterým se říká fotobuňky. Dokud je závěrka otevřená, každá z nich sbírá světlo, které na ni dopadne, a mění ho na elektrický náboj; když se zavře, převodník změní každý náboj na číslo. Vztah je **lineární**: dvakrát víc světla dá dvakrát větší číslo. Naše oči lineární vůbec nejsou, jas posuzují poměrově, a proto soubor RAW vypadá tmavě a ploše, dokud se nevyvolá.

Tu mřížku vyčítají dva druhy snímačů:

- **CCD** (charge-coupled device) posouvá náboj každé fotobuňky, řádek po řádku, k jednomu zesilovači na okraji čipu. Byl to snímač prvních digitálních fotoaparátů a mnoha raných digitálních stěn pro střední formát: čisté snímky, ale pomalé vyčítání a velká spotřeba energie.
- **CMOS** dává každé fotobuňce vlastní zesilovač a vyčítá jich mnoho najednou. Je rychlejší, spotřebuje méně energie a jeho výroba je levnější, takže ho dnes má téměř každý fotoaparát a telefon. Většina snímačů CMOS vyčítá řádky jeden po druhém, a proto se rychle se pohybující objekt na fotce pořízené elektronickou závěrkou může naklonit (*rolling shutter*) a proto se vrstvené snímače, které vyčítají mnohem rychleji, naklánějí méně.

Čísla mají mnohem víc stupňů než JPEG. Fotoaparát ukládá 12 nebo 14 bitů na fotobuňku, 4 096 nebo 16 384 úrovní, kde JPEG drží 8 bitů, 256 úrovní. Právě tyto úrovně navíc dovolí zvednout stín nebo stáhnout jasnou oblohu bez pruhování. A žádné světlo není nula: každý snímač přidává malý posun, **úroveň černé**, aby se šum kolem černé neořízl. Ve vzorku z Nikonu D750 je černá 600 a nejjasnější hodnota, kterou snímač udrží, **úroveň bílé**, je 15 520. Úplně první věc, kterou každý vyvolávač RAW udělá, je převést tato dvě čísla na 0 a 1, pro každou pozici barevného filtru, protože některé fotoaparáty dávají každé z nich vlastní úroveň černé.

### Barva pochází z filtru nad každou fotobuňkou

Fotobuňka měří jen to, kolik světla dopadne, nikdy jeho barvu. Aby snímač viděl barvy, je pokrytý **maticí barevných filtrů**: nad každou fotobuňkou je drobný červený, zelený nebo modrý filtr, takže každá počítá jen část spektra.

- **Bayer**, kterou si v roce 1976 nechal patentovat Bryce Bayer z Kodaku, opakuje čtverec 2 × 2: jedna červená, dvě zelené, jedna modrá. Polovina fotobuněk je zelených, protože naše oči berou většinu vjemu detailu ze středu spektra, takže zelená nese většinu ostrosti. Tento vzor používá téměř každý fotoaparát, v jedné ze čtyř orientací (RGGB, BGGR, GRBG, GBRG), kterou musí soubor uvést.
- **X-Trans**, vzor Fujifilmu od roku 2012, opakuje čtverec 6 × 6 s červenou, zelenou a modrou v každém řádku i každém sloupci. Jeho nepravidelné rozložení snižuje pravděpodobnost barevného moaré na jemných opakujících se detailech, takže Fujifilm může vynechat rozmazávací filtr, který má mnoho snímačů, za cenu obtížnější rekonstrukce.
- **Jiná řešení:** monochromatické snímače nemají filtr vůbec, takže každá fotobuňka vidí plný detail (fotoaparáty Monochrom od Leicy); Foveon od Sigmy skládá na sebe tři vrstvy, které pohlcují barvy v různých hloubkách křemíku; mnoho telefonů seskupuje čtyři fotobuňky jedné barvy (*Quad Bayer*) a za šera je čte jako jednu.

Bayerův snímač s 24 MP tedy zaznamená 24 milionů čísel, každé pro jedinou barvu: dvě ze tří barev každého pixelu chybějí. Proměnit tuto mozaiku v obraz je úkol demozaikování, viz níže.

### Co obsahuje soubor RAW

JPEG je hotový obrázek: fotoaparát už udělal všechna rozhodnutí. Soubor RAW drží čísla snímače z doby před těmito rozhodnutími, se vším, co je potřeba k tomu, aby se udělala později:

- samotné vzorky, jeden na fotobuňku, a pod jakou barvou filtru každý ležel;
- úrovně černé a bílé;
- **aktivní oblast** (fotobuňky, které viděly světlo; pár řádků a sloupců na okrajích je záměrně zakrytých, aby se změřila černá) a **výchozí ořez** fotoaparátu;
- **vyvážení bílé**, které fotoaparát zvolil při pořízení fotky;
- **barevné matice**, které říkají, jak se vlastní červená, zelená a modrá tohoto snímače vztahují k barvám, které vidí lidé;
- **náhled**, JPEG, který si fotoaparát udělal sám, s vlastní barvou a tónem, a k tomu data EXIF.

Každý výrobce má svůj formát, většinou postavený na TIFF a zčásti nezdokumentovaný: CR2 a CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** je formát, který Adobe otevřeně publikuje (verze 1.7), používají ho Ricoh, Leica, Pentax a mnoho telefonů a právě podle jeho pravidel postupuje tato stránka.

Fotollection čte DNG vlastní čtečkou napsanou podle specifikace a ostatní formáty rozbaluje rawler, knihovna pod licencí LGPL zabudovaná do Fotollection. Miniatury v mřížce data snímače vůbec nepoužívají: pocházejí z náhledu uvnitř souboru, za 9 až 32 ms, a proto se složka souborů RAW otevře stejně rychle jako složka JPEG.

### Demozaikování: dopočítání dvou chybějících barev

Rekonstrukci dvou chybějících barev každého pixelu z jeho sousedů se říká **demozaikování** a ta víc než kterýkoli jiný krok rozhoduje o tom, jak ostrá a čistá fotka z RAW bude. Každá metoda stojí před stejným dilematem: průměrovat sousedy je bezpečné v rovných plochách, ale přes hranu to smíchá dva různé povrchy, takže umění spočívá v tom poznat, kteří sousedé k sobě patří.

- **Bilineární** metoda prostě zprůměruje nejbližší fotobuňky každé barvy. Je to dno, podle kterého se měří každá metoda: měkké a s barevnými lemy podél každé ostré hrany, protože červená, zelená a modrá se průměrují přes trochu odlišná místa.
- **Malvar–He–Cutler** (2004) vychází z tohoto průměru a opravuje ho detailem barvy, kterou fotobuňka viděla, pomocí pevných filtrů 5 × 5. Je ostřejší a pořád velmi rychlá. Fotollection s ní vyvolával až do v0.10.0.
- **PPG** (patterned pixel grouping) nejdřív rozhodne, ve kterém směru se obraz mění nejméně, a interpoluje jen v tomto směru. Používá ji rawler a do v0.10.0 ji pro většinu formátů používaly i verze Fotollection pro Windows a Linux.
- **RCD** (ratio corrected demosaicing, od Luise Sanze Rodrígueze) pracuje s poměry mezi barvami místo rozdílů, což lépe drží ve světlých i tmavých oblastech, a je známá málem artefaktů na jemných detailech; nabízejí ji darktable a RawTherapee.
- **Markesteijn**, podle Franka Markesteijna, je metoda určená pro X-Trans, v jednom průchodu nebo, pomaleji a čistěji, ve třech.
- **Demozaikování Fotollection od v0.10.0** přebírá od RCD způsob, jak rozhodnout o směru, pak odhadne zelenou z každé strany opravenou o vlastní barvu fotobuňky (jak to dělali Hamilton a Adams), váženou podle toho, jak moc se každá strana mění. Nejdřív jsme napsali vlastní poměrový krok RCD a na našich testovacích obrazcích dopadl hůř, takže v aplikaci je varianta, která zvítězila, pod vlastním jménem.

Když metoda odhadne špatně, projeví se to čtyřmi typickými způsoby:

- **zip (zipper):** řada střídavě světlých a tmavých pixelů podél ostré hrany, kde metoda vzala sousedy z obou stran;
- **bludiště (maze):** jemný červovitý vzor v rovných plochách, když dvě zelené Bayerova čtverce nečtou úplně stejně (leží na řádcích s jinými sousedy a na mnoha snímačích se liší o procento nebo dvě);
- **falešné barvy:** barevné tečky nebo lemy na detailu, který je ve skutečnosti šedý, jako látka, vlasy, cihlová zeď nebo vzdálený plot;
- **moaré:** barevné vlny přes velmi jemné opakující se vzory, jemnější, než dokáže barevný filtr navzorkovat.

**Jak jednu hodnotíme.** Naše oči snadno oklame výsledek, který vypadá ostřeji, takže každá metoda se hodnotí na obrázcích, jejichž pravda je přesně známá. Počítač nakreslí pět testovacích obrazců o 768 px: zónovou desku (kruhy, které se k okraji zjemňují až po nejjemnější detail, který udrží zelené fotobuňky), Siemensovu hvězdu se 72 paprsky, pruhy syté barvy na šedé ve třech směrech, plynulé barevné přechody a rovnou šedou se dvěma zelenými o 2 % od sebe. Každý se převede na to, co by zaznamenal Bayerův snímač, demozaikuje se a porovná s originálem. PSNR měří celkovou chybu v decibelech (víc je lépe; každých 6 dB ji půlí); falešná barva je průměrná barva, která se objeví na šedých obrazcích; chyba hran je průměrná chyba do 2 px od hrany.

| Obrazec (768 px) | Bilineární | Malvar (naše do v0.10.0) | Naše od v0.10.0 |
|---|---|---|---|
| Zónová deska: PSNR, víc je lépe | 13,0 dB | 17,3 dB | 16,5 dB |
| Zónová deska: falešná barva, méně je lépe | 65,9 | 43,0 | 18,7 |
| Siemensova hvězda: PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Siemensova hvězda: falešná barva | 9,4 | 8,4 | 0,16 |
| Pruhy syté barvy: chyba podél hran, méně je lépe | 11,6 | 21,6 | 10,5 |
| Plynulé přechody: PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Rovná šedá s rozdílem zelených: bludiště, méně je lépe | 0,38 | 0,38 | 0 |
| Textura s vadnými pixely: zbylé na 10 000, méně je lépe | 35,8 | 66,5 | 0 |

Malvar je na jemném šedém detailu asi o 4 dB lepší než bilineární metoda, jak slibuje jeho článek, ale horší podél hran mezi sytými barvami, protože jeho oprava si půjčuje detail jedné barvy pro ostatní a předpokládá, že se mění společně, což červená cedule na šedé zdi nedělá. Ani jedna neodstraní bludiště, které není problémem demozaikování, ale rozdílem mezi dvěma zelenými, který je třeba vyrovnat předem. To jsou mezery, které náš řetězec od v0.10.0 zavírá: vadné pixely a nerovnováha zelených se opraví, dokud každá fotobuňka drží jen to, co změřila, směrové demozaikování zrekonstruuje barvy a malý medián na barevných rozdílech odstraní zbylou falešnou barvu. Na testovacích obrazcích vítězí všude kromě celkového PSNR zónové desky, kde je o 0,8 dB pod Malvarem u nejjemnějšího detailu, kde aliasuje každá metoda; i tam se jeho falešná barva půlí. Na devíti skutečných souborech přiblíží vyvolání každého fotoaparátu s Bayerovým snímačem k vlastnímu JPEG fotoaparátu stejně nebo víc a na 24 MP stojí asi 180 ms proti Malvarovým 40: pár desetin sekundy navíc na fotku, za tisky s čistšími detaily.

### Od barev fotoaparátu k tvým

Červená, zelená a modrá fotoaparátu nejsou ty z tvé obrazovky: filtry každého snímače používají vlastní barviva a překrývají se jinak než čípky našich očí. Barevné matice v souboru převedou barvy fotoaparátu do **CIE XYZ**, standardního modelu lidského barevného vidění definovaného v roce 1931, a odtud do **sRGB**, barevného prostoru většiny obrazovek a fotolabů.

- **Vyvážení bílé** přichází první: zisk na červené a na modré, aby něco bílého ve scéně vyšlo neutrálně. Fotoaparát zaznamená zisky, které zvolil, a Fotollection je použije před demozaikováním, aby barevné rozdíly, o které se výše uvedené metody opírají, zůstaly malé.
- DNG obvykle nese **dvě matice**, jednu změřenou pod teplým žárovkovým světlem a jednu za denního světla. Ta správná pro tvou fotku leží mezi nimi: vyvolávač z vyvážení bílé snímku zjistí jeho světlo jako barevnou teplotu a obě smíchá. Fotollection dělá totéž pro ostatní formáty, z matic, které rawler zná pro každý model fotoaparátu.
- To všechno se děje v **lineárním světle**, na vlastní stupnici snímače, před jakoukoli křivkou. Míchání, škálování a zmenšování barev v lineárním světle dává fyzikálně správné výsledky; po křivce to tmaví hrany a posouvá barvy.

**Vzhled fotoaparátu.** Vlastní JPEG fotoaparátu není neutrální: každý výrobce přidává svůj kontrast, své barvy a své pleťové tóny a tento vzhled je jedním z důvodů, proč si lidé vybírají značku. Od v0.10.0 se fotka RAW otevře v **neutrálním** podání, věrném matici snímače, se stíněním objektivu a expozicí, kterou zvolil fotoaparát; **Like the camera's JPEG** je o jedno kliknutí dál v kartě *RAW colour* fotky RAW, pro jednu fotku nebo pro všechny RAW najednou. (Do v0.10.0 se každý RAW přizpůsoboval JPEG fotoaparátu; projekt uložený tehdy si tento vzhled ponechá.) Tato volba přizpůsobí JPEG fotoaparátu oběma způsoby, jednou křivkou a maticí nebo jednou křivkou na kanál, a ponechá ten, který je pro daný soubor blíž: na Nikonu D750 rozdíl půlí, z 2,26 na 1,16 v ΔE2000. Miniatury v mřížce si ponechávají vlastní náhled fotoaparátu, jako v Lightroomu.

### Tón: od lineárních čísel k obrazu

Lineární čísla vypadají na obrazovce tmavě a mdle, protože zdvojnásobení světla, které snímač zaznamená jako dvojnásobné číslo, nám připadá jako jeden stupeň jasu. **Tónová křivka** promění stupnici snímače v obraz: umístí střední šedou, zvedne stíny, světla jemně zaoblí, místo aby je naráz ořízla, a dá obrazu kontrast. Obraz se pak uloží s křivkou sRGB (její „gamma“), která věnuje víc z 256 úrovní JPEG stínům, kde si naše oči kroků všimnou nejvíc.

Dvě opravy, které fotoaparát dělá ve svém JPEG, v datech RAW nejsou, a obě jsou pro tisk důležité:

- **Stínění objektivu:** každý objektiv je směrem do rohů tmavší. Fotollection změří, o kolik je fotoaparát zesvětlil, z jeho vlastního náhledu, jako radiální zisk. Když se naše vyvolání DNG poprvé porovnalo s JPEG fotoaparátu, střed byl asi o 4 L\* příliš světlý a rohy o 1,5 příliš tmavé; radiální zisk snížil rozdíl z ΔE 3,8 na 2,67, přičemž každá oblast fotky byla v rozmezí ±0,5 L\*.
- **Zkreslení:** mírný soudkovitý nebo poduškovitý tvar objektivu. Fotoaparáty Micro Four Thirds (Panasonic, Olympus) jsou navrženy tak, aby ho opravoval fotoaparát, a jejich soubory zaznamenávají jak. Na Macu tyto soubory vyvolává macOS, který opravu použije; oprava v našem vlastním vyvolání je v plánu (FR-085).

### Jak Fotollection vyvolá RAW, krok za krokem

Pro plné vyvolání, to za náhledem tisku, zoomem a exportem:

1. **Rozpoznání** formátu podle prvních bajtů souboru, nikdy podle názvu, takže i přejmenovaný soubor nebo RAW, který vypadá jako obyčejný TIFF, se přečte správně.
2. **Čtení** čísel snímače: DNG naší vlastní čtečkou, ostatní formáty rozbalí rawler. Příliš velký nebo poškozený soubor se odmítne dřív, než se dekóduje jediný pixel, s kódem chyby.
3. **Úrovně:** černá na 0 a bílá na 1, pro každou pozici barevného filtru, počítáno od rohu aktivní oblasti.
4. **Vyvážení bílé**, pak **demozaikování**: nejdřív vadné pixely a vyvážení zelených, směrové demozaikování, medián proti falešné barvě (Markesteijn pro X-Trans přijde ve v0.15.0).
5. **Barva:** matice fotoaparátu, smíchané podle světla snímku, v lineárním světle.
6. **Tón a vzhled**, přizpůsobené náhledu fotoaparátu, s jeho stíněním objektivu.
7. Od této chvíle je fotka RAW jako každá jiná: ořez, datum na fotce, rámeček, formát tisku a barevný profil, export v plném rozlišení ve 4:4:4 v kvalitě 95.

Po celou dobu platí několik pravidel. Najednou běží jen jedno vyvolání, rozložené na všechna jádra procesoru a nikdy ne na vlastním vlákně okna, takže Fotollection odpovídá, i když se vyvolává fotka s 30 MP. Exporty vždy běží na procesoru, takže tisk vyjde stejně na každém počítači; grafická karta jen zrychluje to, co vidíš při úpravách. A kód je náš vlastní, napsaný podle publikovaných specifikací a odborných článků, nezkopírovaný z jiných projektů, a pak ověřený: na syntetických obrazcích výše, na devíti souborech RAW z devíti fotoaparátů (CC0, z raw.pixls.us) a záměrným ořezáváním a poškozováním souborů, aby špatný soubor byl chybou, nikdy pádem.

**Od v0.10.0: naše vlastní vyvolání pro každý fotoaparát s Bayerovým snímačem.** rawler soubory jen rozbalí a zbytek je práce Fotollection. Plné vyvolání devíti vzorových souborů, medián ze 3, na stejném notebooku: naše vyvolání s čistším demozaikováním proti jednoduššímu z rawleru a proti macOS:

| Fotoaparát (formát) | Vlastní Fotollection | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1 044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1 164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1 647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | na řadě | 524 ms | 1 339 ms |

To naše trvá o desetinu až čtvrtinu sekundy déle než to z rawleru, kvůli čistšímu demozaikování a přizpůsobeným barvám, a na většině fotoaparátů zůstává rychlejší než macOS; děje se jednou na fotku a posuvník pak dokončí uchovanou kopii znovu za 4 až 11 ms. Na Macu zůstávají soubory z Panasonicu a Olympusu u macOS kvůli jeho korekci objektivu, dokud naše vlastní vyvolání neopraví zkreslení. Šedá pole z S5 vycházejí neutrálněji než přes rawler (barevný nádech 1,42 proti 2,43, kde 0 je dokonale šedá). Fujifilm X-Trans přijde s Markesteijnem a po něm vyvolání na grafické kartě pro náhled, s výsledkem procesoru jako referencí.
