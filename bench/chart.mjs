// SPDX-License-Identifier: GPL-3.0-only
// Turns bench/results/*.json into bench/results/summary.json and the README/RESULT charts
// (plain SVG, no dependencies), in English and Vietnamese. Each metric is the median of runs.
//
//   node bench/chart.mjs
//
// Charts (docs/images/, `-vi` suffix for Vietnamese):
//   bench-budgets.svg  measured vs the NFR budgets (reference: budget line)
//   bench-memory.svg   peak memory vs batch size (reference: 2 GB ceiling)
//   bench.svg          Fotollection vs Chromium (reference: the browser)
//   bench-preview.svg  where the warm preview time goes (reference: budget and Chromium)
//   bench-share.svg    share export after Facebook's recompression, vs the usual way (FR-053)
//   bench-raw.svg      camera RAW per format, and against a RAW-to-DNG conversion (FR-016)

import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { published } from '../eleventy.registry.mjs'

const root = new URL('..', import.meta.url).pathname
const results = join(root, 'bench/results')
const images = join(root, 'docs/images')
const read = (f) => JSON.parse(readFileSync(join(results, f), 'utf8'))

function runs(prefix) {
  return readdirSync(results)
    .filter((f) => new RegExp(`^${prefix}-\\d+\\.json$`).test(f))
    .map(read)
}
function median(list, key) {
  const v = list
    .map((r) => r[key])
    .filter((x) => typeof x === 'number')
    .sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : undefined
}
function summarize(list) {
  if (list.length === 0) return undefined
  const keys = Object.keys(list[0]).filter((k) => typeof list[0][k] === 'number')
  return {
    engine: list[0].engine,
    runs: list.length,
    ...Object.fromEntries(keys.map((k) => [k, median(list, k)])),
  }
}

const rust = summarize(runs('rust'))
const web = summarize(runs('web'))
const inflight8 = summarize(runs('inflight8'))
// Largest batch first; batch-<n>.json comes from bench/batch.sh.
const batches = readdirSync(results)
  .filter((f) => /^batch-\d+\.json$/.test(f))
  .map(read)
  .sort((a, b) => b.photos - a.photos)
const batch = batches[0]
const budgets = read('budgets.json')
const bestshot = read('bestshot.json')
const quality = read('quality.json')
const share = read('share.json')
const raw = read('raw.json')
const meta = read('machine.json')
// The GPU while editing and the photos in flight (ADR-0039, RESULT §22).
const gpu = read('gpu.json')
const drag = read('drag.json')
const inflight = read('inflight.json')
const browse = read('browse.json')
if (!rust || !web) throw new Error('run bench/run.sh first')

const openSource = batch ?? rust
const measured = {
  // Import + EXIF + embedded thumbnails, scaled to the NFR-001 unit (1,000 photos).
  openPer1000S: (openSource.importMs + openSource.t0Ms) / openSource.photos,
  openPhotos: openSource.photos,
  previewP95Ms: rust.previewWarmP95Ms,
  exportPerSecond: rust.exportPerSecond,
  peakGb: Math.max(rust.peakRssBytes, batch?.peakRssBytes ?? 0) / 1e9,
}
writeFileSync(
  join(results, 'summary.json'),
  `${JSON.stringify({ ...meta, measured, rust, web, inflight8, batch }, null, 2)}\n`,
)

// ---------------------------------------------------------------------------------------
// Words. Numbers use the locale's decimal separator (18.6 / 18,6).

