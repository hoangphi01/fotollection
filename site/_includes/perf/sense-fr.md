### Un test de charge, pas une journée ordinaire

La plupart des gens trient un voyage à la fois, 2 000 à 3 000 photos. Pour voir si Fotollection reste stable bien au-delà, RollSense a reçu 10 044 photos d'un coup sur un portable Windows ordinaire de 2020 (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 Go) : JPEG d'appareil photo, photos de téléphone, captures d'écran et fichiers RAW Canon et Sony. Il en a lu 9 639 en 70 minutes sur tous les cœurs, le processeur occupé tout du long et la mémoire entre 400 et 850 Mo, sans jamais grimper.

### Où va le temps

Chaque photo est décodée une fois, réduite et lue par le modèle de sujets. Le modèle seul prend 128 ms par photo sur ce portable et l'étape entière 431 ms : environ 70 % du temps est donc la lecture et le décodage de la photo elle-même, surtout des JPEG et des RAW en pleine taille. La carte graphique est restée au repos, 1,5 % au plus : tout cela tourne en Rust sur le processeur.

### Plus de threads, et la limite d'un portable

Deux threads prennent 614 ms par photo, quatre 498 ms et huit 438 ms. Un portable fin partage 15 watts entre ses cœurs : avec les huit occupés, chacun tourne moins vite (1,8 GHz en moyenne, contre 3,4 GHz au repos), et quatre des huit threads partagent un cœur avec les quatre autres.

| | Ce portable (mesuré) |
|---|---|
| Un voyage de 2 500 photos | 18 min |
| La pellicule d'un téléphone, 15 000 photos | 108 min |
| Une photo relue depuis le cache | 1,3 ms |

Mesuré le 20/11/2026 avec `sort_pass_bench`, relevés chaque seconde.
