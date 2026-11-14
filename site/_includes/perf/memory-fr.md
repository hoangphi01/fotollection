### Pourquoi la mémoire ne grandit pas avec le dossier

Fotollection garde quelques octets par photo dans sa bibliothèque (13 Mo pour 5 035 photos), écrit les vignettes dans un cache sur disque au lieu de les garder, garde en mémoire un nombre limité d’aperçus, et ne travaille à la fois que sur autant de photos en pleine taille que son budget le permet. La grille à l’écran est virtuelle : seules les vignettes visibles existent. Pour le tester, les mêmes 95 photos ont été liées 53 fois dans un dossier de 5 035 fichiers :

| | 95 photos | 5 035 photos |
|---|---|---|
| Import et EXIF | 5 ms, 7 Mo | 383 ms, 13 Mo |
| Vignettes de 400 px vers le cache sur disque | 47,8 photos/s | 46,4 photos/s |
| Export, 6 à la fois | 18,0 photos/s | 15,4 photos/s, 0 échec |
| Pic mémoire de toute l’exécution | 1,35 Go | 1,45 Go |

53 fois plus de photos coûtent 0,1 Go de plus. Le long export est un peu plus lent par photo uniquement parce qu’il a duré plus de cinq minutes, et le processeur d’un portable ralentit sous une charge prolongée.
