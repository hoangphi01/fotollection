### Ce qu’un capteur enregistre vraiment

Le capteur d’un appareil photo est une grille de millions de minuscules puits appelés photosites. Tant que l’obturateur est ouvert, chacun recueille la lumière qui l’atteint et la transforme en charge électrique ; quand il se ferme, un convertisseur transforme chaque charge en nombre. La relation est **linéaire** : deux fois plus de lumière donne un nombre deux fois plus grand. Nos yeux ne sont pas linéaires du tout, ils jugent la luminosité en proportion, c’est pourquoi un fichier RAW paraît sombre et plat tant qu’il n’est pas développé.

Deux sortes de capteurs lisent cette grille :

- **CCD** (dispositif à transfert de charge) fait glisser la charge de chaque photosite, rangée par rangée, vers un seul amplificateur au bord de la puce. C’était le capteur des premiers appareils numériques et de nombreux dos moyen format des débuts : des images propres, mais une lecture lente et une forte consommation.
- **CMOS** donne à chaque photosite son propre amplificateur et en lit beaucoup à la fois. Il est plus rapide, consomme moins et coûte moins cher à fabriquer, si bien que presque chaque appareil et chaque téléphone en a un aujourd’hui. La plupart des capteurs CMOS lisent leurs rangées l’une après l’autre, c’est pourquoi un sujet rapide peut pencher sur une photo prise à l’obturateur électronique (le *rolling shutter*), et pourquoi les capteurs empilés, qui lisent bien plus vite, penchent moins.

Les nombres ont bien plus de paliers qu’un JPEG. Un appareil enregistre 12 ou 14 bits par photosite, 4 096 ou 16 384 niveaux, là où un JPEG garde 8 bits, 256 niveaux. Ces niveaux en plus sont ce qui permet de déboucher une ombre ou de baisser un ciel lumineux sans postérisation. Et l’absence de lumière n’est pas zéro : chaque capteur ajoute un petit décalage, le **niveau de noir**, pour que le bruit autour du noir ne soit pas coupé. Dans l’échantillon du Nikon D750, le noir vaut 600 et la valeur la plus haute que le capteur puisse contenir, le **niveau de blanc**, 15 520. La toute première chose que fait un développeur RAW est de ramener ces deux nombres à 0 et 1, pour chaque position du filtre coloré, car certains appareils donnent à chacune son propre niveau de noir.

### La couleur vient d’un filtre sur chaque photosite

Un photosite ne mesure que la quantité de lumière qui arrive, jamais sa couleur. Pour voir la couleur, le capteur est couvert d’une **matrice de filtres colorés** : un minuscule filtre rouge, vert ou bleu sur chaque photosite, si bien que chacun ne compte qu’une partie du spectre.

- **Bayer**, breveté par Bryce Bayer chez Kodak en 1976, répète un carré de 2 × 2 : un rouge, deux verts, un bleu. La moitié des photosites sont verts parce que nos yeux tirent l’essentiel de leur perception du détail du milieu du spectre, si bien que le vert porte l’essentiel de la netteté. Presque tous les appareils utilisent ce motif, dans l’une de quatre orientations (RGGB, BGGR, GRBG, GBRG) que le fichier doit indiquer.
- **X-Trans**, le motif de Fujifilm depuis 2012, répète un carré de 6 × 6 avec du rouge, du vert et du bleu dans chaque rangée et chaque colonne. Sa disposition irrégulière rend le moiré coloré moins probable sur les détails fins répétitifs, si bien que Fujifilm peut se passer du filtre adoucissant que portent de nombreux capteurs, au prix d’une reconstruction plus difficile.
- **Autres conceptions :** les capteurs monochromes n’ont aucun filtre, si bien que chaque photosite voit tout le détail (les appareils Monochrom de Leica) ; le Foveon de Sigma empile trois couches qui absorbent la couleur à différentes profondeurs du silicium ; beaucoup de téléphones regroupent quatre photosites d’une même couleur (*Quad Bayer*) et les lisent comme un seul en faible lumière.

