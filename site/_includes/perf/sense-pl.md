### Test obciążenia, nie zwykły dzień

Większość osób porządkuje jedną wycieczkę naraz, 2000–3000 zdjęć. Aby sprawdzić, czy Fotollection zostaje stabilny daleko poza tym, RollSense dostał naraz 10 044 zdjęcia na zwykłym laptopie z Windows z 2020 roku (Lenovo ThinkPad X1, Intel i7-10610U, 15 W, 16 GB): pliki JPEG z aparatu, zdjęcia z telefonu, zrzuty ekranu oraz pliki RAW z Canona i Sony. Przeczytał 9639 z nich w 70 minut na wszystkich rdzeniach, procesor był zajęty przez cały czas, a pamięć mieściła się między 400 a 850 MB, nie rosnąc.

### Gdzie idzie czas

Każde zdjęcie jest raz dekodowane, zmniejszane i czytane przez model treści. Sam model zajmuje na tym laptopie 128 ms na zdjęcie, a cały krok 431 ms, więc około 70 % czasu to odczyt i dekodowanie samego zdjęcia, głównie plików JPEG i RAW w pełnym rozmiarze. Karta graficzna odpoczywała, najwyżej 1,5 %: wszystko to działa w Rust na procesorze.

### Więcej wątków i granica laptopa

Dwa wątki potrzebują 614 ms na zdjęcie, cztery 498 ms, osiem 438 ms. Cienki laptop dzieli 15 watów między rdzenie, więc gdy pracuje wszystkie osiem, każdy działa wolniej (średnio 1,8 GHz wobec 3,4 GHz w spoczynku), a cztery z ośmiu wątków dzielą rdzeń z pozostałymi czterema.

| | Ten laptop (zmierzone) |
|---|---|
| Wycieczka z 2500 zdjęciami | 18 min |
| Galeria telefonu z 15 000 zdjęć | 108 min |
| Zdjęcie ponownie z pamięci podręcznej | 1,3 ms |

Zmierzono 20.11.2026 narzędziem `sort_pass_bench`, dane co sekundę.
