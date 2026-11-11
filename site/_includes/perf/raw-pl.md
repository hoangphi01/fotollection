### Co naprawdę rejestruje matryca

Matryca aparatu to siatka milionów maleńkich studzienek zwanych fotodiodami. Gdy migawka jest otwarta, każda z nich zbiera docierające do niej światło i zamienia je w ładunek elektryczny; gdy się zamyka, przetwornik zamienia każdy ładunek w liczbę. Ta zależność jest **liniowa**: dwa razy więcej światła daje dwa razy większą liczbę. Nasze oczy wcale nie są liniowe, oceniają jasność proporcjonalnie, i dlatego plik RAW wygląda ciemno i płasko, dopóki nie zostanie wywołany.

Tę siatkę odczytują dwa rodzaje matryc:

- **CCD** (charge-coupled device) przesuwa ładunek każdej fotodiody, rząd po rzędzie, do jednego wzmacniacza na krawędzi układu. Była to matryca pierwszych aparatów cyfrowych i wielu wczesnych cyfrowych ścianek średnioformatowych: czyste obrazy, ale powolny odczyt i duży apetyt na energię.
- **CMOS** daje każdej fotodiodzie własny wzmacniacz i odczytuje wiele z nich naraz. Jest szybsza, zużywa mniej energii i jest tańsza w produkcji, więc ma ją dziś niemal każdy aparat i telefon. Większość matryc CMOS odczytuje rzędy jeden po drugim, dlatego szybko poruszający się obiekt może się pochylić na zdjęciu zrobionym migawką elektroniczną (*rolling shutter*), a matryce warstwowe (stacked), odczytywane znacznie szybciej, pochylają go mniej.

Te liczby mają znacznie więcej stopni niż JPEG. Aparat zapisuje 12 lub 14 bitów na fotodiodę, 4096 lub 16 384 poziomy, podczas gdy JPEG przechowuje 8 bitów, 256 poziomów. To te dodatkowe poziomy pozwalają rozjaśnić cień albo przyciemnić jasne niebo bez pasm. A brak światła to nie zero: każda matryca dodaje niewielkie przesunięcie, **poziom czerni**, by szum wokół czerni nie został ucięty. W próbce z Nikona D750 czerń to 600, a najjaśniejsza wartość, jaką matryca może zapisać, **poziom bieli**, to 15 520. Pierwsze, co robi każdy wywoływacz RAW, to sprowadzenie tych dwóch liczb do 0 i 1, osobno dla każdej pozycji filtra kolorów, bo niektóre aparaty dają każdej z nich własny poziom czerni.

### Kolor bierze się z filtra nad każdą fotodiodą

Fotodioda mierzy tylko, ile światła dociera, nigdy jego kolor. Aby widzieć kolor, matryca jest pokryta **mozaiką filtrów barwnych**: maleńkim czerwonym, zielonym lub niebieskim filtrem nad każdą fotodiodą, więc każda liczy tylko część widma.

- **Bayer**, opatentowany przez Bryce'a Bayera w Kodaku w 1976 roku, powtarza kwadrat 2 × 2: jeden czerwony, dwa zielone, jeden niebieski. Połowa fotodiod jest zielona, bo nasze oczy czerpią większość poczucia szczegółu ze środka widma, więc zieleń niesie większość ostrości. Niemal każdy aparat używa tego wzoru, w jednej z czterech orientacji (RGGB, BGGR, GRBG, GBRG), którą plik musi podać.
- **X-Trans**, wzór Fujifilm od 2012 roku, powtarza kwadrat 6 × 6 z czerwienią, zielenią i błękitem w każdym rzędzie i każdej kolumnie. Jego nieregularny układ zmniejsza ryzyko kolorowej mory na drobnych powtarzających się detalach, więc Fujifilm może pominąć filtr rozmywający, który ma wiele matryc, kosztem trudniejszej rekonstrukcji.
- **Inne konstrukcje:** matryce monochromatyczne nie mają żadnego filtra, więc każda fotodioda widzi pełny szczegół (aparaty Monochrom Leiki); Foveon Sigmy układa trzy warstwy, które pochłaniają kolor na różnych głębokościach krzemu; wiele telefonów grupuje po cztery fotodiody jednego koloru (*Quad Bayer*) i w słabym świetle odczytuje je jako jedną.

Matryca Bayera 24 MP zapisuje więc 24 miliony liczb, każdą dla jednego koloru: brakuje dwóch z trzech kolorów każdego piksela. Zamiana tej mozaiki w obraz to zadanie demozaikowania, opisanego niżej.

