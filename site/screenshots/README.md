# Website screenshots

The screenshots on the website show the **real app on demo photos**, never anyone's own photos.

1. `python3 -I site/screenshots/fetch_photos.py <folder>` downloads the demo photos listed in [`photos.json`](photos.json) from Wikimedia Commons and checks each one's SHA-256 and licence. They are not kept in the repository.
2. `site/screenshots/build.sh <folder> <work-dir>` then:
   - runs the debug app with `FOTOLLECTION_DEMO_FOLDER` and `FOTOLLECTION_DEMO_DUMP` ([`src-tauri/src/demo.rs`](../../src-tauri/src/demo.rs)): it imports the folder with its own settings, cache and project (never the user's), runs the best-shot analysis, records what each IPC command answers and the image protocol's bytes, then quits;
   - shoots the real interface (`ui/dist`) with Playwright ([`shoot.mjs`](shoot.mjs)), answering IPC from that recording, in English and Vietnamese, dark and light: the library, the date stamp, the info frame, the crop editor and best-shot groups;
   - renders the link previews from [`og.html`](og.html) (1200 × 630);
   - writes 1600 px WebP screenshots and JPEG link previews to `site/assets/img/`.

Photos are chosen by file name in `shoot.mjs`. Needs cargo, npm, `cwebp` and Python with Pillow.

## Demo photos

All are CC0 on Wikimedia Commons. Camera EXIF is intact, so the info frame and the date stamp show real data; one photo has GPS, for the place on the date line.

| File | Source | Licence | Author | SHA-256 |
|------|--------|---------|--------|---------|
| `alte-aula-tuebingen-2018.jpg` | [Alte-aula-tuebingen-2018.jpg](https://commons.wikimedia.org/wiki/File:Alte-aula-tuebingen-2018.jpg) | CC0 | Dktue | `d8a648fc87644914…` |
| `ammerschlag.jpg` | [Ammerschlag.jpg](https://commons.wikimedia.org/wiki/File:Ammerschlag.jpg) | CC0 | Dktue | `d206497f2c588802…` |
| `anlagensee-mit-nymphengruppe-1.jpg` | [Anlagensee-mit-nymphengruppe-1.jpg](https://commons.wikimedia.org/wiki/File:Anlagensee-mit-nymphengruppe-1.jpg) | CC0 | Dktue | `772496cb62439e13…` |
| `anlagensee-nymphengruppe-durch-trauerweide.jpg` | [Anlagensee-nymphengruppe-durch-trauerweide.jpg](https://commons.wikimedia.org/wiki/File:Anlagensee-nymphengruppe-durch-trauerweide.jpg) | CC0 | Dktue | `7cdc7b1a4764b53a…` |
| `antibes-48931644202.jpg` | [Antibes (48931644202).jpg](https://commons.wikimedia.org/wiki/File:Antibes_(48931644202).jpg) | CC0 | Pierre Blaché from Paris, France | `c491f610a321db84…` |
| `baeren-tuebingen-1.jpg` | [Baeren-tuebingen-1.jpg](https://commons.wikimedia.org/wiki/File:Baeren-tuebingen-1.jpg) | CC0 | Dktue | `57f7252b227a96f9…` |
| `baeren-tuebingen-2.jpg` | [Baeren-tuebingen-2.jpg](https://commons.wikimedia.org/wiki/File:Baeren-tuebingen-2.jpg) | CC0 | Dktue | `22d4e7c9bc24228b…` |
| `bennetts-beach-hawks-nest-captured-by-kate-branch.jpg` | [Bennetts Beach Hawks Nest captured by Kate Branch.jpg](https://commons.wikimedia.org/wiki/File:Bennetts_Beach_Hawks_Nest_captured_by_Kate_Branch.jpg) | CC0 | Kate Branch | `9e9924037167a910…` |
| `bir-hakeim-bridge-paris-24-june-2021.jpg` | [Bir-Hakeim bridge, Paris 24 June 2021.jpg](https://commons.wikimedia.org/wiki/File:Bir-Hakeim_bridge,_Paris_24_June_2021.jpg) | CC0 | Pierre Blaché from Paris, France | `93a007b9dad22a2b…` |
| `brillenpinguin-in-der-wilhelma.jpg` | [Brillenpinguin in der Wilhelma.jpg](https://commons.wikimedia.org/wiki/File:Brillenpinguin_in_der_Wilhelma.jpg) | CC0 | Dktue | `9d4d3d33a1ad7085…` |
| `charles-bridge-at-sunset-prague-50489016846.jpg` | [Charles Bridge at Sunset, Prague (50489016846).jpg](https://commons.wikimedia.org/wiki/File:Charles_Bridge_at_Sunset,_Prague_(50489016846).jpg) | CC0 | Pierre Blaché from Paris, France | `c363c220360e5ddb…` |
| `erdmaennchen-in-der-wilhelma-1.jpg` | [Erdmännchen in der Wilhelma 1.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_1.jpg) | CC0 | Dktue | `87a86548e8b620b5…` |
| `erdmaennchen-in-der-wilhelma-2.jpg` | [Erdmännchen in der Wilhelma 2.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_2.jpg) | CC0 | Dktue | `46e942a31e195be8…` |
| `erdmaennchen-in-der-wilhelma-3.jpg` | [Erdmännchen in der Wilhelma 3.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_3.jpg) | CC0 | Dktue | `653ae3bc20f37d0f…` |
| `erdmaennchen-in-der-wilhelma-4.jpg` | [Erdmännchen in der Wilhelma 4.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_4.jpg) | CC0 | Dktue | `fbe3dd08c032d4e7…` |
| `erdmaennchen-in-der-wilhelma-5.jpg` | [Erdmännchen in der Wilhelma 5.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_5.jpg) | CC0 | Dktue | `8d7c1c3f36486ee4…` |
| `erdmaennchen-in-der-wilhelma-6.jpg` | [Erdmännchen in der Wilhelma 6.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_6.jpg) | CC0 | Dktue | `2a6964ed92ba65f8…` |
| `erdmaennchen-in-der-wilhelma-7.jpg` | [Erdmännchen in der Wilhelma 7.jpg](https://commons.wikimedia.org/wiki/File:Erdm%C3%A4nnchen_in_der_Wilhelma_7.jpg) | CC0 | Dktue | `758978e30c430af8…` |
| `feuerwehrboot-vor-neckarmueller-bei-nacht.jpg` | [Feuerwehrboot-vor-neckarmueller-bei-nacht.jpg](https://commons.wikimedia.org/wiki/File:Feuerwehrboot-vor-neckarmueller-bei-nacht.jpg) | CC0 | Dktue | `7cbd9d24efc6e1d5…` |
| `grevy-zebras-in-der-wilhelma-1.jpg` | [Grevy-Zebras in der Wilhelma 1.jpg](https://commons.wikimedia.org/wiki/File:Grevy-Zebras_in_der_Wilhelma_1.jpg) | CC0 | Dktue | `536b882c72f82494…` |
| `grevy-zebras-in-der-wilhelma-2.jpg` | [Grevy-Zebras in der Wilhelma 2.jpg](https://commons.wikimedia.org/wiki/File:Grevy-Zebras_in_der_Wilhelma_2.jpg) | CC0 | Dktue | `40c0fe8fa4a6bce0…` |
| `grevy-zebras-in-der-wilhelma-3.jpg` | [Grevy-Zebras in der Wilhelma 3.jpg](https://commons.wikimedia.org/wiki/File:Grevy-Zebras_in_der_Wilhelma_3.jpg) | CC0 | Dktue | `bbcffa37f62bf414…` |