Un capteur Bayer de 24 MP enregistre donc 24 millions de nombres, chacun pour une seule couleur : deux des trois couleurs de chaque pixel manquent. Transformer cette mosaïque en image est le travail du dématriçage, plus bas.

### Ce que contient un fichier RAW

Un JPEG est une image terminée : l’appareil a déjà pris toutes les décisions. Un fichier RAW garde les nombres du capteur d’avant ces décisions, avec tout ce qu’il faut pour les prendre plus tard :

- les échantillons eux-mêmes, un par photosite, et sous quelle couleur de filtre chacun se trouvait ;
- les niveaux de noir et de blanc ;
- la **zone active** (les photosites qui ont vu la lumière ; quelques rangées et colonnes sur les bords sont masquées exprès, pour mesurer le noir) et le **recadrage par défaut** de l’appareil ;
- la **balance des blancs** choisie par l’appareil à la prise de vue ;
- des **matrices de couleur**, qui disent comment le rouge, le vert et le bleu propres à ce capteur se rapportent aux couleurs que voient les gens ;
- un **aperçu**, un JPEG fait par l’appareil lui-même, avec ses propres couleur et tonalité, plus les données EXIF.

Chaque fabricant a son propre format, le plus souvent bâti sur TIFF et en partie non documenté : CR2 et CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). Le **DNG** est le format qu’Adobe publie ouvertement (version 1.7), utilisé par Ricoh, Leica, Pentax et de nombreux téléphones, et celui dont cette page suit les règles.

Fotollection lit le DNG avec son propre lecteur, écrit d’après la spécification, et les autres formats sont décompressés par rawler, une bibliothèque LGPL intégrée à Fotollection. Les vignettes de la grille ne touchent jamais aux données du capteur : elles viennent de l’aperçu contenu dans le fichier, en 9 à 32 ms, c’est pourquoi un dossier de fichiers RAW s’ouvre aussi vite qu’un dossier de JPEG.

### Le dématriçage : retrouver les deux couleurs manquantes

Reconstruire les deux couleurs manquantes de chaque pixel à partir de ses voisins s’appelle le **dématriçage**, et c’est l’étape qui décide plus que toute autre de la netteté et de la propreté d’une photo RAW. Chaque méthode fait face au même dilemme : moyenner les voisins est sûr dans les zones unies, mais à travers un bord cela mélange deux surfaces différentes, si bien que tout l’art est de savoir quels voisins vont ensemble.

- **Bilinéaire** fait simplement la moyenne des photosites les plus proches de chaque couleur. C’est le plancher auquel chaque méthode se mesure : doux, et avec des franges colorées le long de chaque bord net, parce que le rouge, le vert et le bleu sont moyennés sur des endroits légèrement différents.
- **Malvar–He–Cutler** (2004) part de cette moyenne et la corrige avec le détail de la couleur que le photosite a vue, par des filtres fixes de 5 × 5. Il est plus net et reste très rapide. Fotollection développait avec lui jusqu’à la v0.10.0.
- **PPG** (patterned pixel grouping) décide d’abord dans quelle direction l’image change le moins, et n’interpole que dans cette direction. rawler l’utilise, tout comme les versions Windows et Linux de Fotollection pour la plupart des formats jusqu’à la v0.10.0.
- **RCD** (ratio corrected demosaicing, de Luis Sanz Rodríguez) travaille sur des rapports entre les couleurs plutôt que sur des différences, ce qui tient mieux dans les zones claires comme sombres, et il est connu pour donner peu d’artefacts sur les détails fins ; darktable et RawTherapee le proposent.
- **Markesteijn**, d’après Frank Markesteijn, est la méthode faite pour X-Trans, en une passe ou, plus lente et plus propre, en trois.
- **Le dématriçage de Fotollection depuis la v0.10.0** reprend la façon dont RCD décide de la direction, puis estime le vert de chaque côté, corrigé par la couleur propre du site (comme l’ont fait Hamilton et Adams), pondéré selon combien chaque côté varie. Nous avons d’abord écrit l’étape de rapport propre à RCD, et elle a obtenu de moins bons scores sur nos mires, si bien que la variante gagnante est celle qui est livrée, sous son propre nom.

