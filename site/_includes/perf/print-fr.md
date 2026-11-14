### Pourquoi les tirages issus d’un éditeur web paraissent ternes

Le test exporte 20 photos d’appareil et 5 versions Display P3 (la couleur plus large des téléphones et des Mac récents) par le véritable export de Fotollection, puis lit les deux fichiers avec un décodeur indépendant, les convertit en CIELAB via leurs profils de couleur et les compare pixel par pixel à l’original. Un export typique d’éditeur web passe par la même comparaison : profil abandonné, réduit à 2 000 px puis agrandi de nouveau pour le tirage, enregistré en qualité 80 avec la couleur divisée par deux (4:2:0).

| Médiane | Fotollection, sRGB | Éditeur web | Fotollection, Display P3 | Éditeur web |
|---|---|---|---|---|
| ΔE2000 (erreur de couleur, invisible sous 1) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Saturation conservée | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Détail fin conservé | 99,8 % | 13 % | 100 % | 18 % |

Deux causes expliquent l’aspect terne. Réduire à 2 000 px puis agrandir de nouveau pour le labo jette 87 % du détail fin qui rend un tirage net. Et abandonner le profil de couleur fait lire une photo Display P3 comme si elle était en sRGB, ce qui lui retire 13 % de sa saturation. Fotollection garde le profil (25 fichiers sur 25), exporte à la pleine résolution de la photo avec la couleur complète (4:4:4) en qualité 95 (mesuré alors ; 90 par défaut depuis la v0.13.0, toujours sous une différence visible), et ses plus grandes erreurs se trouvent uniquement sur les bords nets, là où tout JPEG peine le plus. Encore à venir : de vrais fichiers HEIC d’iPhone, et des tirages mesurés au spectrophotomètre.