const T = {
  en: {
    lower: 'lower is better',
    higher: 'higher is better',
    budget: 'budget',
    ceiling: 'ceiling',
    photos: 'photos',
    rust: 'Fotollection (Rust)',
    web: 'Browser (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'photos/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `${meta.megapixels} MP JPEGs · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} photos: median of ${rust.runs} runs${withBatch && batch ? ` · ${f(batch.photos, 0)} photos: 1 run` : ''}`,
    budgetsTitle: 'Measured against the performance budgets',
    open: 'Open a folder (import + EXIF + first thumbnails)',
    openUnit: (n) => `s per 1,000 photos (measured on ${n})`,
    preview: 'Preview update after a settings change',
    previewUnit: 'ms, p95',
    export: 'Export throughput',
    exportUnit: (mp) => `photos/s at ${mp} MP (budget is for 24 MP)`,
    memory: 'Peak memory, largest batch',
    memoryUnit: 'GB',
    memTitle: 'Peak memory stays flat as the batch grows',
    memUnit: 'GB peak RSS: import + thumbnails + export',
    compareTitle: (n) => `Same ${n} photos, same job: decode → crop 2:3 → date stamp → JPEG q95`,
    cExport: 'Batch export',
    cExportUnit: 'photos/s',
    cPreview: 'Warm preview (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Peak memory',
    cMemoryUnit: 'GB',
    previewTitle: 'Where the 1600 px preview time goes',
    resize: 'crop + resize',
    stamp: 'date stamp',
    encode: 'JPEG encode',
    ms: 'ms, p50',
    qTitle: 'Print quality: what an export keeps of the photo',
    qWeb: 'Typical web editor',
    qDetail: 'Detail kept (camera JPEGs)',
    qChroma: 'Colour saturation kept (Display P3, like iPhone photos)',
    qDe: 'Colour error, ΔE2000 mean (camera JPEGs)',
    qPercent: '% of the original · higher is better',
    qDeUnit: 'lower is better',
    qVisible: 'visible at 1',
    qFooter: (n, m) =>
      `${n} camera JPEGs and ${m} Display P3 versions, every pixel compared with the original · web editor: profile dropped, 4:2:0, q80, 2000 px`,
    shTitle: 'Photos for Facebook: what viewers see after Facebook recompresses them',
    shUsual: 'Usual way',
    shChroma: 'Colour saturation kept (Display P3, like iPhone photos)',
    shDiff: 'Difference from the original, 1 − SSIM (camera JPEGs)',
    shSize: 'File size per photo (camera JPEGs)',
    shSource: 'Camera file',
    shOriginal: 'Original size',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · lower is lighter',
    shFooter: (n, m) =>
      `${n} camera JPEGs and ${m} Display P3 versions · usual way: 2000 px, q80, profile dropped · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'Camera RAW: from the file to the screen',
    rwPerFormat: 'Print preview (full develop), per camera',
    rwCompare: 'Median of the nine cameras',
    rwThumb: 'Thumbnail (embedded preview)',
    rwDevelop: 'Print preview (full develop)',
    rwDng: 'DNG Converter: RAW → DNG only *',
    rwSeconds: 'seconds · lower is faster',
    rwThumbOf: (ms) => `thumbnail ${ms} ms`,
    rwFooter: (n) => [
      `${n} CC0 samples from raw.pixls.us · thumbnails come from the preview inside the file`,
      `* Adobe DNG Converter not measured here: a third-party estimate (1,000 CR3 in 20–30 min); it only converts to another RAW`,
    ],
    bsTitle: 'Best-shot suggestions, step by step',
    bsGroups: (n) => `Groups found, of ${n} labelled by hand`,
    bsPicks: 'Suggested photo acceptable, in labelled groups with a clear best',
    bsFooter: (n, g) => [
      `${n} real photos (people, food, streets, night, sunsets) · ${g} groups labelled by hand`,
      `No group was ever wrong: pair precision 100 % at every step`,
    ],
    gpuTitle: 'One step of a colour slider: the CPU path against the GPU',
    gpuUnit: 'ms a step · lower is better',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'No GPU (SwiftShader)' },
    gpuFooter: (f) => `1600 px preview, 26 MP photos · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 to Rust's render: mean ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'A machine without a GPU keeps the CPU path.',
    photoOpenTitle: 'Opening a photo, then and now',
    photoOpenUnit: 'ms to open a photo with nothing of it in memory, p50 (lower is better)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'budget 100 ms',
    photoOpenFooter: () => `${browse.photos.jpeg} JPEG and ${browse.photos.raw} RAW from one trip (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 opens each photo from a preview prepared in the background; before, each was decoded on demand',
    inflightTitle: 'Export speed by photos worked on at once',
    inflightUnit: 'photos/s, 24–26 MP JPEGs · higher is better',
    inflightRow: (n) => `${n} at once`,
    inflightChosen: 'the app now',
    inflightWhy: 'Why not 8: on a real trip folder with RAW files, 8 was no faster and used 0.2 GB more.',
    inflightFooter: (f) => `22 CC0 photos · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · memory is the peak of the export`,
  },
  cs: {
    lower: 'méně je lépe',
    higher: 'více je lépe',
    budget: 'limit',
    ceiling: 'strop',
    photos: 'fotek',
    rust: 'Fotollection (Rust)',
    web: 'Prohlížeč (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'fotek/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `JPEGy ${meta.megapixels} MP · ${meta.machine} · ${meta.date} · počet fotek ${f(rust.photos, 0)}: medián, počet běhů ${rust.runs}${withBatch && batch ? ` · počet fotek ${f(batch.photos, 0)}: 1 běh` : ''}`,
    budgetsTitle: 'Změřeno proti limitům výkonu',
    open: 'Otevření složky (import + EXIF + první miniatury)',
    openUnit: (n) => `s na 1 000 fotek (změřeno na počtu ${n})`,
    preview: 'Aktualizace náhledu po změně nastavení',
    previewUnit: 'ms, p95',
    export: 'Propustnost exportu',
    exportUnit: (mp) => `fotek/s při ${mp} MP (limit platí pro 24 MP)`,
    memory: 'Špička paměti, největší dávka',
    memoryUnit: 'GB',
    memTitle: 'Špička paměti se drží na stejné úrovni, i když dávka roste',
    memUnit: 'GB, špička RSS: import + miniatury + export',
    compareTitle: (n) => `Stejné fotky (${n}), stejná úloha: dekódování → ořez 2:3 → datum na fotce → JPEG q95`,
    cExport: 'Hromadný export',
    cExportUnit: 'fotek/s',
    cPreview: 'Teplý náhled (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Špička paměti',
    cMemoryUnit: 'GB',
    previewTitle: 'Kam jde čas náhledu 1600 px',
    resize: 'ořez + změna velikosti',
    stamp: 'datum na fotce',
    encode: 'kódování JPEG',
    ms: 'ms, p50',
    qTitle: 'Kvalita tisku: co export zachová z fotky',
    qWeb: 'Běžný webový editor',
    qDetail: 'Zachované detaily (JPEGy z fotoaparátu)',
    qChroma: 'Zachovaná sytost barev (Display P3, jako fotky z iPhonu)',
    qDe: 'Barevná chyba, průměr ΔE2000 (JPEGy z fotoaparátu)',
    qPercent: '% originálu · více je lépe',
    qDeUnit: 'méně je lépe',
    qVisible: 'viditelné od 1',
    qFooter: (n, m) =>
      `JPEGy z fotoaparátu: ${n}, verze v Display P3: ${m}, každý pixel porovnaný s originálem · webový editor: bez profilu, 4:2:0, q80, 2000 px`,
    shTitle: 'Fotky pro Facebook: co uvidí diváci, když je Facebook znovu zkomprimuje',
    shUsual: 'Obvyklý způsob',
    shChroma: 'Zachovaná sytost barev (Display P3, jako fotky z iPhonu)',
    shDiff: 'Rozdíl od originálu, 1 − SSIM (JPEGy z fotoaparátu)',
    shSize: 'Velikost souboru na fotku (JPEGy z fotoaparátu)',
    shSource: 'Soubor z fotoaparátu',
    shOriginal: 'Original size',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · méně je lehčí',
    shFooter: (n, m) =>
      `JPEGy z fotoaparátu: ${n}, verze v Display P3: ${m} · obvyklý způsob: 2000 px, q80, bez profilu · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'RAW z fotoaparátu: od souboru na obrazovku',
    rwPerFormat: 'Náhled tisku (plné vyvolání), podle fotoaparátu',
    rwCompare: 'Medián devíti fotoaparátů',
    rwThumb: 'Miniatura (vložený náhled)',
    rwDevelop: 'Náhled tisku (plné vyvolání)',
    rwDng: 'DNG Converter: jen RAW → DNG *',
    rwSeconds: 'sekundy · méně je rychlejší',
    rwThumbOf: (ms) => `miniatura ${ms} ms`,
    rwFooter: (n) => [
      `Vzorky CC0 z raw.pixls.us: ${n} · miniatury pocházejí z náhledu uvnitř souboru`,
      `* Adobe DNG Converter se tu neměřil: odhad třetí strany (1 000 CR3 za 20–30 min); jen převádí na jiný RAW`,
    ],
    bsTitle: 'Návrhy nejlepších snímků, krok za krokem',
    bsGroups: (n) => `Nalezené skupiny z ručně označených (${n})`,
    bsPicks: 'Přijatelná navržená fotka v označených skupinách s jasně nejlepší',
    bsFooter: (n, g) => [
      `Skutečné fotky: ${n} (lidé, jídlo, ulice, noc, západy slunce) · ručně označené skupiny: ${g}`,
      `Žádná skupina nebyla nikdy chybná: přesnost dvojic 100 % v každém kroku`,
    ],
    gpuTitle: 'Jeden krok barevného posuvníku: cesta přes CPU proti GPU',
    gpuUnit: 'ms na krok · méně je lépe',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'Bez GPU (SwiftShader)' },
    gpuFooter: (f) => `Náhled 1600 px, fotky s 26 MP · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 proti vykreslení v Rustu: průměr ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'Počítač bez GPU zůstane u cesty přes CPU.',
    photoOpenTitle: 'Otevření fotky, dřív a teď',
    photoOpenUnit: 'ms na otevření fotky, ze které nic není v paměti, p50 (méně je lépe)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'limit 100 ms',
    photoOpenFooter: () => `JPEG: ${browse.photos.jpeg}, RAW: ${browse.photos.raw}, z jedné cesty (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 otevírá každou fotku z náhledu připraveného na pozadí; dřív se každá dekódovala až na požádání',
    inflightTitle: 'Rychlost exportu podle počtu fotek zpracovávaných najednou',
    inflightUnit: 'fotek/s, JPEGy 24–26 MP · více je lépe',
    inflightRow: (n) => `najednou: ${n}`,
    inflightChosen: 'Fotollection teď',
    inflightWhy: 'Proč ne 8: na skutečné složce z cesty se soubory RAW nebylo 8 o nic rychlejší a spotřebovalo o 0,2 GB víc.',
    inflightFooter: (f) => `22 fotek CC0 · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · paměť je špička exportu`,
  },
  de: {
    lower: 'niedriger ist besser',
    higher: 'höher ist besser',
    budget: 'Budget',
    ceiling: 'Obergrenze',
    photos: 'Fotos',
    rust: 'Fotollection (Rust)',
    web: 'Browser (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'Fotos/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `JPEGs mit ${meta.megapixels} MP · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} Fotos: Median aus ${rust.runs} Läufen${withBatch && batch ? ` · ${f(batch.photos, 0)} Fotos: 1 Lauf` : ''}`,
    budgetsTitle: 'Gemessen an den Leistungsbudgets',
    open: 'Ordner öffnen (Import + EXIF + erste Vorschaubilder)',
    openUnit: (n) => `s pro 1.000 Fotos (gemessen an ${n})`,
    preview: 'Vorschau nach einer geänderten Einstellung',
    previewUnit: 'ms, p95',
    export: 'Exportdurchsatz',
    exportUnit: (mp) => `Fotos/s bei ${mp} MP (Budget gilt für 24 MP)`,
    memory: 'Spitzenspeicher, größter Stapel',
    memoryUnit: 'GB',
    memTitle: 'Der Spitzenspeicher bleibt flach, während der Stapel wächst',
    memUnit: 'GB Spitzen-RSS: Import + Vorschaubilder + Export',
    compareTitle: (n) => `Dieselben ${n} Fotos, dieselbe Aufgabe: Dekodieren → Zuschnitt 2:3 → Datumsstempel → JPEG q95`,
    cExport: 'Stapelexport',
    cExportUnit: 'Fotos/s',
    cPreview: 'Warme Vorschau (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Spitzenspeicher',
    cMemoryUnit: 'GB',
    previewTitle: 'Wohin die Zeit der 1600-px-Vorschau geht',
    resize: 'Zuschnitt + Skalieren',
    stamp: 'Datumsstempel',
    encode: 'JPEG-Kodierung',
    ms: 'ms, p50',
    qTitle: 'Druckqualität: was ein Export vom Foto behält',
    qWeb: 'Typischer Web-Editor',
    qDetail: 'Erhaltene Details (Kamera-JPEGs)',
    qChroma: 'Erhaltene Farbsättigung (Display P3, wie iPhone-Fotos)',
    qDe: 'Farbfehler, ΔE2000-Mittel (Kamera-JPEGs)',
    qPercent: '% des Originals · höher ist besser',
    qDeUnit: 'niedriger ist besser',
    qVisible: 'sichtbar ab 1',
    qFooter: (n, m) =>
      `${n} Kamera-JPEGs und ${m} Display-P3-Versionen, jedes Pixel mit dem Original verglichen · Web-Editor: Profil verworfen, 4:2:0, q80, 2000 px`,
    shTitle: 'Fotos für Facebook: was Betrachter sehen, nachdem Facebook sie neu komprimiert hat',
    shUsual: 'Üblicher Weg',
    shChroma: 'Erhaltene Farbsättigung (Display P3, wie iPhone-Fotos)',
    shDiff: 'Abweichung vom Original, 1 − SSIM (Kamera-JPEGs)',
    shSize: 'Dateigröße pro Foto (Kamera-JPEGs)',
    shSource: 'Kameradatei',
    shOriginal: 'Originalgröße',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · niedriger ist leichter',
    shFooter: (n, m) =>
      `${n} Kamera-JPEGs und ${m} Display-P3-Versionen · üblicher Weg: 2000 px, q80, Profil verworfen · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'Kamera-RAW: von der Datei auf den Bildschirm',
    rwPerFormat: 'Druckvorschau (volle Entwicklung), pro Kamera',
    rwCompare: 'Median der neun Kameras',
    rwThumb: 'Vorschaubild (eingebettete Vorschau)',
    rwDevelop: 'Druckvorschau (volle Entwicklung)',
    rwDng: 'DNG Converter: nur RAW → DNG *',
    rwSeconds: 'Sekunden · niedriger ist schneller',
    rwThumbOf: (ms) => `Vorschaubild ${ms} ms`,
    rwFooter: (n) => [
      `${n} CC0-Beispiele von raw.pixls.us · Vorschaubilder stammen aus der Vorschau in der Datei`,
      `* Adobe DNG Converter hier nicht gemessen: eine Schätzung Dritter (1.000 CR3 in 20–30 min); er wandelt nur in ein anderes RAW um`,
    ],
    bsTitle: 'Vorschläge für die besten Aufnahmen, Schritt für Schritt',
    bsGroups: (n) => `Gefundene Gruppen, von ${n} von Hand markierten`,
    bsPicks: 'Vorgeschlagenes Foto akzeptabel, in markierten Gruppen mit klarem besten',
    bsFooter: (n, g) => [
      `${n} echte Fotos (Menschen, Essen, Straßen, Nacht, Sonnenuntergänge) · ${g} Gruppen von Hand markiert`,
      `Keine Gruppe war je falsch: Paarpräzision 100 % in jedem Schritt`,
    ],
    gpuTitle: 'Ein Schritt eines Farbreglers: der CPU-Weg gegen die GPU',
    gpuUnit: 'ms pro Schritt · niedriger ist besser',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'Keine GPU (SwiftShader)' },
    gpuFooter: (f) => `1600-px-Vorschau, Fotos mit 26 MP · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 zur Berechnung von Rust: Mittel ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'Ein Rechner ohne GPU bleibt beim CPU-Weg.',
    photoOpenTitle: 'Ein Foto öffnen, damals und heute',
    photoOpenUnit: 'ms, um ein Foto zu öffnen, von dem nichts im Speicher liegt, p50 (niedriger ist besser)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'Budget 100 ms',
    photoOpenFooter: () => `${browse.photos.jpeg} JPEG und ${browse.photos.raw} RAW von einer Reise (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 öffnet jedes Foto aus einer im Hintergrund vorbereiteten Vorschau; vorher wurde jedes bei Bedarf dekodiert',
    inflightTitle: 'Exporttempo nach Zahl der gleichzeitig bearbeiteten Fotos',
    inflightUnit: 'Fotos/s, JPEGs mit 24–26 MP · höher ist besser',
    inflightRow: (n) => `${n} gleichzeitig`,
    inflightChosen: 'Fotollection jetzt',
    inflightWhy: 'Warum nicht 8: In einem echten Reiseordner mit RAW-Dateien war 8 nicht schneller und brauchte 0,2 GB mehr.',
    inflightFooter: (f) => `22 CC0-Fotos · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · Speicher ist die Spitze des Exports`,
  },
  hu: {
    lower: 'a kisebb a jobb',
    higher: 'a nagyobb a jobb',
    budget: 'keret',
    ceiling: 'felső határ',
    photos: 'fotó',
    rust: 'Fotollection (Rust)',
    web: 'Böngésző (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'fotó/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `${meta.megapixels} MP-es JPEG-ek · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} fotó: ${rust.runs} futás mediánja${withBatch && batch ? ` · ${f(batch.photos, 0)} fotó: 1 futás` : ''}`,
    budgetsTitle: 'A teljesítménykeretekhez mérve',
    open: 'Mappa megnyitása (importálás + EXIF + első bélyegképek)',
    openUnit: (n) => `s 1000 fotónként (mérve: ${n} fotón)`,
    preview: 'Előnézet frissítése egy beállítás módosítása után',
    previewUnit: 'ms, p95',
    export: 'Exportálási teljesítmény',
    exportUnit: (mp) => `fotó/s, ${mp} MP (a keret 24 MP-re szól)`,
    memory: 'Memóriacsúcs, a legnagyobb köteg',
    memoryUnit: 'GB',
    memTitle: 'A memóriacsúcs egyenletes marad, ahogy a köteg nő',
    memUnit: 'GB RSS-csúcs: importálás + bélyegképek + exportálás',
    compareTitle: (n) => `Ugyanazok a fotók (${n}), ugyanaz a feladat: dekódolás → vágás 2:3-ra → dátum → JPEG q95`,
    cExport: 'Kötegelt exportálás',
    cExportUnit: 'fotó/s',
    cPreview: 'Meleg előnézet (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Memóriacsúcs',
    cMemoryUnit: 'GB',
    previewTitle: 'Mire megy el az 1600 px-es előnézet ideje',
    resize: 'vágás + átméretezés',
    stamp: 'dátum',
    encode: 'JPEG-kódolás',
    ms: 'ms, p50',
    qTitle: 'Nyomtatási minőség: mit tart meg az exportálás a fotóból',
    qWeb: 'Tipikus webes szerkesztő',
    qDetail: 'Megtartott részletek (gépes JPEG-ek)',
    qChroma: 'Megtartott színtelítettség (Display P3, mint az iPhone-fotók)',
    qDe: 'Színhiba, ΔE2000 átlag (gépes JPEG-ek)',
    qPercent: 'az eredeti %-a · a nagyobb a jobb',
    qDeUnit: 'a kisebb a jobb',
    qVisible: '1-nél látható',
    qFooter: (n, m) =>
      `${n} gépes JPEG és ${m} Display P3-változat, minden pixel összevetve az eredetivel · webes szerkesztő: profil elhagyva, 4:2:0, q80, 2000 px`,
    shTitle: 'Fotók Facebookra: amit a nézők látnak, miután a Facebook újratömörítette őket',
    shUsual: 'Szokásos mód',
    shChroma: 'Megtartott színtelítettség (Display P3, mint az iPhone-fotók)',
    shDiff: 'Eltérés az eredetitől, 1 − SSIM (gépes JPEG-ek)',
    shSize: 'Fájlméret fotónként (gépes JPEG-ek)',
    shSource: 'Gépes fájl',
    shOriginal: 'Original size',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · a kisebb a könnyebb',
    shFooter: (n, m) =>
      `${n} gépes JPEG és ${m} Display P3-változat · szokásos mód: 2000 px, q80, profil elhagyva · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'Fényképezőgépes RAW: a fájltól a képernyőig',
    rwPerFormat: 'Nyomtatási előnézet (teljes kidolgozás), gépenként',
    rwCompare: 'A kilenc gép mediánja',
    rwThumb: 'Bélyegkép (beágyazott előnézet)',
    rwDevelop: 'Nyomtatási előnézet (teljes kidolgozás)',
    rwDng: 'DNG Converter: csak RAW → DNG *',
    rwSeconds: 'másodperc · a kisebb a gyorsabb',
    rwThumbOf: (ms) => `bélyegkép: ${ms} ms`,
    rwFooter: (n) => [
      `${n} CC0 minta a raw.pixls.us oldalról · a bélyegképek a fájlba ágyazott előnézetből jönnek`,
      `* Az Adobe DNG Convertert itt nem mértük: külső becslés (1000 CR3 20–30 perc alatt); csak másik RAW-vá alakít`,
    ],
    bsTitle: 'A legjobb kép ajánlásai, lépésről lépésre',
    bsGroups: (n) => `Megtalált csoportok (kézzel címkézve: ${n})`,
    bsPicks: 'Elfogadható ajánlott fotó, a kézzel címkézett, egyértelmű legjobbal bíró csoportokban',
    bsFooter: (n, g) => [
      `${n} valódi fotó (emberek, ételek, utcák, éjszaka, naplementék) · ${g} kézzel címkézett csoport`,
      `Egyetlen csoport sem volt hibás: a párpontosság minden lépésben 100 %`,
    ],
    gpuTitle: 'Egy színcsúszka egy lépése: a CPU-s út a GPU-val szemben',
    gpuUnit: 'ms lépésenként · a kisebb a jobb',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'GPU nélkül (SwiftShader)' },
    gpuFooter: (f) => `1600 px-es előnézet, 26 MP-es fotók · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 a Rust renderjéhez képest: átlag ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'GPU nélküli gépen a CPU-s út marad.',
    photoOpenTitle: 'Egy fotó megnyitása, akkor és most',
    photoOpenUnit: 'ms egy olyan fotó megnyitásáig, amelyből semmi sincs a memóriában, p50 (a kisebb a jobb)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'keret: 100 ms',
    photoOpenFooter: () => `${browse.photos.jpeg} JPEG és ${browse.photos.raw} RAW egy útról (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'A v0.11.0 minden fotót egy háttérben előkészített előnézetből nyit meg; korábban mindegyiket igény szerint dekódolta',
    inflightTitle: 'Exportálási sebesség az egyszerre feldolgozott fotók szerint',
    inflightUnit: 'fotó/s, 24–26 MP-es JPEG-ek · a nagyobb a jobb',
    inflightRow: (n) => `${n} egyszerre`,
    inflightChosen: 'Fotollection most',
    inflightWhy: 'Miért nem 8: egy valódi, RAW fájlokat is tartalmazó úti mappán a 8 nem volt gyorsabb, és 0,2 GB-tal több memóriát használt.',
    inflightFooter: (f) => `22 CC0 fotó · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · a memória az exportálás csúcsa`,
  },
  pl: {
    lower: 'mniej znaczy lepiej',
    higher: 'więcej znaczy lepiej',
    budget: 'budżet',
    ceiling: 'pułap',
    photos: 'zdjęć',
    rust: 'Fotollection (Rust)',
    web: 'Przeglądarka (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'zdj./s', GB: 'GB' },
    footer: (f, withBatch) =>
      `JPEG-i ${meta.megapixels} MP · ${meta.machine} · ${meta.date} · zdjęcia: ${f(rust.photos, 0)}, mediana (przebiegi: ${rust.runs})${withBatch && batch ? ` · zdjęcia: ${f(batch.photos, 0)}, 1 przebieg` : ''}`,
    budgetsTitle: 'Zmierzone wobec budżetów wydajności',
    open: 'Otwarcie folderu (import + EXIF + pierwsze miniatury)',
    openUnit: (n) => `s na 1000 zdjęć (zmierzono przy n = ${n})`,
    preview: 'Odświeżenie podglądu po zmianie ustawienia',
    previewUnit: 'ms, p95',
    export: 'Przepustowość eksportu',
    exportUnit: (mp) => `zdj./s przy ${mp} MP (budżet dotyczy 24 MP)`,
    memory: 'Szczytowe zużycie pamięci, największa partia',
    memoryUnit: 'GB',
    memTitle: 'Szczytowe zużycie pamięci stoi w miejscu, gdy partia rośnie',
    memUnit: 'GB szczytowego RSS: import + miniatury + eksport',
    compareTitle: (n) => `Te same zdjęcia (${n}), to samo zadanie: dekodowanie → kadr 2:3 → data na zdjęciu → JPEG q95`,
    cExport: 'Eksport wsadowy',
    cExportUnit: 'zdj./s',
    cPreview: 'Ciepły podgląd (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'Szczytowe zużycie pamięci',
    cMemoryUnit: 'GB',
    previewTitle: 'Na co idzie czas podglądu 1600 px',
    resize: 'kadr + skalowanie',
    stamp: 'data na zdjęciu',
    encode: 'kodowanie JPEG',
    ms: 'ms, p50',
    qTitle: 'Jakość druku: co eksport zachowuje ze zdjęcia',
    qWeb: 'Typowy edytor internetowy',
    qDetail: 'Zachowane szczegóły (JPEG-i z aparatu)',
    qChroma: 'Zachowane nasycenie kolorów (Display P3, jak zdjęcia z iPhone\'a)',
    qDe: 'Błąd koloru, średnia ΔE2000 (JPEG-i z aparatu)',
    qPercent: '% oryginału · więcej znaczy lepiej',
    qDeUnit: 'mniej znaczy lepiej',
    qVisible: 'widoczne przy 1',
    qFooter: (n, m) =>
      `JPEG-i z aparatu: ${n}, wersje Display P3: ${m}, każdy piksel porównany z oryginałem · edytor internetowy: profil usunięty, 4:2:0, q80, 2000 px`,
    shTitle: 'Zdjęcia na Facebooka: co widzą oglądający po ponownej kompresji przez Facebooka',
    shUsual: 'Zwykły sposób',
    shChroma: 'Zachowane nasycenie kolorów (Display P3, jak zdjęcia z iPhone\'a)',
    shDiff: 'Różnica od oryginału, 1 − SSIM (JPEG-i z aparatu)',
    shSize: 'Rozmiar pliku na zdjęcie (JPEG-i z aparatu)',
    shSource: 'Plik z aparatu',
    shOriginal: 'Oryginalny rozmiar',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · mniej znaczy lżej',
    shFooter: (n, m) =>
      `JPEG-i z aparatu: ${n}, wersje Display P3: ${m} · zwykły sposób: 2000 px, q80, profil usunięty · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'RAW z aparatu: od pliku do ekranu',
    rwPerFormat: 'Podgląd do druku (pełne wywołanie), dla każdego aparatu',
    rwCompare: 'Mediana dziewięciu aparatów',
    rwThumb: 'Miniatura (wbudowany podgląd)',
    rwDevelop: 'Podgląd do druku (pełne wywołanie)',
    rwDng: 'DNG Converter: tylko RAW → DNG *',
    rwSeconds: 'sekundy · mniej znaczy szybciej',
    rwThumbOf: (ms) => `miniatura ${ms} ms`,
    rwFooter: (n) => [
      `Próbki CC0 z raw.pixls.us: ${n} · miniatury pochodzą z podglądu wewnątrz pliku`,
      `* Adobe DNG Converter nie był tu mierzony: szacunek z zewnętrznego źródła (1000 CR3 w 20–30 min); tylko konwertuje do innego RAW`,
    ],
    bsTitle: 'Podpowiedzi najlepszych ujęć, krok po kroku',
    bsGroups: (n) => `Znalezione grupy, spośród ${n} oznaczonych ręcznie`,
    bsPicks: 'Akceptowalne podpowiedziane zdjęcie w oznaczonych grupach z wyraźnie najlepszym',
    bsFooter: (n, g) => [
      `Prawdziwe zdjęcia: ${n} (ludzie, jedzenie, ulice, noc, zachody słońca) · grupy oznaczone ręcznie: ${g}`,
      `Żadna grupa nigdy nie była błędna: precyzja par 100 % na każdym kroku`,
    ],
    gpuTitle: 'Jeden krok suwaka koloru: ścieżka CPU wobec GPU',
    gpuUnit: 'ms na krok · mniej znaczy lepiej',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'Bez GPU (SwiftShader)' },
    gpuFooter: (f) => `Podgląd 1600 px, zdjęcia 26 MP · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 względem renderu Rusta: średnio ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'Komputer bez GPU zostaje przy ścieżce CPU.',
    photoOpenTitle: 'Otwieranie zdjęcia kiedyś i dziś',
    photoOpenUnit: 'ms na otwarcie zdjęcia, gdy nic z niego nie ma w pamięci, p50 (mniej znaczy lepiej)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'budżet 100 ms',
    photoOpenFooter: () => `JPEG: ${browse.photos.jpeg}, RAW: ${browse.photos.raw}, z jednej podróży (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 otwiera każde zdjęcie z podglądu przygotowanego w tle; wcześniej każde było dekodowane na żądanie',
    inflightTitle: 'Szybkość eksportu a liczba zdjęć przetwarzanych naraz',
    inflightUnit: 'zdj./s, JPEG-i 24–26 MP · więcej znaczy lepiej',
    inflightRow: (n) => `${n} naraz`,
    inflightChosen: 'Fotollection teraz',
    inflightWhy: 'Dlaczego nie 8: na prawdziwym folderze z podróży z plikami RAW 8 nie było szybsze i zużywało 0,2 GB więcej.',
    inflightFooter: (f) => `22 zdjęcia CC0 · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · pamięć to szczyt podczas eksportu`,
  },
  vi: {
    lower: 'càng thấp càng tốt',
    higher: 'càng cao càng tốt',
    budget: 'ngân sách',
    ceiling: 'trần',
    photos: 'ảnh',
    rust: 'Fotollection (Rust)',
    web: 'Trình duyệt (Chromium)',
    webShort: 'Chromium',
    units: { s: 's', ms: 'ms', 'photos/s': 'ảnh/s', GB: 'GB' },
    footer: (f, withBatch) =>
      `JPEG ${meta.megapixels} MP · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} ảnh: trung vị của ${rust.runs} lần chạy${withBatch && batch ? ` · ${f(batch.photos, 0)} ảnh: 1 lần` : ''}`,
    budgetsTitle: 'Số đo so với ngân sách hiệu năng',
    open: 'Mở thư mục (import + EXIF + thumbnail đầu tiên)',
    openUnit: (n) => `giây cho 1.000 ảnh (đo trên ${n} ảnh)`,
    preview: 'Cập nhật preview khi đổi thiết lập',
    previewUnit: 'ms, p95',
    export: 'Tốc độ export',
    exportUnit: (mp) => `ảnh/s với ảnh ${mp} MP (ngân sách tính cho 24 MP)`,
    memory: 'RAM đỉnh, batch lớn nhất',
    memoryUnit: 'GB',
    memTitle: 'RAM đỉnh giữ nguyên khi batch tăng',
    memUnit: 'GB RSS đỉnh: import + thumbnail + export',
    compareTitle: (n) => `Cùng ${n} ảnh, cùng việc: giải mã → crop 2:3 → dấu ngày → JPEG q95`,
    cExport: 'Export cả lô',
    cExportUnit: 'ảnh/s',
    cPreview: 'Preview đã cache (1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: 'RAM đỉnh',
    cMemoryUnit: 'GB',
    previewTitle: 'Thời gian preview 1600 px đi đâu',
    resize: 'crop + resize',
    stamp: 'dấu ngày',
    encode: 'mã hóa JPEG',
    ms: 'ms, p50',
    qTitle: 'Chất lượng in: bản xuất giữ lại được bao nhiêu',
    qWeb: 'Web editor thông thường',
    qDetail: 'Chi tiết giữ lại (ảnh JPEG máy ảnh)',
    qChroma: 'Độ bão hoà màu giữ lại (Display P3, như ảnh iPhone)',
    qDe: 'Sai lệch màu, ΔE2000 trung bình (ảnh JPEG máy ảnh)',
    qPercent: '% so với ảnh gốc · càng cao càng tốt',
    qDeUnit: 'càng thấp càng tốt',
    qVisible: 'thấy được từ 1',
    qFooter: (n, m) =>
      `${n} ảnh JPEG máy ảnh và ${m} bản Display P3, so từng điểm ảnh với ảnh gốc · web editor: bỏ profile, 4:2:0, q80, 2000 px`,
    shTitle: 'Ảnh đăng Facebook: người xem thấy gì sau khi Facebook nén lại',
    shUsual: 'Cách thông thường',
    shChroma: 'Độ bão hoà màu giữ lại (Display P3, như ảnh iPhone)',
    shDiff: 'Khác biệt so với ảnh gốc, 1 − SSIM (ảnh JPEG máy ảnh)',
    shSize: 'Dung lượng mỗi ảnh (ảnh JPEG máy ảnh)',
    shSource: 'File máy ảnh',
    shOriginal: 'Giữ cỡ gốc',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · càng thấp càng nhẹ',
    shFooter: (n, m) =>
      `${n} ảnh JPEG máy ảnh và ${m} bản Display P3 · cách thông thường: 2000 px, q80, bỏ profile · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: 'Ảnh RAW: từ tệp đến màn hình',
    rwPerFormat: 'Xem trước để in (giải mã đầy đủ), theo từng máy',
    rwCompare: 'Trung vị của chín máy',
    rwThumb: 'Ảnh thu nhỏ (bản xem trước nhúng)',
    rwDevelop: 'Xem trước để in (giải mã đầy đủ)',
    rwDng: 'DNG Converter: chỉ RAW → DNG *',
    rwSeconds: 'giây · càng thấp càng nhanh',
    rwThumbOf: (ms) => `ảnh thu nhỏ ${ms} ms`,
    rwFooter: (n) => [
      `${n} tệp mẫu CC0 từ raw.pixls.us · ảnh thu nhỏ lấy từ bản xem trước có sẵn trong tệp`,
      `* Adobe DNG Converter không đo ở đây: ước tính bên ngoài (1.000 tệp CR3 trong 20–30 phút), chỉ đổi sang RAW khác`,
    ],
    bsTitle: 'Gợi ý ảnh đẹp nhất, từng bước cải tiến',
    bsGroups: (n) => `Số nhóm tìm được, trên ${n} nhóm gán nhãn tay`,
    bsPicks: 'Ảnh được gợi ý chấp nhận được, ở các nhóm có ảnh tốt nhất rõ ràng',
    bsFooter: (n, g) => [
      `${n} ảnh thật (người, đồ ăn, phố, cảnh đêm, hoàng hôn) · ${g} nhóm gán nhãn tay`,
      `Không lần nào gom sai: độ chính xác theo cặp 100 % ở mọi bước`,
    ],
    gpuTitle: 'Một nhịp kéo thanh màu: đường CPU so với GPU',
    gpuUnit: 'ms mỗi nhịp · càng thấp càng tốt',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'Không có GPU (SwiftShader)' },
    gpuFooter: (f) => `Bản xem trước 1600 px, ảnh 26 MP · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · ΔE2000 so với Rust: trung bình ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'Máy không có GPU vẫn dùng đường CPU.',
    photoOpenTitle: 'Mở một tấm ảnh, trước và nay',
    photoOpenUnit: 'ms để mở một ảnh chưa có gì trong bộ nhớ, trung vị (thấp hơn là tốt hơn)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: 'ngân sách 100 ms',
    photoOpenFooter: () => `${browse.photos.jpeg} JPEG và ${browse.photos.raw} RAW của một chuyến đi (Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 mở mỗi ảnh từ bản xem trước chuẩn bị sẵn ở chế độ nền; trước đó mỗi ảnh được giải mã khi cần',
    inflightTitle: 'Tốc độ xuất theo số ảnh xử lý cùng lúc',
    inflightUnit: 'ảnh/giây, JPEG 24–26 MP · càng cao càng tốt',
    inflightRow: (n) => `${n} ảnh cùng lúc`,
    inflightChosen: 'app hiện dùng',
    inflightWhy: 'Vì sao không 8: với thư mục thật của một chuyến đi có ảnh RAW, 8 không nhanh hơn mà tốn thêm 0,2 GB.',
    inflightFooter: (f) => `22 ảnh CC0 · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · RAM là mức đỉnh khi xuất`,
  },
  ja: {
    lower: '小さいほど良い',
    higher: '大きいほど良い',
    budget: '目標値',
    ceiling: '上限',
    photos: '枚',
    rust: 'Fotollection (Rust)',
    web: 'ブラウザー (Chromium)',
    webShort: 'Chromium',
    units: { s: '秒', ms: 'ms', 'photos/s': '枚/秒', GB: 'GB' },
    footer: (f, withBatch) =>
      `${meta.megapixels} MP の JPEG · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)} 枚：${rust.runs} 回の中央値${withBatch && batch ? ` · ${f(batch.photos, 0)} 枚：1 回` : ''}`,
    budgetsTitle: '性能目標と比べた実測値',
    open: 'フォルダーを開く（読み込み＋EXIF＋最初のサムネイル）',
    openUnit: (n) => `1,000 枚あたりの秒数（${n} 枚で測定）`,
    preview: '設定を変えた後のプレビュー更新',
    previewUnit: 'ms、p95',
    export: '書き出しの速さ',
    exportUnit: (mp) => `${mp} MP での枚/秒（目標値は 24 MP 基準）`,
    memory: 'メモリーのピーク、最大のバッチ',
    memoryUnit: 'GB',
    memTitle: 'バッチが大きくなってもメモリーのピークは横ばい',
    memUnit: 'GB、RSS のピーク：読み込み＋サムネイル＋書き出し',
    compareTitle: (n) => `同じ ${n} 枚、同じ処理：デコード → 2:3 にトリミング → 日付 → JPEG q95`,
    cExport: '一括書き出し',
    cExportUnit: '枚/秒',
    cPreview: 'ウォームプレビュー（1600 px）',
    cPreviewUnit: 'ms、p50',
    cMemory: 'メモリーのピーク',
    cMemoryUnit: 'GB',
    previewTitle: '1600 px プレビューの時間の内訳',
    resize: 'トリミング＋リサイズ',
    stamp: '日付',
    encode: 'JPEG エンコード',
    ms: 'ms、p50',
    qTitle: 'プリント品質：書き出しが写真から残すもの',
    qWeb: '一般的な Web 編集ツール',
    qDetail: '残った細部（カメラの JPEG）',
    qChroma: '残った彩度（Display P3、iPhone の写真と同じ）',
    qDe: '色の誤差、ΔE2000 の平均（カメラの JPEG）',
    qPercent: '元の写真に対する % · 大きいほど良い',
    qDeUnit: '小さいほど良い',
    qVisible: '1 で見える',
    qFooter: (n, m) =>
      `カメラの JPEG ${n} 枚と Display P3 版 ${m} 枚、全ピクセルを元の写真と比較 · Web 編集ツール：プロファイル削除、4:2:0、q80、2000 px`,
    shTitle: 'Facebook 用の写真：Facebook が再圧縮した後に見える姿',
    shUsual: '一般的な方法',
    shChroma: '残った彩度（Display P3、iPhone の写真と同じ）',
    shDiff: '元の写真との差、1 − SSIM（カメラの JPEG）',
    shSize: '1 枚あたりのファイルサイズ（カメラの JPEG）',
    shSource: 'カメラのファイル',
    shOriginal: 'Original size',
    shFacebook: 'Facebook、2048 px',
    shMb: 'MB · 小さいほど軽い',
    shFooter: (n, m) =>
      `カメラの JPEG ${n} 枚と Display P3 版 ${m} 枚 · 一般的な方法：2000 px、q80、プロファイル削除 · Facebook：q71、4:2:0、≤ 2048 px`,
    rwTitle: 'カメラの RAW：ファイルから画面まで',
    rwPerFormat: 'プリントプレビュー（フル現像）、カメラ別',
    rwCompare: '9 機種の中央値',
    rwThumb: 'サムネイル（埋め込みプレビュー）',
    rwDevelop: 'プリントプレビュー（フル現像）',
    rwDng: 'DNG Converter：RAW → DNG のみ *',
    rwSeconds: '秒 · 小さいほど速い',
    rwThumbOf: (ms) => `サムネイル ${ms} ms`,
    rwFooter: (n) => [
      `raw.pixls.us の CC0 サンプル ${n} 点 · サムネイルはファイル内のプレビューから`,
      `* Adobe DNG Converter はここでは未測定：第三者の見積もり（CR3 1,000 枚で 20〜30 分）。別の RAW に変換するだけです`,
    ],
    bsTitle: 'ベストショットの提案、ステップごと',
    bsGroups: (n) => `見つかったグループ（手作業でラベル付けした ${n} 件中）`,
    bsPicks: '提案された写真が妥当だった数（明確なベストがあるラベル付きグループ）',
    bsFooter: (n, g) => [
      `実際の写真 ${n} 枚（人物、料理、街並み、夜景、夕焼け） · 手作業でラベル付けしたグループ ${g} 件`,
      `誤ったグループは一度もなし：どのステップでもペア精度 100%`,
    ],
    gpuTitle: 'カラースライダーの 1 ステップ：CPU 経路と GPU の比較',
    gpuUnit: '1 ステップの ms · 小さいほど良い',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU、macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU、Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'GPU なし (SwiftShader)' },
    gpuFooter: (f) => `1600 px プレビュー、26 MP の写真 · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · Rust のレンダリングとの ΔE2000：平均 ${f(gpu.parity.webkit.photoMean, 2)}、p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'GPU のないマシンでは CPU 経路のままです。',
    photoOpenTitle: '写真を開く速さ、以前と今',
    photoOpenUnit: 'メモリーに何も残っていない写真を開くまでの ms、p50（小さいほど良い）',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: '目標値 100 ms',
    photoOpenFooter: () => `1 回の旅行の JPEG ${browse.photos.jpeg} 枚と RAW ${browse.photos.raw} 枚（Lumix GX9、20 MP） · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0 はバックグラウンドで用意したプレビューから写真を開きます。以前は開くたびにデコードしていました',
    inflightTitle: '同時に処理する枚数ごとの書き出し速度',
    inflightUnit: '枚/秒、24〜26 MP の JPEG · 大きいほど良い',
    inflightRow: (n) => `同時に ${n} 枚`,
    inflightChosen: '現在のアプリ',
    inflightWhy: '8 枚にしない理由：RAW を含む実際の旅行フォルダーでは、8 枚は速くならず、メモリーを 0.2 GB 多く使いました。',
    inflightFooter: (f) => `CC0 の写真 22 枚 · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · メモリーは書き出し中のピーク`,
  },
  ko: {
    lower: '작을수록 좋음',
    higher: '클수록 좋음',
    budget: '목표치',
    ceiling: '상한',
    photos: '장',
    rust: 'Fotollection (Rust)',
    web: '브라우저 (Chromium)',
    webShort: 'Chromium',
    units: { s: '초', ms: 'ms', 'photos/s': '장/초', GB: 'GB' },
    footer: (f, withBatch) =>
      `${meta.megapixels} MP JPEG · ${meta.machine} · ${meta.date} · ${f(rust.photos, 0)}장: ${rust.runs}회의 중앙값${withBatch && batch ? ` · ${f(batch.photos, 0)}장: 1회` : ''}`,
    budgetsTitle: '성능 목표 대비 실측값',
    open: '폴더 열기(읽기 + EXIF + 첫 썸네일)',
    openUnit: (n) => `1,000장당 초(${n}장으로 측정)`,
    preview: '설정 변경 후 미리보기 갱신',
    previewUnit: 'ms, p95',
    export: '내보내기 속도',
    exportUnit: (mp) => `${mp} MP에서 장/초(목표치는 24 MP 기준)`,
    memory: '메모리 최대치, 가장 큰 배치',
    memoryUnit: 'GB',
    memTitle: '배치가 커져도 메모리 최대치는 그대로',
    memUnit: 'GB, RSS 최대치: 읽기 + 썸네일 + 내보내기',
    compareTitle: (n) => `같은 ${n}장, 같은 처리: 디코딩 → 2:3 자르기 → 날짜 → JPEG q95`,
    cExport: '일괄 내보내기',
    cExportUnit: '장/초',
    cPreview: '웜 미리보기(1600 px)',
    cPreviewUnit: 'ms, p50',
    cMemory: '메모리 최대치',
    cMemoryUnit: 'GB',
    previewTitle: '1600 px 미리보기의 시간 내역',
    resize: '자르기 + 크기 조정',
    stamp: '날짜',
    encode: 'JPEG 인코딩',
    ms: 'ms, p50',
    qTitle: '인화 품질: 내보내기가 사진에서 지키는 것',
    qWeb: '일반적인 웹 편집 도구',
    qDetail: '남은 디테일(카메라 JPEG)',
    qChroma: '남은 채도(Display P3, iPhone 사진과 같음)',
    qDe: '색 오차, ΔE2000 평균(카메라 JPEG)',
    qPercent: '원본 사진 대비 % · 클수록 좋음',
    qDeUnit: '작을수록 좋음',
    qVisible: '1이면 보임',
    qFooter: (n, m) =>
      `카메라 JPEG ${n}장과 Display P3 버전 ${m}장, 모든 픽셀을 원본 사진과 비교 · 웹 편집 도구: 프로필 삭제, 4:2:0, q80, 2000 px`,
    shTitle: 'Facebook용 사진: Facebook이 재압축한 뒤의 모습',
    shUsual: '일반적인 방법',
    shChroma: '남은 채도(Display P3, iPhone 사진과 같음)',
    shDiff: '원본 사진과의 차이, 1 − SSIM(카메라 JPEG)',
    shSize: '한 장당 파일 크기(카메라 JPEG)',
    shSource: '카메라 파일',
    shOriginal: 'Original size',
    shFacebook: 'Facebook, 2048 px',
    shMb: 'MB · 작을수록 가벼움',
    shFooter: (n, m) =>
      `카메라 JPEG ${n}장과 Display P3 버전 ${m}장 · 일반적인 방법: 2000 px, q80, 프로필 삭제 · Facebook: q71, 4:2:0, ≤ 2048 px`,
    rwTitle: '카메라 RAW: 파일에서 화면까지',
    rwPerFormat: '인화 미리보기(전체 현상), 카메라별',
    rwCompare: '9개 기종의 중앙값',
    rwThumb: '썸네일(내장 미리보기)',
    rwDevelop: '인화 미리보기(전체 현상)',
    rwDng: 'DNG Converter: RAW → DNG만 *',
    rwSeconds: '초 · 작을수록 빠름',
    rwThumbOf: (ms) => `썸네일 ${ms} ms`,
    rwFooter: (n) => [
      `raw.pixls.us의 CC0 샘플 ${n}개 · 썸네일은 파일 안의 미리보기에서`,
      `* Adobe DNG Converter는 여기서 측정하지 않음: 제3자 추정치(CR3 1,000장에 20~30분). 다른 RAW로 변환할 뿐입니다`,
    ],
    bsTitle: '베스트 샷 제안, 단계별',
    bsGroups: (n) => `찾은 그룹(손으로 라벨을 붙인 ${n}개 중)`,
    bsPicks: '제안된 사진이 타당했던 수(분명한 베스트가 있는 라벨 그룹)',
    bsFooter: (n, g) => [
      `실제 사진 ${n}장(인물, 음식, 거리, 야경, 노을) · 손으로 라벨을 붙인 그룹 ${g}개`,
      `잘못된 그룹은 한 번도 없음: 모든 단계에서 쌍 정확도 100%`,
    ],
    gpuTitle: '색 슬라이더 한 단계: CPU 경로와 GPU 비교',
    gpuUnit: '한 단계의 ms · 작을수록 좋음',
    gpuCpu: 'CPU (Rust + JPEG)',
    gpuEngines: { 'WebKit (Apple GPU)': 'GPU, macOS (WebKit)', 'Chromium, ANGLE Metal': 'GPU, Chromium (Metal)', 'Chromium, SwiftShader (no GPU)': 'GPU 없음 (SwiftShader)' },
    gpuFooter: (f) => `1600 px 미리보기, 26 MP 사진 · ${gpu.machine.split(',').slice(0, 2).join(',')} · ${gpu.date} · Rust 렌더링과의 ΔE2000: 평균 ${f(gpu.parity.webkit.photoMean, 2)}, p99 ${f(gpu.parity.webkit.photoP99, 2)}`,
    noGpuNote: 'GPU가 없는 컴퓨터에서는 CPU 경로를 그대로 씁니다.',
    photoOpenTitle: '사진 여는 속도, 이전과 지금',
    photoOpenUnit: '메모리에 아무것도 없는 사진을 여는 ms, p50(작을수록 좋음)',
    photoOpenRow: (v, kind) => `${v} · ${kind}`,
    photoOpenBudget: '목표치 100 ms',
    photoOpenFooter: () => `여행 한 번의 JPEG ${browse.photos.jpeg}장과 RAW ${browse.photos.raw}장(Lumix GX9, 20 MP) · ${browse.machine} · ${browse.date}`,
    photoOpenWhy: 'v0.11.0은 백그라운드에서 준비한 미리보기로 사진을 엽니다. 이전에는 열 때마다 디코딩했습니다',
    inflightTitle: '동시에 처리하는 장수별 내보내기 속도',
    inflightUnit: '장/초, 24~26 MP JPEG · 클수록 좋음',
    inflightRow: (n) => `동시에 ${n}장`,
    inflightChosen: '현재 앱',
    inflightWhy: '8장으로 하지 않는 이유: RAW가 섞인 실제 여행 폴더에서 8장은 빨라지지 않고 메모리를 0.2 GB 더 썼습니다.',
    inflightFooter: (f) => `CC0 사진 22장 · ${inflight.machine.split(',').slice(0, 2).join(',')} · ${inflight.date} · 메모리는 내보내기 중 최대치`,
  },
}