Quand une méthode se trompe, cela se voit de quatre façons typiques :

- **fermeture éclair (zipper) :** une rangée de pixels alternativement clairs et sombres le long d’un bord net, là où la méthode a pris des voisins des deux côtés ;
- **labyrinthe (maze) :** un fin motif vermiculé dans les zones unies, quand les deux verts d’un carré Bayer ne lisent pas tout à fait la même chose (ils se trouvent sur des rangées aux voisins différents et, sur beaucoup de capteurs, diffèrent d’un ou deux pour cent) ;
- **fausse couleur :** des points ou des franges colorés sur un détail en réalité gris, comme un tissu, des cheveux, un mur de briques ou une clôture lointaine ;
- **moiré :** des ondes colorées sur des motifs répétitifs très fins, plus fins que ce que le filtre coloré peut échantillonner.

**Comment nous en jugeons.** Nos yeux se laissent facilement tromper par un résultat qui paraît plus net, alors chaque méthode est notée sur des images dont la vérité est connue exactement. Un ordinateur dessine cinq mires de 768 px : une mire de zones (des anneaux de plus en plus fins vers le bord, jusqu’au détail le plus fin que les photosites verts peuvent contenir), une étoile de Siemens à 72 branches, des barres de couleur saturée sur du gris dans trois directions, des dégradés de couleur doux, et un gris uni avec les deux verts écartés de 2 %. Chacune est transformée en ce qu’enregistrerait un capteur Bayer, dématricée et comparée à l’original. Le PSNR mesure l’erreur globale en décibels (plus haut, c’est mieux ; chaque 6 dB la divise par deux) ; la fausse couleur est la couleur moyenne qui apparaît sur les mires grises ; l’erreur de bord est l’erreur moyenne à moins de 2 px d’un bord.

| Mire (768 px) | Bilinéaire | Malvar (le nôtre jusqu’à la v0.10.0) | Le nôtre depuis la v0.10.0 |
|---|---|---|---|
| Mire de zones : PSNR, plus haut est mieux | 13,0 dB | 17,3 dB | 16,5 dB |
| Mire de zones : fausse couleur, plus bas est mieux | 65,9 | 43,0 | 18,7 |
| Étoile de Siemens : PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Étoile de Siemens : fausse couleur | 9,4 | 8,4 | 0,16 |
| Barres de couleur saturée : erreur le long des bords, plus bas est mieux | 11,6 | 21,6 | 10,5 |
| Dégradés doux : PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Gris uni avec un écart de vert : labyrinthe, plus bas est mieux | 0,38 | 0,38 | 0 |
| Texture avec pixels morts : restants pour 10 000, plus bas est mieux | 35,8 | 66,5 | 0 |

Malvar fait environ 4 dB de mieux que le bilinéaire sur le détail gris fin, comme son article le promet, mais moins bien le long des bords entre couleurs fortes, parce que sa correction emprunte le détail d’une couleur pour les autres et suppose qu’elles varient ensemble, ce que ne fait pas un panneau rouge sur un mur gris. Aucun des deux ne supprime le labyrinthe, qui n’est pas un problème de dématriçage mais une différence entre les deux verts, à égaliser avant. Ce sont les lacunes que notre chaîne comble depuis la v0.10.0 : les pixels morts et le déséquilibre des verts sont corrigés tant que chaque site ne contient encore que ce qu’il a mesuré, le dématriçage directionnel reconstruit les couleurs, et un petit filtre médian sur les différences de couleur supprime la fausse couleur restante. Sur les mires, il gagne partout sauf sur le PSNR global de la mire de zones, 0,8 dB sous celui de Malvar près du détail le plus fin, là où chaque méthode crée du repliement ; même là, sa fausse couleur est divisée par deux. Sur les neuf vrais fichiers, il rapproche autant ou davantage le développement de chaque appareil Bayer du JPEG de l’appareil lui-même, et coûte environ 180 ms sur 24 MP, contre 40 pour Malvar : quelques dixièmes de seconde de plus par photo, pour des tirages au détail plus propre.

