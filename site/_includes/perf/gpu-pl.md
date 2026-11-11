### Dziesięć suwaków w cenie jednego

Każdy suwak koloru (jasność, kontrast, ciepło, nasycenie, wyblaknięcie, tonowanie cieni i świateł, czerń i biel) jest składany w jedną tablicę 33 × 33 × 33 kolorów: 35 937 punktów, które mówią, dokąd trafia każdy kolor. Nałożenie jej na zdjęcie polega na znalezieniu komórki tablicy dla każdego piksela i interpolacji między czterema jej narożnikami (interpolacja czworościenna), więc dziesięć suwaków kosztuje tyle co jeden. Zbudowanie tablicy przy ustawionych wszystkich suwakach trwa około 2,6 ms; nałożenie jej na podgląd 1600 px – poniżej 2,2 ms; na eksport 24 MP – około 24 ms.

### Dlaczego przeciąganie wciąż wydawało się wolne i co zmieniła karta graficzna

Mierzone krok po kroku, jedno przesunięcie suwaka kosztowało około 26 ms, a sam kolor stanowił z tego tylko 2 ms: resztę zajmowało kodowanie podglądu jako JPEG (7 do 11 ms) i ponowne dekodowanie go przez okno (9 do 15 ms). Karta graficzna usuwa oba te kroki. Gdy zaczyna się przeciąganie, zwykły podgląd i maska tego, gdzie jest zdjęcie (a nie data na zdjęciu, ramka czy obramowanie), trafiają do karty graficznej raz; przy każdym kroku idzie za nimi tylko nowa tablica, a mały program WebGL2 interpoluje ją dokładnie tak jak rdzeń: 1 ms na krok w WebKit systemu macOS, 2 ms w Chromium. Gdy puszczasz suwak, zastępuje go dokładny render rdzenia. Oba porównano na mocnym looku z przesuniętymi wszystkimi suwakami: różnią się średnio o 0,13 w ΔE2000 i o 0,77 na 99. percentylu, daleko poniżej 1, którą oko widzi przy porównaniu obok siebie.

Nie trzeba niczego przełączać. Przy pierwszym przeciągnięciu Fotollection sprawdza samo: jeśli nie ma WebGL2, renderer jest programowy albo krok testowy trwa 8 ms lub dłużej, zostaje na ścieżce procesora do końca sesji. Renderer programowy potrzebował w teście 16,5 ms na krok, więc słusznie zostaje wtedy na procesorze.

### Eksport na wszystkich rdzeniach

Dekodowanie i kodowanie zdjęcia działają na jednym rdzeniu każde, więc eksport jest szybki, gdy przetwarza się kilka zdjęć naraz. Ile – to zależy od pamięci: na zdjęciach 24 do 26 MP 2 naraz dawały 4,9 zdjęcia na sekundę, 4 – 9,4, 6 – 11,5, a 8 – 12,5, przy pamięci rosnącej z 0,49 do 1,21 GB. Fotollection wybiera tę liczbę z budżetu pamięci 1,6 GB i rozmiaru największego zdjęcia i zatrzymuje się na 6, bo na zdjęciach z prawdziwej podróży 8 naraz nie było szybsze, a zużywało o 0,2 GB więcej. Eksport nigdy nie używa karty graficznej: odbitka musi wyjść taka sama na każdym komputerze, a od tego zależą testy porównujące eksporty bajt po bajcie.