// ---------------------------------------------------------------------------------------
// SVG building blocks. Validated palette (dataviz reference): slot 1 blue = Fotollection,
// slot 2 orange = browser, slots 3–5 = preview segments. Light and dark steps are chosen.

const W = 720
const LEFT = 190
const RIGHT = 110
const PLOT = W - LEFT - RIGHT
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;')
// Two themes: the docs' neutral one (README, RESULT), and the website's, taken from the logo
// (site/assets/css/site.css). Both pairs pass the dataviz palette validator on their surface.
const THEMES = {
  docs: {
    light: { bg: '#fcfcfb', t1: '#1f1f1e', t2: '#52514c', t3: '#6f6e68', axis: '#c9c8c1', s1: '#2a78d6', s2: '#eb6834', s3: '#1baf7a', s4: '#eda100', s5: '#e87ba4' },
    dark: { bg: '#1a1a19', t1: '#f0efec', t2: '#c3c2b7', t3: '#9d9c93', axis: '#45443f', s1: '#3987e5', s2: '#d95926', s3: '#199e70', s4: '#c98500', s5: '#d55181' },
  },
  site: {
    light: { bg: '#fffaee', t1: '#2b2219', t2: '#514435', t3: '#7a6a55', axis: '#e2d4b3', s1: '#008a74', s2: '#b5652a', s3: '#1baf7a', s4: '#eda100', s5: '#e87ba4' },
    dark: { bg: '#262019', t1: '#f3ead6', t2: '#d8cab0', t3: '#a8977c', axis: '#3b3127', s1: '#21a593', s2: '#cc7f3e', s3: '#199e70', s4: '#c98500', s5: '#d55181' },
  },
}
const styleFor = ({ light: l, dark: d }) => `<style>
  .bg{fill:${l.bg}} .t1{fill:${l.t1};font:600 13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t2{fill:${l.t2};font:13px system-ui,-apple-system,Segoe UI,sans-serif}
  .t3{fill:${l.t3};font:12px system-ui,-apple-system,Segoe UI,sans-serif}
  .h{font-size:15px} .num{font-variant-numeric:tabular-nums} .axis{stroke:${l.axis};stroke-width:1}
  .ref{stroke:${l.t2};stroke-width:1.5;stroke-dasharray:4 3} .refw{stroke:${l.s2};stroke-width:1.5;stroke-dasharray:4 3}
  .s1{fill:${l.s1}} .old{fill-opacity:.45} .s2{fill:${l.s2}} .s3{fill:${l.s3}} .s4{fill:${l.s4}} .s5{fill:${l.s5}} .gap{stroke:${l.bg};stroke-width:2}
  .in{fill:#fff}
  @media (prefers-color-scheme: dark){
    .bg{fill:${d.bg}} .t1{fill:${d.t1}} .t2{fill:${d.t2}} .t3{fill:${d.t3}} .axis{stroke:${d.axis}}
    .ref{stroke:${d.t2}} .refw{stroke:${d.s2}}
    .s1{fill:${d.s1}} .s2{fill:${d.s2}} .s3{fill:${d.s3}} .s4{fill:${d.s4}} .s5{fill:${d.s5}} .gap{stroke:${d.bg}}
  }
</style>`
let STYLE = styleFor(THEMES.docs)

