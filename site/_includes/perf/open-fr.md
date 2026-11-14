### Avant : chaque photo décodée au clic

Jusqu’à la v0.10.0, un aperçu était fait quand vous le demandiez : le fichier lu, la photo décodée (un RAW développé), remise droite, ramenée à 2 048 px et convertie en sRGB, puis recadrée, colorée, datée et encodée pour l’écran. Sur une photo de 20 MP d’un Lumix GX9, cela prenait 274 à 278 ms pour un JPEG et 440 à 531 ms pour un RAW, chaque fois qu’on ouvrait une photo qui ne faisait pas partie des huit dernières. Assez rapide pour une photo ; pas pour parcourir un voyage.

### Des aperçus standard, comme Lightroom les garde

Depuis la v0.11.0, le travail que l’utilisateur attend est fait avant qu’il le demande. Une fois les vignettes d’un dossier prêtes, Fotollection fait en arrière-plan la copie de 2 048 px de chaque photo, une photo à la fois par fil, et la garde sur le disque à côté des vignettes, en JPEG d’environ 0,66 Mo (environ 280 Mo pour tout le voyage). Un clic sur n’importe quelle photo, proche ou lointaine, dans cette séance ou la suivante, part de cette copie : 27 ms pour un JPEG comme pour un RAW. Il reste le travail propre à l’aperçu (recadrage, redimensionnement, couleur, date incrustée et encodage), les mêmes 27 ms que depuis la mémoire. Un aperçu fait à partir de la copie sur disque diffère d’un aperçu neuf de 1,35 niveau sur 255 en moyenne, trop peu pour se voir.{% include fn.html keys="fn_lightroom" %}

### La mémoire bien utilisée, pas le moins possible

Une visionneuse qui garde huit photos en mémoire sur un portable de 16 Go gaspille le portable. Fotollection donne désormais à l’aperçu un quart de la mémoire de la machine (sur un Mac de 16 Go, environ 190 photos à 2 048 px, 20 développements RAW gardés pour leurs curseurs et 600 Mo pour le zoom), comptés en octets et libérés du plus ancien au plus récent. Quand une photo est à l’écran, les deux suivantes dans le sens où vous avancez et celle de derrière sont chargées aussi, si bien que les touches fléchées n’attendent jamais. Le plafond de 2 Go tient toujours pour les exports.

### La photo à l’écran passe en premier

Un seul RAW est développé à la fois, pour qu’une série de photos RAW tienne en mémoire. Avant, l’ordre de la file était une question de chance : un aperçu en préparation, l’estimation de taille de l’export ou un export pouvaient faire attendre la photo sur laquelle vous aviez cliqué. Désormais, chaque développement dit qui l’attend, et le travail en arrière-plan s’efface dès que la photo à l’écran en a besoin. Ce même travail a aussi révélé deux blocages rares, disparus depuis : l’un où chaque fil attendait un développement qui n’avait plus aucun fil pour tourner, l’autre où l’écran attendait une image qu’il n’avait jamais demandée.

| Version | JPEG p50 / p95 | RAW p50 / p95 | Comment |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Décodé au clic ; RAW via macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Décodé au clic ; RAW via notre propre développement |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | Depuis l’aperçu standard sur disque, ou la mémoire |

Mesuré sur le voyage du propriétaire à Prague (396 JPEG et 26 RAW, 20 MP) sur un Apple M3 avec `measure_browse` : la v0.9.0 compilée depuis son propre tag, la v0.10.0 sur son code juste avant ce changement.
