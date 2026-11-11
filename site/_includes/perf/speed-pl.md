### Co się dzieje, gdy otwierasz folder

Folder zdjęć nie jest czytany w całości. Z każdego pliku Fotollection czyta dane EXIF (datę, aparat, miejsce) i małą miniaturę, którą każdy aparat zapisuje w pliku, i nic więcej: 1000 zdjęć zajmuje 0,12 s, przy budżecie 1 s ustalonym, zanim napisano kod. Ostrzejsze miniatury 400 px powstają potem w tle, na dwóch wątkach, by Fotollection pozostało szybkie, i trafiają do pamięci podręcznej na dysku, więc ponownie otwarty folder pojawia się natychmiast. Dla 5035 zdjęć import zajął 383 ms, a wbudowane miniatury 236 ms.

### Na co idzie 17 ms podglądu

Zmiana ustawienia renderuje podgląd ponownie z kopii zdjęcia 2048 px trzymanej w pamięci, nigdy z pełnego pliku. Z 16,7 ms, które to trwa, kadrowanie i zmiana rozmiaru filtrem Lanczos zajmują 9,7 ms, data na zdjęciu poniżej 0,1 ms, a kodowanie podglądu jako JPEG na ekran 7,0 ms. Ten koder dostrojono pomiarami: z ustawieniami eksportu (pełna rozdzielczość koloru i zoptymalizowane tablice) potrzebował 16,2 ms; ekran ich nie potrzebuje, więc podgląd używa połowy rozdzielczości koloru i zwykłych tablic, a eksport zachowuje pełny kolor (4:4:4). Wypróbowano tańszy filtr zmiany rozmiaru, który okazał się na tym procesorze wolniejszy (12,5 ms), więc Lanczos został.

### Kod natywny kontra przeglądarka

To samo zadanie (dekodowanie, kadr 2:3, data na zdjęciu, JPEG w jakości 95) uruchomiono na tych samych 95 zdjęciach w Fotollection i w Chromium, z jednym workerem na rdzeń. Szybkość eksportu wyszła taka sama, 18,0 wobec 17,9 zdjęcia na sekundę, co obaliło nasze pierwsze przypuszczenie: przeglądarka nie jest wolna. Powody, by pisać kod natywny, leżą gdzie indziej: Fotollection zapisuje pliki 4:4:4 w pełnej rozdzielczości koloru tam, gdzie koder przeglądarki ją połowi, zachowuje profil kolorów każdego zdjęcia, zapisuje pliki prosto na dysk i zużyło o jedną czwartą mniej pamięci (1,35 GB wobec 1,78 GB). Od tamtej pory koder do druku przyspieszył jeszcze bardziej: 23,5 zdjęcia na sekundę przy 16 MP.