function doc(title, desc, height, body) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-16 -16 ${W + 32} ${height + 32}" width="${W + 32}" height="${height + 32}" role="img" aria-labelledby="t d">
<title id="t">${esc(title)}</title>
<desc id="d">${esc(desc)}</desc>
${STYLE}
<rect class="bg" x="-16" y="-16" width="${W + 32}" height="${height + 32}" rx="8"/>
<text class="t1 h" x="0" y="8">${esc(title)}</text>
${body}
</svg>
`
}

/** A bar with a 4 px rounded data end, square at the baseline. */
function bar(cls, x, y, w, h, tip) {
  const r = Math.min(4, w / 2)
  return `<path class="${cls}" d="M${x} ${y}h${w - r}a${r} ${r} 0 0 1 ${r} ${r}v${h - 2 * r}a${r} ${r} 0 0 1 ${-r} ${r}h${r - w}z"><title>${esc(tip)}</title></path>`
}

/**
 * A dashed vertical reference line with its label above. `gap` = [top, bottom] leaves the
 * line out where it would cross a bar and its value label.
 */
function refLine(cls, x, y0, y1, label, anchor = 'middle', gap = null) {
  const line = (a, b) => `<line class="${cls}" x1="${x}" x2="${x}" y1="${a}" y2="${b}"/>`
  // A gap that reaches the end leaves only the first part.
  const lines = gap ? line(y0, gap[0]) + (gap[1] < y1 ? line(gap[1], y1) : '') : line(y0, y1)
  return `${lines}<text class="t3" x="${x}" y="${y0 - 4}" text-anchor="${anchor}">${esc(label)}</text>`
}

function charts(lang) {
  const t = T[lang]
  const f = (v, d) =>
    new Intl.NumberFormat(lang, { minimumFractionDigits: d, maximumFractionDigits: d }).format(v)
  const out = {}
  // "< 50 ms" from budgets.json, with the unit and number in this language.
  const budgetText = (b) => `${b.text.split(' ')[0]} ${f(b.value, 0)} ${t.units[b.unit] ?? b.unit}`

  // 1. Budgets: one panel per NFR, the measured bar against a dashed budget line.
  {
    const panels = [
      { title: t.open, unit: t.openUnit(f(measured.openPhotos, 0)), v: measured.openPer1000S, b: budgets['NFR-001'], d: 2 },
      { title: t.preview, unit: t.previewUnit, v: measured.previewP95Ms, b: budgets['NFR-002'], d: 0 },
      { title: t.export, unit: t.exportUnit(meta.megapixels), v: measured.exportPerSecond, b: budgets['NFR-003'], d: 1 },
      { title: t.memory, unit: t.memoryUnit, v: measured.peakGb, b: budgets['NFR-004'], d: 2 },
    ]
    const ph = 84
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 30 + i * ph
      const max = Math.max(p.v, p.b.value) * 1.2
      const x = (v) => LEFT + (PLOT * v) / max
      const ok = p.b.better === 'lower' ? p.v <= p.b.value : p.v >= p.b.value
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="0" y="${y0 + 30}">${esc(p.unit)} · ${esc(p.b.better === 'lower' ? t.lower : t.higher)}</text>`
      const by = y0 + 50
      body += `<text class="t2" x="${LEFT - 10}" y="${by + 16}" text-anchor="end">${esc(t.rust)}</text>`
      body += bar('s1', LEFT, by, Math.max(4, x(p.v) - LEFT), 22, `${p.title}: ${f(p.v, p.d)}`)
      body += `<text class="t1 num" x="${x(p.v) + 8}" y="${by + 16}">${f(p.v, p.d)}${ok ? ' ✓' : ''}</text>`
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${by - 4}" y2="${by + 26}"/>`
      body += refLine('ref', x(p.b.value), by - 4, by + 26, `${t.budget} ${budgetText(p.b)}`)
    })
    const h = 30 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, true))}</text>`
    const desc = panels.map((p) => `${p.title}: ${f(p.v, p.d)} (${t.budget} ${budgetText(p.b)})`).join('. ')
    out['bench-budgets'] = doc(t.budgetsTitle, desc, h, body)
  }

  // 2. Memory vs batch size, against the 2 GB ceiling.
  {
    const rows = [{ n: rust.photos, gb: rust.peakRssBytes / 1e9 }]
    for (const b of [...batches].reverse()) rows.push({ n: b.photos, gb: b.peakRssBytes / 1e9 })
    const ceiling = budgets['NFR-004'].value
    const max = ceiling * 1.2
    const x = (v) => LEFT + (PLOT * v) / max
    let body = `<text class="t3" x="0" y="30">${esc(t.memUnit)} · ${esc(t.lower)}</text>`
    const y0 = 62
    rows.forEach((r, i) => {
      const y = y0 + i * 32
      const label = `${f(r.n, 0)} ${t.photos}`
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(label)}</text>`
      body += bar('s1', LEFT, y, x(r.gb) - LEFT, 22, `${label}: ${f(r.gb, 2)} GB`)
      body += `<text class="t1 num" x="${x(r.gb) + 8}" y="${y + 16}">${f(r.gb, 2)} GB</text>`
    })
    const yEnd = y0 + rows.length * 32 - 6
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 - 4}" y2="${yEnd}"/>`
    body += refLine('ref', x(ceiling), y0 - 6, yEnd, `${t.ceiling} ${f(ceiling, 0)} GB (NFR-004)`)
    const h = yEnd + 30
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, true))}</text>`
    const desc = rows.map((r) => `${r.n} ${t.photos}: ${f(r.gb, 2)} GB`).join('. ')
    out['bench-memory'] = doc(t.memTitle, `${desc}. ${t.ceiling} ${ceiling} GB.`, h, body)
  }

  // 3. Fotollection vs Chromium; the browser is the reference.
  {
    const panels = [
      { title: t.cExport, unit: t.cExportUnit, better: t.higher, key: 'exportPerSecond', fmt: (v) => f(v, 1) },
      { title: t.cPreview, unit: t.cPreviewUnit, better: t.lower, key: 'previewWarmP50Ms', fmt: (v) => f(v, 1) },
      { title: t.cMemory, unit: t.cMemoryUnit, better: t.lower, key: 'peakRssBytes', fmt: (v) => f(v / 1e9, 2) },
    ]
    const rows = [
      { name: t.rust, cls: 's1', data: rust },
      { name: t.web, cls: 's2', data: web },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      const max = Math.max(...rows.map((r) => r.data[p.key]))
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)} · ${esc(p.better)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = r.data[p.key]
        const w = Math.max(4, (PLOT * v) / max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)} ${p.unit}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${p.fmt(v)}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * 30 - 4}"/>`
    })
    const h = 34 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, false))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(r.data[p.key])} ${p.unit}`).join(', ')}`)
      .join('. ')
    out.bench = doc(t.compareTitle(rust.photos), desc, h, body)
  }

  // 4. Preview breakdown: stacked segments, against the budget and Chromium.
  {
    const segs = [
      { name: t.resize, cls: 's3', v: rust.previewDrawMs },
      { name: t.stamp, cls: 's4', v: Math.max(0, rust.previewStampMs) },
      { name: t.encode, cls: 's5', v: rust.previewEncodeMs },
    ]
    const total = segs.reduce((s, x) => s + x.v, 0)
    const ms = (v) => (v < 0.05 ? `< ${f(0.1, 1)} ms` : `${f(v, 1)} ms`)
    const budget = budgets['NFR-002'].value
    const max = budget * 1.1
    const x = (v) => LEFT + (PLOT * v) / max
    let body = `<text class="t3" x="0" y="30">${esc(t.ms)} · ${esc(t.lower)}</text>`
    // Legend: swatch + name in text ink, so identity never relies on colour alone.
    let lx = 0
    for (const s of segs) {
      body += `<rect class="${s.cls}" x="${lx}" y="44" width="10" height="10" rx="2"/><text class="t2" x="${lx + 16}" y="53">${esc(`${s.name} ${ms(s.v)}`)}</text>`
      lx += 40 + (s.name.length + 7) * 7
    }
    const y = 100
    body += `<text class="t2" x="${LEFT - 10}" y="${y + 17}" text-anchor="end">${esc(t.rust)}</text>`
    let cx = LEFT
    for (const s of segs) {
      const w = (PLOT * s.v) / max
      if (w <= 0.5) continue
      body += `<rect class="${s.cls}" x="${cx}" y="${y}" width="${w}" height="26"><title>${esc(`${s.name}: ${f(s.v, 1)} ms`)}</title></rect>`
      if (w > 48) body += `<text class="t1 num in" x="${cx + w / 2}" y="${y + 17}" text-anchor="middle">${f(s.v, 1)}</text>`
      cx += w
      body += `<line class="gap" x1="${cx}" x2="${cx}" y1="${y}" y2="${y + 26}"/>`
    }
    // The total sits after the bar, or after the Chromium line when that is just past the bar.
    const xWeb = x(web.previewWarmP50Ms)
    const webInsideBar = xWeb < cx
    const totalX = !webInsideBar && xWeb - cx < 80 ? xWeb + 8 : cx + 8
    body += `<text class="t1 num" x="${totalX}" y="${y + 18}">${f(total, 1)} ms</text>`
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y - 6}" y2="${y + 32}"/>`
    body += refLine('refw', xWeb, y - 10, y + 36, `${t.webShort} ${f(web.previewWarmP50Ms, 1)}`, 'end', webInsideBar ? [y - 2, y + 28] : null)
    body += refLine('ref', x(budget), y - 10, y + 34, `${t.budget} ${budgetText(budgets['NFR-002'])}`, 'end')
    const h = y + 66
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.footer(f, false))}</text>`
    const desc = `${segs.map((s) => `${s.name} ${f(s.v, 1)} ms`).join(', ')}; ${f(total, 1)} ms. ${t.web} ${f(web.previewWarmP50Ms, 1)} ms. ${t.budget} ${budget} ms.`
    out['bench-preview'] = doc(t.previewTitle, desc, h, body)
  }
  // 6. Print quality: Fotollection's export against a typical web-editor export.
  {
    const S = quality.sets.srgb
    const P = quality.sets.displayP3
    const rows = [
      { name: t.rust, cls: 's1', who: 'fotollection' },
      { name: t.qWeb, cls: 's2', who: 'web_editor' },
    ]
    const panels = [
      { title: t.qDetail, unit: t.qPercent, v: (w) => S[w].detail_kept * 100, max: 100, fmt: (v) => `${f(v, 0)} %` },
      { title: t.qChroma, unit: t.qPercent, v: (w) => P[w].chroma_kept * 100, max: 100, fmt: (v) => `${f(v, 1)} %` },
      { title: t.qDe, unit: t.qDeUnit, v: (w) => S[w].de_mean, max: 2, fmt: (v) => f(v, 2), ref: 1 },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = p.v(r.who)
        const w = Math.max(4, (PLOT * Math.min(v, p.max)) / p.max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.fmt(v))}</text>`
      })
      const yEnd = y0 + 26 + rows.length * 30 - 4
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${yEnd}"/>`
      if (p.ref !== undefined) {
        body += refLine('ref', LEFT + (PLOT * p.ref) / p.max, y0 + 30, yEnd, t.qVisible, 'middle')
      }
    })
    const h = 34 + panels.length * ph + 14
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.qFooter(f(S.photos, 0), f(P.photos, 0)))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(p.v(r.who))}`).join(', ')}`)
      .join('. ')
    out['bench-quality'] = doc(t.qTitle, desc, h, body)
  }

  // 7. Share export (FR-053): after Facebook's recompression, Fotollection vs the usual way.
  {
    const fb = share.presets.facebook
    const S = fb.srgb
    const P = fb.displayP3
    const rows = [
      { name: t.rust, cls: 's1', who: 'fotollection_after_facebook' },
      { name: t.shUsual, cls: 's2', who: 'usual_after_facebook' },
    ]
    const panels = [
      { title: t.shChroma, unit: t.qPercent, v: (w) => P[w].chroma_kept * 100, max: 100, fmt: (v) => `${f(v, 1)} %` },
      { title: t.shDiff, unit: t.qDeUnit, v: (w) => (1 - S[w].ssim) * 100, max: 5, fmt: (v) => `${f(v, 1)} %` },
    ]
    const ph = 30 + rows.length * 30 + 18
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 34 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(p.unit)}</text>`
      rows.forEach((r, j) => {
        const y = y0 + 26 + j * 30
        const v = p.v(r.who)
        const w = Math.max(4, (PLOT * Math.min(v, p.max)) / p.max)
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
        body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${p.fmt(v)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.fmt(v))}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * 30 - 4}"/>`
    })
    // File size: the camera file, then what each preset writes.
    const sizes = [
      { name: t.shSource, cls: 's3', v: share.presets.original.srgb.source_mb },
      { name: t.shOriginal, cls: 's1', v: share.presets.original.srgb.export_mb },
      { name: t.shFacebook, cls: 's1', v: S.export_mb },
    ]
    const y0 = 34 + panels.length * ph
    const maxMb = Math.max(...sizes.map((s) => s.v))
    body += `<text class="t1" x="0" y="${y0 + 14}">${esc(t.shSize)}</text>`
    body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(t.shMb)}</text>`
    sizes.forEach((s, j) => {
      const y = y0 + 26 + j * 30
      const w = Math.max(4, (PLOT * s.v) / maxMb)
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(s.name)}</text>`
      body += bar(s.cls, LEFT, y, w, 22, `${s.name}: ${f(s.v, 2)} MB`)
      body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(`${f(s.v, 2)} MB`)}</text>`
    })
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + sizes.length * 30 - 4}"/>`
    const h = y0 + 26 + sizes.length * 30 + 22
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.shFooter(f(S.photos, 0), f(P.photos, 0)))}</text>`
    const desc = rows
      .map((r) => `${r.name}: ${panels.map((p) => `${p.title} ${p.fmt(p.v(r.who))}`).join(', ')}`)
      .concat(sizes.map((s) => `${s.name} ${f(s.v, 2)} MB`))
      .join('. ')
    out['bench-share'] = doc(t.shTitle, desc, h, body)
  }

  // 8. Camera RAW (FR-016): the develop per camera, then the medians against a RAW-to-DNG
  // conversion, which is a third-party estimate (bench/results/raw.json).
  {
    const ms = (list, key) => {
      const v = list.map((x) => x[key]).sort((a, b) => a - b)
      return v[Math.floor(v.length / 2)]
    }
    const samples = [...raw.samples].sort((a, b) => a.develop_ms - b.develop_ms)
    const maxS = Math.max(...samples.map((x) => x.develop_ms / 1000), raw.dng_converter.seconds_per_file_high)
    // Wider names on the left, and room on the right for the value and the thumbnail time.
    const L = 250
    const P = W - L - 190
    const sec = (v) => `${f(v, v < 0.1 ? 2 : 1)} s`
    let body = ''
    let y0 = 34
    body += `<text class="t1" x="0" y="${y0 + 14}">${esc(t.rwPerFormat)}</text>`
    body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(t.rwSeconds)}</text>`
    samples.forEach((x, j) => {
      const y = y0 + 26 + j * 30
      const v = x.develop_ms / 1000
      const w = Math.max(4, (P * v) / maxS)
      body += `<text class="t2" x="${L - 10}" y="${y + 15}" text-anchor="end">${esc(x.label)}</text>`
      body += bar('s1', L, y, w, 22, `${x.label}: ${sec(v)}, ${t.rwThumbOf(x.preview_400_ms)}`)
      body += `<text class="t1 num" x="${L + w + 8}" y="${y + 16}">${esc(sec(v))}</text>`
      body += `<text class="t3" x="${W}" y="${y + 16}" text-anchor="end">${esc(t.rwThumbOf(x.preview_400_ms))}</text>`
    })
    body += `<line class="axis" x1="${L}" x2="${L}" y1="${y0 + 22}" y2="${y0 + 26 + samples.length * 30 - 4}"/>`
    y0 += 26 + samples.length * 30 + 18
    const dng = raw.dng_converter
    const rows = [
      { name: t.rwThumb, cls: 's1', v: ms(raw.samples, 'preview_400_ms') / 1000, text: (v) => sec(v) },
      { name: t.rwDevelop, cls: 's1', v: ms(raw.samples, 'develop_ms') / 1000, text: (v) => sec(v) },
      {
        name: t.rwDng,
        cls: 's2',
        v: (dng.seconds_per_file_low + dng.seconds_per_file_high) / 2,
        text: () => `${f(dng.seconds_per_file_low, 1)}–${f(dng.seconds_per_file_high, 1)} s`,
      },
    ]
    body += `<text class="t1" x="0" y="${y0 + 14}">${esc(t.rwCompare)}</text>`
    rows.forEach((r, j) => {
      const y = y0 + 26 + j * 30
      const w = Math.max(4, (P * r.v) / maxS)
      body += `<text class="t2" x="${L - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
      body += bar(r.cls, L, y, w, 22, `${r.name}: ${r.text(r.v)}`)
      body += `<text class="t1 num" x="${L + w + 8}" y="${y + 16}">${esc(r.text(r.v))}</text>`
    })
    body += `<line class="axis" x1="${L}" x2="${L}" y1="${y0 + 22}" y2="${y0 + 26 + rows.length * 30 - 4}"/>`
    const h = y0 + 26 + rows.length * 30 + 40
    t.rwFooter(f(raw.samples.length, 0)).forEach((line, j) => {
      body += `<text class="t3" x="0" y="${h - 22 + j * 18}">${esc(line)}</text>`
    })
    const desc = samples
      .map((x) => `${x.label}: ${sec(x.develop_ms / 1000)}, ${t.rwThumbOf(x.preview_400_ms)}`)
      .concat(rows.map((r) => `${r.name}: ${r.text(r.v)}`))
      .join('. ')
    out['bench-raw'] = doc(t.rwTitle, desc, h, body)
  }

  // 5. Best-shot accuracy per improvement step (higher is better), against the labels.
  {
    const steps = bestshot.steps
    const panels = [
      {
        title: t.bsGroups(bestshot.labelledGroups),
        max: bestshot.labelledGroups,
        v: (s) => s.foundGroups,
        text: (s) => `${f(s.foundGroups, 0)}/${f(bestshot.labelledGroups, 0)}`,
      },
      {
        title: t.bsPicks,
        max: 1,
        v: (s) => s.picksRight / s.picksScored,
        text: (s) => `${f(s.picksRight, 0)}/${f(s.picksScored, 0)}`,
      },
    ]
    const ph = 40 + steps.length * 30
    let body = ''
    panels.forEach((p, i) => {
      const y0 = 30 + i * ph
      body += `<text class="t1" x="0" y="${y0 + 14}">${esc(p.title)}</text>`
      body += `<text class="t3" x="${W}" y="${y0 + 14}" text-anchor="end">${esc(t.higher)}</text>`
      steps.forEach((s, j) => {
        const y = y0 + 26 + j * 30
        const w = Math.max(4, (PLOT * p.v(s)) / p.max)
        const last = j === steps.length - 1
        body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(s[lang])}</text>`
        body += bar(last ? 's1' : 's1 old', LEFT, y, w, 22, `${s[lang]}: ${p.text(s)}`)
        body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${esc(p.text(s))}</text>`
      })
      body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 + 22}" y2="${y0 + 26 + steps.length * 30 - 4}"/>`
    })
    const h = 30 + panels.length * ph + 30
    const [line1, line2] = t.bsFooter(f(bestshot.photos, 0), f(bestshot.labelledGroups, 0))
    body += `<text class="t3" x="0" y="${h - 20}">${esc(line1)}</text>`
    body += `<text class="t3" x="0" y="${h - 4}">${esc(line2)}</text>`
    const desc = steps
      .map((s) => `${s[lang]}: ${panels.map((p) => p.text(s)).join(', ')}`)
      .join('. ')
    out['bench-bestshot'] = doc(t.bsTitle, desc, h, body)
  }
  // 9. The GPU while editing (ADR-0039): one slider step, the CPU path's parts against the GPU.
  {
    const cpu = drag.photos[0]
    const decode = gpu.engines.find((e) => e.name.startsWith('WebKit'))?.decodeMs ?? 0
    const cpuMs = cpu.parts.tableMs + cpu.parts.lookStampMs + cpu.parts.encodeMs + decode
    const rows = [
      { name: t.gpuCpu, cls: 's2', v: cpuMs },
      ...gpu.engines.map((e) => ({ name: t.gpuEngines[e.name] ?? e.name, cls: /SwiftShader/.test(e.name) ? 's2 old' : 's1', v: e.stepMs })),
    ]
    const max = Math.max(...rows.map((r) => r.v)) * 1.1
    let body = `<text class="t3" x="0" y="30">${esc(t.gpuUnit)}</text>`
    const y0 = 50
    rows.forEach((r, i) => {
      const y = y0 + i * 32
      const w = Math.max(4, (PLOT * r.v) / max)
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
      body += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${f(r.v, 1)} ms`)
      body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${f(r.v, 1)} ms</text>`
    })
    const yEnd = y0 + rows.length * 32 - 6
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 - 4}" y2="${yEnd}"/>`
    const h = yEnd + 46
    body += `<text class="t3" x="0" y="${h - 22}">${esc(t.noGpuNote)}</text>`
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.gpuFooter(f))}</text>`
    out['bench-gpu'] = doc(t.gpuTitle, rows.map((r) => `${r.name}: ${f(r.v, 1)} ms`).join('. '), h, body)
  }

  // 10. Export speed by photos in flight (ADR-0039), with the peak memory of each.
  {
    const set = inflight.sets[0]
    const chosen = set.defaultAfter.inflight
    const max = Math.max(...set.runs.map((r) => r.photosPerSecond)) * 1.15
    let body = `<text class="t3" x="0" y="30">${esc(t.inflightUnit)}</text>`
    const y0 = 50
    set.runs.forEach((r, i) => {
      const y = y0 + i * 32
      const w = Math.max(4, (PLOT * r.photosPerSecond) / max)
      const label = t.inflightRow(r.inflight)
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(label)}</text>`
      body += bar(r.inflight === chosen ? 's1' : 's1 old', LEFT, y, w, 22, `${label}: ${f(r.photosPerSecond, 1)}`)
      const gb = f(r.exportRssMb / 1000, 2)
      body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${f(r.photosPerSecond, 1)} · ${gb} GB${r.inflight === chosen ? ` · ${esc(t.inflightChosen)}` : ''}</text>`
    })
    const yEnd = y0 + set.runs.length * 32 - 6
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 - 4}" y2="${yEnd}"/>`
    const h = yEnd + 48
    body += `<text class="t3" x="0" y="${h - 22}">${esc(t.inflightWhy)}</text>`
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.inflightFooter(f))}</text>`
    out['bench-inflight'] = doc(t.inflightTitle, set.runs.map((r) => `${r.inflight}: ${f(r.photosPerSecond, 1)} photos/s, ${r.exportRssMb} MB`).join('. '), h, body)
  }

  // 11. Opening a photo across versions (FR-091): one hue, the older versions muted, the
  // current one full, a line at NFR-012's 100 ms.
  {
    const current = browse.versions[browse.versions.length - 1].version
    const rows = browse.versions.flatMap((v) => [
      { name: t.photoOpenRow(v.version, 'JPEG'), v: v.jpeg_p50, now: v.version === current },
      { name: t.photoOpenRow(v.version, 'RAW'), v: v.raw_p50, now: v.version === current },
    ])
    const max = Math.max(...rows.map((r) => r.v)) * 1.15
    let body = `<text class="t3" x="0" y="30">${esc(t.photoOpenUnit)}</text>`
    const y0 = 62
    rows.forEach((r, i) => {
      // A small gap between versions, none between a version's two rows.
      const y = y0 + i * 30 + Math.floor(i / 2) * 10
      const w = Math.max(4, (PLOT * r.v) / max)
      body += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
      body += bar(r.now ? 's1' : 's1 old', LEFT, y, w, 22, `${r.name}: ${f(r.v, 0)} ms`)
      body += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${f(r.v, 0)} ms</text>`
    })
    const yEnd = y0 + rows.length * 30 + Math.floor((rows.length - 1) / 2) * 10 - 6
    body += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${y0 - 4}" y2="${yEnd}"/>`
    // The line breaks where the current version's short bars carry their numbers.
    const lastGroup = y0 + (rows.length - 2) * 30 + Math.floor((rows.length - 2) / 2) * 10
    body += refLine('ref', LEFT + (PLOT * 100) / max, y0 - 8, yEnd, t.photoOpenBudget, 'middle', [lastGroup - 6, yEnd])
    const h = yEnd + 48
    body += `<text class="t3" x="0" y="${h - 22}">${esc(t.photoOpenWhy)}</text>`
    body += `<text class="t3" x="0" y="${h - 4}">${esc(t.photoOpenFooter())}</text>`
    out['bench-open'] = doc(t.photoOpenTitle, rows.map((r) => `${r.name}: ${f(r.v, 0)} ms`).join('. '), h, body)
  }
  return out
}

