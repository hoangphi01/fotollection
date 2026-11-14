### Ce que Facebook fait à une photo, et ce que nous faisons avant

Facebook réduit chaque photo à 2 048 px au plus, la recompresse en qualité 71 environ avec la couleur divisée par deux, et ignore les profils de couleur. L’export de partage fait ce travail correctement avant l’envoi, si bien que Facebook n’a plus rien à réduire : la photo est convertie en sRGB via son profil, redimensionnée avec un filtre de Lanczos en lumière linéaire, légèrement accentuée, et enregistrée en qualité 88. Chaque réglage a été choisi par la mesure : une accentuation plus forte (0,35 au lieu de 0,2) a obtenu un moins bon score, et les qualités 84 ou 80 perdaient plus qu’elles ne faisaient gagner.

| Médiane | Taille d’origine | Facebook, 2 048 px | Instagram, 1 080 px |
|---|---|---|---|
| Taille du fichier (JPEG d’appareil de 3,08 Mo) | 1,42 Mo | 0,51 Mo | 0,24 Mo |
| SSIM par rapport au redimensionnement idéal (1 = identique) | 0,981 | 0,979 | 0,982 |
| ΔE2000 | 0,96 | 1,27 | 1,58 |

Après la recompression de Facebook, le résultat reste plus proche de la photo que la méthode habituelle (SSIM 0,965 contre 0,961), et une photo Display P3 garde 99,6 % de sa couleur au lieu de 86 %, parce que son profil a été converti en vraies couleurs sRGB au lieu d’être abandonné. Les préréglages redimensionnés restent au-dessus de ΔE 1 à cause de la couleur divisée par deux que chaque plateforme applique de toute façon ; *Original size* garde chaque pixel pour environ la moitié du fichier, sous ΔE 1.