### Co zawiera plik RAW

JPEG to gotowy obraz: aparat podjął już wszystkie decyzje. Plik RAW przechowuje liczby z matrycy sprzed tych decyzji, ze wszystkim, co potrzebne, by podjąć je później:

- same próbki, po jednej na fotodiodę, i kolor filtra, pod którym leżała każda z nich;
- poziomy czerni i bieli;
- **obszar aktywny** (fotodiody, które widziały światło; kilka rzędów i kolumn na brzegach jest celowo zasłoniętych, by mierzyć czerń) oraz **domyślny kadr** aparatu;
- **balans bieli**, który aparat wybrał w chwili zrobienia zdjęcia;
- **macierze kolorów**, które mówią, jak własna czerwień, zieleń i błękit tej matrycy mają się do kolorów, które widzą ludzie;
- **podgląd**, JPEG zrobiony przez sam aparat, z jego własnym kolorem i tonacją, plus dane EXIF.

Każdy producent ma własny format, przeważnie oparty na TIFF i częściowo nieudokumentowany: CR2 i CR3 (Canon), NEF (Nikon), ARW (Sony), RW2 (Panasonic), ORF (Olympus), RAF (Fujifilm). **DNG** to format, który Adobe publikuje otwarcie (wersja 1.7), używany przez Ricoh, Leikę, Pentaxa i wiele telefonów, i to jego reguł trzyma się ta strona.

Fotollection czyta DNG własnym czytnikiem, napisanym na podstawie specyfikacji, a pozostałe formaty rozpakowuje rawler, biblioteka na licencji LGPL wbudowana w Fotollection. Miniatury w siatce w ogóle nie dotykają danych z matrycy: pochodzą z podglądu zapisanego w pliku, w 9 do 32 ms, i dlatego folder plików RAW otwiera się tak szybko jak folder JPEG-ów.

### Demozaikowanie: odtworzenie dwóch brakujących kolorów

Odtwarzanie dwóch brakujących kolorów każdego piksela z jego sąsiadów nazywa się **demozaikowaniem** i to ono bardziej niż jakikolwiek inny krok decyduje, jak ostre i czyste wygląda zdjęcie RAW. Każda metoda staje przed tym samym dylematem: uśrednianie sąsiadów jest bezpieczne na płaskich obszarach, ale na krawędzi miesza dwie różne powierzchnie, więc cała sztuka polega na tym, by wiedzieć, którzy sąsiedzi do siebie pasują.

- **Bilinear** po prostu uśrednia najbliższe fotodiody każdego koloru. To poziom odniesienia, z którym porównuje się każdą metodę: miękki obraz i kolorowe obwódki wzdłuż każdej ostrej krawędzi, bo czerwień, zieleń i błękit są uśredniane w nieco różnych miejscach.
- **Malvar–He–Cutler** (2004) zaczyna od tej średniej i koryguje ją szczegółem koloru, który fotodioda widziała, przez stałe filtry 5 × 5. Jest ostrzejsza i nadal bardzo szybka. Fotollection wywoływało nią do v0.10.0.
- **PPG** (patterned pixel grouping) najpierw ustala, w którym kierunku obraz zmienia się najmniej, i interpoluje tylko wzdłuż tego kierunku. Używa jej rawler, a do v0.10.0 także wersje Fotollection na Windows i Linuksa dla większości formatów.
- **RCD** (ratio corrected demosaicing, autorstwa Luisa Sanza Rodrígueza) pracuje na stosunkach między kolorami zamiast na różnicach, co lepiej sprawdza się zarówno w jasnych, jak i ciemnych partiach, i słynie z niewielu artefaktów na drobnych detalach; oferują ją darktable i RawTherapee.
- **Markesteijn**, od nazwiska Franka Markesteijna, to metoda stworzona dla X-Trans, w jednym przebiegu lub, wolniej i czyściej, w trzech.
- **Demozaikowanie Fotollection od v0.10.0** przejmuje od RCD sposób wyboru kierunku, a następnie szacuje zieleń z każdej strony, skorygowaną własnym kolorem danego miejsca (jak u Hamiltona i Adamsa), ważoną tym, jak bardzo zmienia się każda strona. Najpierw napisaliśmy własny krok stosunków z RCD i na naszych tablicach testowych wypadł gorzej, więc w aplikacji trafia wariant, który wygrał, pod własną nazwą.

Gdy metoda zgaduje źle, widać to na cztery typowe sposoby:

- **zipper (zamek błyskawiczny):** rząd naprzemiennie jasnych i ciemnych pikseli wzdłuż ostrej krawędzi, gdzie metoda wzięła sąsiadów z obu stron;
- **labirynt:** drobny, robaczkowaty wzór na płaskich obszarach, gdy dwie zielenie kwadratu Bayera nie odczytują się całkiem tak samo (leżą w rzędach o różnych sąsiadach i na wielu matrycach różnią się o procent lub dwa);
- **fałszywy kolor:** kolorowe kropki lub obwódki na detalach, które naprawdę są szare, jak tkanina, włosy, ceglany mur czy odległe ogrodzenie;
- **mora:** kolorowe fale na bardzo drobnych powtarzających się wzorach, drobniejszych, niż filtr barwny jest w stanie próbkować.

**Jak oceniamy metodę.** Oko łatwo dać się zwieść wynikowi, który wygląda ostrzej, więc każdą metodę ocenia się na obrazach, których prawda jest dokładnie znana. Komputer rysuje pięć tablic testowych w 768 px: płytkę strefową (pierścienie, które drobnieją ku brzegowi, aż do najdrobniejszego detalu, jaki mogą utrzymać zielone fotodiody), gwiazdę Siemensa z 72 ramionami, paski nasyconego koloru na szarym tle w trzech kierunkach, płynne przejścia kolorów oraz płaską szarość z dwiema zieleniami różniącymi się o 2 %. Każda jest zamieniana w to, co zapisałaby matryca Bayera, demozaikowana i porównywana z oryginałem. PSNR mierzy ogólny błąd w decybelach (więcej znaczy lepiej; każde 6 dB to błąd o połowę mniejszy); fałszywy kolor to średni kolor, który pojawia się na szarych tablicach; błąd krawędzi to średni błąd w odległości do 2 px od krawędzi.

| Tablica (768 px) | Bilinear | Malvar (nasza do v0.10.0) | Nasza od v0.10.0 |
|---|---|---|---|
| Płytka strefowa: PSNR, więcej znaczy lepiej | 13,0 dB | 17,3 dB | 16,5 dB |
| Płytka strefowa: fałszywy kolor, mniej znaczy lepiej | 65,9 | 43,0 | 18,7 |
| Gwiazda Siemensa: PSNR | 21,3 dB | 25,2 dB | 37,9 dB |
| Gwiazda Siemensa: fałszywy kolor | 9,4 | 8,4 | 0,16 |
| Paski nasyconego koloru: błąd wzdłuż krawędzi, mniej znaczy lepiej | 11,6 | 21,6 | 10,5 |
| Płynne przejścia: PSNR | 78,9 dB | 77,5 dB | 78,4 dB |
| Płaska szarość z różnicą zieleni: labirynt, mniej znaczy lepiej | 0,38 | 0,38 | 0 |
| Tekstura z martwymi pikselami: pozostałe na 10 000, mniej znaczy lepiej | 35,8 | 66,5 | 0 |

Malvar jest o około 4 dB lepsza od metody bilinear na drobnych szarych detalach, jak obiecuje jej publikacja, ale gorsza wzdłuż krawędzi między mocnymi kolorami, bo jej korekta pożycza szczegół jednego koloru dla pozostałych i zakłada, że zmieniają się razem, czego czerwony znak na szarym murze nie robi. Żadna z nich nie usuwa labiryntu, który nie jest problemem demozaikowania, lecz różnicą między dwiema zieleniami, do wyrównania wcześniej. To właśnie te luki zamyka nasz łańcuch od v0.10.0: martwe piksele i nierównowaga zieleni są poprawiane, póki każde miejsce zawiera jeszcze tylko to, co zmierzyło, kierunkowe demozaikowanie odtwarza kolory, a mała mediana na różnicach kolorów usuwa pozostały fałszywy kolor. Na tablicach testowych wygrywa wszędzie poza ogólnym PSNR płytki strefowej, o 0,8 dB niższym niż u Malvar przy najdrobniejszych detalach, gdzie aliasing dotyka każdej metody; nawet tam jej fałszywy kolor spada o połowę. Na dziewięciu prawdziwych plikach zbliża wywołanie każdego aparatu z matrycą Bayera do JPEG-a z aparatu tak samo lub bardziej i kosztuje około 180 ms na 24 MP, wobec 40 ms u Malvar: kilka dziesiątych sekundy więcej na zdjęcie za odbitki z czystszymi detalami.

### Od kolorów aparatu do twoich