for (const lang of ['en', 'vi']) {
  for (const [name, svg] of Object.entries(charts(lang))) {
    writeFileSync(join(images, `${name}${lang === 'en' ? '' : '-vi'}.svg`), svg)
  }
}
// The website shows these in its own colours: the home page and the performance page (site/README.md).
STYLE = styleFor(THEMES.site)
const SITE_CHARTS = ['bench-budgets', 'bench-quality', 'bench-share', 'bench-raw', 'bench-bestshot', 'bench-memory', 'bench', 'bench-gpu', 'bench-inflight', 'bench-open']
mkdirSync(join(root, 'site/assets/img/charts'), { recursive: true })
// Each chart's size, for the width and height of its <img>: a lazy chart with no height grew
// as it loaded and moved the anchors below it while the page scrolled to them (LESSONS).
const sizes = {}
// Every language whose Performance page is published (site/_data/languages.yml); the README
// charts above stay in the READMEs' two languages.
for (const { code: lang } of published.filter((l) => l.pages.includes('performance'))) {
  if (!T[lang]) throw new Error(`bench/chart.mjs: no words for ${lang} in T`)
  const all = charts(lang)
  for (const name of SITE_CHARTS) {
    writeFileSync(join(root, 'site/assets/img/charts', `${name}-${lang}.svg`), all[name])
    const [, , w, h] = (/viewBox="([^"]+)"/.exec(all[name])?.[1] ?? '0 0 752 400').split(/\s+/).map(Number)
    sizes[name] = { width: w, height: h }
  }
}
writeFileSync(join(root, 'site/_data/charts.json'), `${JSON.stringify(sizes, null, 2)}\n`)
console.log(JSON.stringify({ measured, rust, web, inflight8, batch }, null, 2))