### Des couleurs de l’appareil aux vôtres

Le rouge, le vert et le bleu de l’appareil ne sont pas ceux de votre écran : les filtres de chaque capteur utilisent leurs propres colorants, et ils se chevauchent autrement que les cônes de nos yeux. Les matrices de couleur du fichier transforment les couleurs de l’appareil en **CIE XYZ**, le modèle standard de la vision humaine des couleurs défini en 1931, et de là en **sRGB**, l’espace colorimétrique de la plupart des écrans et des laboratoires photo.

- La **balance des blancs** vient en premier : un gain sur le rouge et sur le bleu, pour que quelque chose de blanc dans la scène sorte neutre. L’appareil enregistre les gains qu’il a choisis, et Fotollection les applique avant le dématriçage, si bien que les différences de couleur sur lesquelles reposent les méthodes ci-dessus restent petites.
- Un DNG porte en général **deux matrices**, l’une mesurée sous une lumière de tungstène chaude et l’autre à la lumière du jour. La bonne pour votre photo se trouve entre les deux : le développeur déduit la lumière de la prise de vue de sa balance des blancs, sous forme de température de couleur, et mélange les deux. Fotollection fait de même pour les autres formats, à partir des matrices que rawler connaît pour chaque modèle d’appareil.
- Tout cela se passe en **lumière linéaire**, sur l’échelle propre du capteur, avant toute courbe. Mélanger, mettre à l’échelle et redimensionner les couleurs en lumière linéaire est ce qui donne des résultats physiquement justes ; le faire après la courbe assombrit les bords et décale les couleurs.

**Le rendu de l’appareil.** Le JPEG d’un appareil n’est pas neutre : chaque fabricant ajoute son contraste, sa couleur et ses tons chair, et ce rendu est en partie la raison pour laquelle on choisit une marque. Depuis la v0.10.0, une photo RAW s’ouvre avec un rendu **neutre**, fidèle à la matrice du capteur, avec le vignetage d’objectif corrigé et l’exposition choisie par l’appareil ; **Like the camera's JPEG** est à un clic dans la carte *RAW colour* d’une photo RAW, pour une photo ou pour tous les RAW à la fois. (Jusqu’à la v0.10.0, chaque RAW était ajusté au JPEG de l’appareil ; un projet enregistré à l’époque garde ce rendu.) Ce choix ajuste le JPEG de l’appareil des deux façons, une courbe et une matrice ou une courbe par canal, et garde celle qui s’en approche le plus pour ce fichier : sur le Nikon D750, il divise la différence par deux, de 2,26 à 1,16 en ΔE2000. Les vignettes de la grille gardent l’aperçu propre de l’appareil, comme dans Lightroom.

### La tonalité : des nombres linéaires à une image

Les nombres linéaires paraissent sombres et ternes à l’écran, parce qu’un doublement de la lumière, que le capteur enregistre comme un nombre deux fois plus grand, nous semble n’être qu’un palier de luminosité. Une **courbe de tonalité** transforme l’échelle du capteur en image : elle place le gris moyen, relève les ombres, adoucit les hautes lumières au lieu de les écrêter d’un coup, et donne à l’image son contraste. L’image est ensuite enregistrée avec la courbe sRGB (son « gamma »), qui consacre davantage des 256 niveaux d’un JPEG aux ombres, là où nos yeux remarquent le plus les paliers.

Deux corrections qu’un appareil fait dans son JPEG ne sont pas dans les données brutes, et toutes deux comptent pour les tirages :