Czerwień, zieleń i błękit aparatu to nie te z twojego ekranu: filtry każdej matrycy używają własnych barwników i nakładają się inaczej niż czopki naszych oczu. Macierze kolorów w pliku zamieniają kolory aparatu na **CIE XYZ**, standardowy model ludzkiego widzenia barw zdefiniowany w 1931 roku, a stamtąd na **sRGB**, przestrzeń kolorów większości ekranów i laboratoriów fotograficznych.

- **Balans bieli** jest pierwszy: wzmocnienie czerwieni i błękitu, tak by coś białego w scenie wyszło neutralne. Aparat zapisuje wybrane wzmocnienia, a Fotollection stosuje je przed demozaikowaniem, by różnice kolorów, na których opierają się powyższe metody, pozostały małe.
- DNG zwykle zawiera **dwie macierze**, jedną zmierzoną w ciepłym świetle żarowym, a drugą w świetle dziennym. Właściwa dla twojego zdjęcia leży pomiędzy: wywoływacz wylicza światło ujęcia z jego balansu bieli, jako temperaturę barwową, i miesza obie. Fotollection robi to samo dla pozostałych formatów, z macierzy, które rawler zna dla każdego modelu aparatu.
- Wszystko to dzieje się w **świetle liniowym**, na własnej skali matrycy, przed jakąkolwiek krzywą. Mieszanie, skalowanie i zmiana rozmiaru kolorów w świetle liniowym daje fizycznie poprawne wyniki; robienie tego po krzywej przyciemnia krawędzie i przesuwa kolory.

**Wygląd aparatu.** Własny JPEG aparatu nie jest neutralny: każdy producent dodaje swój kontrast, swoje kolory i odcienie skóry, a ten wygląd to część powodów, dla których ludzie wybierają markę. Od v0.10.0 zdjęcie RAW otwiera się w **neutralnym** renderingu, wiernym macierzy matrycy, z winietowaniem obiektywu i ekspozycją wybraną przez aparat; **Like the camera's JPEG** jest o jedno kliknięcie dalej, w karcie *RAW colour* zdjęcia RAW, dla jednego zdjęcia lub dla wszystkich RAW-ów naraz. (Do v0.10.0 każdy RAW był dopasowywany do JPEG-a z aparatu; projekt zapisany wtedy zachowuje ten wygląd.) Ten wybór dopasowuje JPEG z aparatu na dwa sposoby, jedną krzywą i macierzą albo jedną krzywą na kanał, i zachowuje ten, który dla danego pliku wypada bliżej: na Nikonie D750 zmniejsza różnicę o połowę, z 2,26 do 1,16 w ΔE2000. Miniatury w siatce zachowują własny podgląd aparatu, jak w Lightroomie.

### Tonacja: od liczb liniowych do obrazu

Liczby liniowe wyglądają na ekranie ciemno i matowo, bo podwojenie światła, które matryca zapisuje jako dwa razy większą liczbę, wydaje się nam jednym stopniem jasności. **Krzywa tonalna** zamienia skalę matrycy w obraz: ustawia średnią szarość, unosi cienie, łagodnie wygasza światła zamiast od razu je obcinać i nadaje obrazowi kontrast. Obraz jest potem zapisywany z krzywą sRGB (jego „gammą”), która przeznacza więcej z 256 poziomów JPEG-a na cienie, gdzie nasze oczy najbardziej zauważają stopnie.

Dwie korekty, które aparat robi w swoim JPEG-u, nie są zapisane w danych RAW, a obie mają znaczenie dla odbitek:

- **Winietowanie obiektywu:** każdy obiektyw jest ciemniejszy ku rogom. Fotollection mierzy, na ile aparat je rozjaśnił, z jego własnego podglądu, jako wzmocnienie promieniowe. Gdy nasze wywołanie DNG po raz pierwszy porównano z JPEG-iem z aparatu, środek był o około 4 L\* za jasny, a rogi o 1,5 za ciemne; wzmocnienie promieniowe zmniejszyło różnicę z ΔE 3,8 do 2,67, a każdy obszar zdjęcia mieścił się w ±0,5 L\*.
- **Dystorsja:** lekkie beczkowate lub poduszkowate zniekształcenie obiektywu. Aparaty Micro Four Thirds (Panasonic, Olympus) są projektowane tak, by korygował je aparat, a ich pliki zapisują, jak to zrobić. Na Macu takie pliki wywołuje macOS, który stosuje tę korektę; korekta w naszym własnym wywołaniu jest zaplanowana (FR-085).

### Jak Fotollection wywołuje RAW, krok po kroku

Dla pełnego wywołania, tego, które stoi za podglądem wydruku, zoomem i eksportem:

