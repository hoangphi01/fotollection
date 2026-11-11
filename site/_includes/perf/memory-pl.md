### Dlaczego pamięć nie rośnie razem z folderem

Fotollection trzyma w bibliotece kilka bajtów na zdjęcie (13 MB dla 5035 zdjęć), zapisuje miniatury w pamięci podręcznej na dysku zamiast trzymać je w pamięci, trzyma w pamięci ograniczoną liczbę podglądów i pracuje naraz tylko na tylu pełnowymiarowych zdjęciach, ile mieści się w jego budżecie. Siatka na ekranie jest wirtualna: istnieją tylko kafelki, które widzisz. Aby to sprawdzić, te same 95 zdjęć podlinkowano 53 razy w folderze z 5035 plikami:

| | 95 zdjęć | 5035 zdjęć |
|---|---|---|
| Import i EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| Miniatury 400 px do pamięci podręcznej na dysku | 47,8 zdjęcia/s | 46,4 zdjęcia/s |
| Eksport, 6 naraz | 18,0 zdjęcia/s | 15,4 zdjęcia/s, 0 nieudanych |
| Szczytowa pamięć całego przebiegu | 1,35 GB | 1,45 GB |

53 razy więcej zdjęć kosztowało 0,1 GB więcej. Długi eksport jest nieco wolniejszy na zdjęcie tylko dlatego, że trwał ponad pięć minut, a procesor laptopa zwalnia pod długotrwałym obciążeniem.
