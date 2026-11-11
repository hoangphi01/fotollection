### Dlaczego odbitki z edytora internetowego wyglądają płasko

Test eksportuje 20 zdjęć z aparatu i 5 wersji w Display P3 (szerszej gamie kolorów najnowszych telefonów i Maców) przez prawdziwy eksport Fotollection, a następnie odczytuje oba pliki niezależnym dekoderem, przelicza je do CIELAB przez ich profile kolorów i porównuje piksel po pikselu z oryginałem. Typowy eksport z edytora internetowego przechodzi to samo porównanie: profil usunięty, zmniejszenie do 2000 px i powiększenie z powrotem do odbitki, zapis w jakości 80 z połową rozdzielczości koloru (4:2:0).

| Mediana | Fotollection, sRGB | Edytor internetowy | Fotollection, Display P3 | Edytor internetowy |
|---|---|---|---|---|
| ΔE2000 (błąd koloru, poniżej 1 jest niewidoczny) | 0,19 | 1,39 | 0,31 | 2,15 |
| PSNR | 52,6 dB | 37,9 dB | 50,8 dB | 37,0 dB |
| Zachowane nasycenie | 100 % | 99,9 % | 99,9 % | 86,8 % |
| Zachowane drobne szczegóły | 99,8 % | 13 % | 100 % | 18 % |

Płaski wygląd tłumaczą dwie przyczyny. Zmniejszenie do 2000 px i ponowne powiększenie dla laboratorium wyrzuca 87 % drobnych szczegółów, dzięki którym odbitka wygląda ostro. A usunięcie profilu kolorów każe odczytać zdjęcie w Display P3 tak, jakby było w sRGB, co zabiera 13 % jego nasycenia. Fotollection zachowuje profil (25 z 25 plików), eksportuje w pełnej rozdzielczości zdjęcia z pełnym kolorem (4:4:4) w jakości 95 (tak zmierzono wtedy; od v0.13.0 domyślnie 90, wciąż poniżej widocznej różnicy), a jego większe błędy występują tylko na ostrych krawędziach, gdzie każdy JPEG pracuje najciężej. Jeszcze przed nami: prawdziwe pliki HEIC z iPhone'a i odbitki zmierzone spektrofotometrem.