1. **Rozpoznanie** formatu po pierwszych bajtach pliku, nigdy po nazwie, więc plik o zmienionej nazwie albo RAW, który wygląda jak zwykły TIFF, jest nadal odczytywany poprawnie.
2. **Odczyt** liczb z matrycy: DNG naszym własnym czytnikiem, pozostałe formaty rozpakowywane przez rawler. Plik zbyt duży lub uszkodzony jest odrzucany, zanim zostanie zdekodowany jakikolwiek piksel, z kodem błędu.
3. **Poziomy:** czerń do 0, a biel do 1, dla każdej pozycji filtra kolorów, liczonej od rogu obszaru aktywnego.
4. **Balans bieli**, potem **demozaikowanie**: najpierw martwe piksele i równowaga zieleni, potem kierunkowe demozaikowanie i mediana na fałszywy kolor (Markesteijn dla X-Trans pojawi się w v0.15.0).
5. **Kolor:** macierze aparatu, zmieszane dla światła ujęcia, w świetle liniowym.
6. **Tonacja i wygląd**, dopasowane do podglądu z aparatu, z jego winietowaniem obiektywu.
7. Od tego miejsca zdjęcie RAW jest jak każde inne: kadr, data na zdjęciu, ramka, rozmiar odbitki i profil kolorów, eksport w pełnej rozdzielczości w 4:4:4 w jakości 95.

Kilka zasad obowiązuje przez cały czas. Naraz działa tylko jedno wywołanie, rozłożone na wszystkie rdzenie procesora i nigdy w wątku samego okna, więc Fotollection odpowiada dalej, gdy wywołuje się zdjęcie 30 MP. Eksport zawsze działa na procesorze, więc odbitka wychodzi taka sama na każdym komputerze; karta graficzna przyspiesza tylko to, co widzisz podczas edycji. A kod jest nasz własny, napisany na podstawie opublikowanych specyfikacji i publikacji naukowych, nie skopiowany z innych projektów, a potem sprawdzony: na syntetycznych tablicach testowych powyżej, na dziewięciu plikach RAW z dziewięciu aparatów (CC0, z raw.pixls.us) i przez celowe przycinanie i psucie plików, tak by zły plik dawał błąd, a nigdy awarię.

**Od v0.10.0: własne wywołanie dla każdego aparatu z matrycą Bayera.** rawler tylko rozpakowuje pliki, a cała reszta należy do Fotollection. Pełne wywołanie dziewięciu przykładowych plików, mediana z 3 pomiarów, na tym samym laptopie: nasze wywołanie z czystszym demozaikowaniem wobec prostszego z rawlera i wywołania macOS:

| Aparat (format) | Własne Fotollection | rawler | macOS |
|---|---|---|---|
| Panasonic GX9 (RW2, 20 MP) | 280 ms | 216 ms | 436 ms |
| Panasonic S5 (RW2, 24 MP) | 442 ms | 301 ms | 1044 ms |
| Olympus E-M10 III (ORF, 16 MP) | 308 ms | 364 ms | 1164 ms |
| Sony A7 III (ARW, 24 MP) | 372 ms | 241 ms | 488 ms |
| Canon R10 (CR3, 24 MP) | 494 ms | 360 ms | 1647 ms |
| Ricoh GR III (DNG, 24 MP) | 523 ms | 650 ms | — |
| Nikon D750 (NEF, 24 MP) | 502 ms | 402 ms | 412 ms |
| Canon 5D Mark IV (CR2, 31 MP) | 691 ms | 565 ms | 418 ms |
| Fujifilm X-T3 (RAF, 26 MP, X-Trans) | następny | 524 ms | 1339 ms |

Nasze wywołanie potrzebuje od jednej dziesiątej do jednej czwartej sekundy więcej niż rawler, za czystsze demozaikowanie i dopasowany kolor, i na większości aparatów pozostaje szybsze niż macOS; dzieje się to raz na zdjęcie, a suwak potem wykańcza zachowaną kopię w 4 do 11 ms. Na Macu pliki Panasonica i Olympusa zostają przy macOS ze względu na korektę obiektywu, dopóki nasze własne wywołanie nie zacznie korygować dystorsji. Szare pola z S5 wychodzą bardziej neutralne niż przez rawler (zafarb 1,42 wobec 2,43, gdzie 0 to idealna szarość). Fujifilm X-Trans pojawi się następny, z metodą Markesteijna, a po nim wywołanie na karcie graficznej dla podglądu, z wynikiem procesora jako wzorcem.