- **Vignetage de l’objectif :** chaque objectif est plus sombre vers les coins. Fotollection mesure, d’après l’aperçu de l’appareil, de combien celui-ci les a éclaircis, sous forme de gain radial. Quand notre développement DNG a été comparé pour la première fois au JPEG de l’appareil, le centre était environ 4 L\* trop clair et les coins 1,5 trop sombres ; le gain radial a fait passer la différence de ΔE 3,8 à 2,67, chaque zone de la photo restant à ±0,5 L\*.
- **Distorsion :** la légère forme en barillet ou en coussinet d’un objectif. Les appareils Micro Four Thirds (Panasonic, Olympus) sont conçus pour que l’appareil la corrige, et leurs fichiers indiquent comment. Sur Mac, ces fichiers sont développés par macOS, qui applique la correction ; la corriger dans notre propre développement est prévu (FR-085).

### Comment Fotollection développe un RAW, étape par étape

Pour un développement complet, celui qui se trouve derrière l’aperçu d’impression, le zoom et l’export :

1. **Reconnaître** le format à partir des premiers octets du fichier, jamais de son nom, si bien qu’un fichier renommé ou un RAW qui ressemble à un simple TIFF est quand même bien lu.
2. **Lire** les nombres du capteur : le DNG avec notre propre lecteur, les autres formats décompressés par rawler. Un fichier trop gros ou endommagé est refusé avant qu’aucun pixel ne soit décodé, avec un code d’erreur.
3. **Niveaux :** le noir à 0 et le blanc à 1, pour chaque position du filtre coloré, comptée depuis le coin de la zone active.
4. **Balance des blancs**, puis **dématriçage** : les pixels morts et l’équilibre des verts d’abord, le dématriçage directionnel, le médian contre la fausse couleur (Markesteijn pour X-Trans arrive dans la v0.15.0).
5. **Couleur :** les matrices de l’appareil, mélangées selon la lumière de la prise de vue, en lumière linéaire.
6. **Tonalité et rendu**, ajustés à l’aperçu de l’appareil, avec son vignetage d’objectif.
7. À partir de là, une photo RAW est comme n’importe quelle autre : recadrage, date incrustée, cadre, format d’impression et profil de couleur, exportée en pleine résolution en 4:4:4 en qualité 95.

Quelques règles valent tout du long. Un seul développement tourne à la fois, réparti sur chaque cœur du processeur, et jamais sur le fil propre de la fenêtre, si bien que Fotollection continue de répondre pendant qu’une photo de 30 MP se développe. Les exports tournent toujours sur le processeur, si bien qu’un tirage sort identique sur chaque ordinateur ; la carte graphique n’accélère que ce que vous voyez pendant la retouche. Et le code est le nôtre, écrit d’après les spécifications et les articles publiés, pas copié d’autres projets, puis vérifié : sur les mires synthétiques ci-dessus, sur neuf fichiers RAW de neuf appareils (CC0, de raw.pixls.us), et en coupant et corrompant des fichiers exprès pour qu’un mauvais fichier donne une erreur, jamais un plantage.

**Depuis la v0.10.0 : notre propre développement pour chaque appareil Bayer.** rawler ne fait que décompresser les fichiers, et le reste est l’œuvre de Fotollection. Développement complet des neuf fichiers d’exemple, médiane de 3, sur le même portable : notre développement avec son dématriçage plus propre, contre celui, plus simple, de rawler et celui de macOS :

| Appareil (format) | Celui de Fotollection | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1 044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1 164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1 647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | à venir | 524 ms | 1 339 ms |

Le nôtre prend un dixième à un quart de seconde de plus que celui de rawler pour son dématriçage plus propre et sa couleur ajustée, et reste plus rapide que macOS sur la plupart des appareils ; cela n’arrive qu’une fois par photo, et un curseur refinit ensuite une copie gardée en 4 à 11 ms. Sur Mac, les fichiers Panasonic et Olympus restent avec macOS pour sa correction d’objectif, jusqu’à ce que notre propre développement corrige la distorsion. Les plages grises du S5 sortent plus neutres qu’avec rawler (une dominante de 1,42 contre 2,43, où 0 est parfaitement gris). Fujifilm X-Trans suit avec Markesteijn, et après lui le développement sur la carte graphique pour l’aperçu, avec le résultat du processeur comme référence.
