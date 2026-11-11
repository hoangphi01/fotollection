### Wcześniej: każde zdjęcie dekodowane w chwili kliknięcia

Do v0.10.0 podgląd powstawał wtedy, gdy o niego prosiłeś: odczyt pliku, dekodowanie zdjęcia (wywołanie RAW-a), obrót do pionu, zmniejszenie do 2048 px i konwersja do sRGB, a potem kadrowanie, kolor, data na zdjęciu i kodowanie na ekran. Dla zdjęcia 20 MP z Lumix GX9 trwało to od 274 do 278 ms dla JPEG-a i od 440 do 531 ms dla RAW-a, za każdym razem, gdy otwierało się zdjęcie spoza ostatnich ośmiu. Wystarczająco szybko dla jednego zdjęcia; za wolno, by przeglądać całą podróż.

### Standardowe podglądy, takie jak w Lightroomie

Od v0.11.0 praca, na którą użytkownik czeka, jest wykonywana, zanim o nią poprosi. Gdy miniatury folderu są gotowe, Fotollection w tle tworzy kopię 2048 px każdego zdjęcia, po jednym zdjęciu na wątek, i trzyma ją na dysku obok miniatur jako JPEG o rozmiarze około 0,66 MB (około 280 MB dla całej podróży). Kliknięcie dowolnego zdjęcia, bliskiego czy dalekiego, w tej sesji lub następnej, zaczyna od tej kopii: 27 ms zarówno dla JPEG-a, jak i dla RAW-a. Zostaje tylko własna praca podglądu (kadrowanie, zmiana rozmiaru, kolor, data na zdjęciu i kodowanie), te same 27 ms co z pamięci. Podgląd zrobiony z kopii na dysku różni się od świeżego średnio o 1,35 poziomu na 255, za mało, by to zobaczyć.{% include fn.html keys="fn_lightroom" %}

### Pamięć użyta dobrze, a nie jak najoszczędniej

Przeglądarka zdjęć, która na laptopie z 16 GB trzyma w pamięci osiem zdjęć, marnuje laptopa. Fotollection daje teraz podglądowi jedną czwartą pamięci komputera (na Macu z 16 GB to około 190 zdjęć w 2048 px, 20 wywołań RAW zachowanych dla ich suwaków i 600 MB na zoom), liczoną w bajtach i zwalnianą od najstarszych. Gdy zdjęcie jest na ekranie, wczytywane są też dwa następne w kierunku, w którym się przesuwasz, i jedno za nim, więc klawisze strzałek nigdy nie czekają. Pułap 2 GB nadal obowiązuje przy eksporcie.

### Zdjęcie na ekranie idzie pierwsze

Naraz wywoływany jest tylko jeden RAW, tak by partia zdjęć RAW zmieściła się w pamięci. Wcześniej kolejność w kolejce była kwestią szczęścia: przygotowywany podgląd, szacowanie rozmiaru eksportu albo sam eksport mogły kazać czekać zdjęciu, które kliknąłeś. Teraz każde wywołanie mówi, kto na nie czeka, a praca w tle ustępuje, ilekroć zdjęcie na ekranie czegoś potrzebuje. Ta sama praca ujawniła też dwa rzadkie zawieszenia, których już nie ma: jedno, w którym każdy wątek czekał na wywołanie, dla którego nie został żaden wolny wątek, i drugie, w którym ekran czekał na obraz, o który nigdy nie poprosił.

| Wersja | JPEG p50 / p95 | RAW p50 / p95 | Jak |
|---|---|---|---|
| v0.9.0 | 274 / 286 ms | 531 / 550 ms | Dekodowane przy kliknięciu; RAW przez macOS |
| v0.10.0 | 278 / 293 ms | 440 / 471 ms | Dekodowane przy kliknięciu; RAW przez nasze własne wywołanie |
| **v0.11.0** | **27 / 32 ms** | **27 / 30 ms** | Ze standardowego podglądu na dysku lub z pamięci |

Zmierzone na zdjęciach z praskiej podróży właściciela projektu (396 JPEG i 26 RAW, 20 MP) na Apple M3 za pomocą `measure_browse`: v0.9.0 zbudowane z własnego tagu, v0.10.0 na kodzie tuż przed tą zmianą.
