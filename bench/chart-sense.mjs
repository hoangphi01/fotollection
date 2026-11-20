// SPDX-License-Identifier: GPL-3.0-only
// RollSense's charts for bench/chart.mjs (NFR-022): split out so the red chart.mjs does not grow
// (CODE_HEALTH, FR-138); its words in every language beside the drawing that uses them.

// RollSense's two charts (NFR-022, owner 2026-11-20: "the performance page must be professional"):
// the load test second by second, and the time per photo by threads. Words per language.
const SENSE_T = {
  en: {
    ecgTitle: 'RollSense under load: 9,639 photos, second by second',
    ecgUnit: 'One Windows laptop reading every photo on all its cores; each strip is one measure over the same minutes',
    rows: { cpu: 'Processor', ghz: 'Clock', mem: 'Memory of the pass', temp: 'Temperature', gpu: 'Graphics card' },
    vals: { cpu: (v) => `${v} % busy`, ghz: (v) => `${v} GHz on average`, mem: (v) => `${v} MB at most`, temp: (v) => `${v} °C on average`, gpu: (v) => `at most ${v} %` },
    phases: { all: 'All 9,639 photos, 8 threads', ladder: '965 photos: 2, 4, 8 threads', model: 'Model alone' },
    min: 'min',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · measured every second, drawn every 15 s`,
    senseTitle: 'RollSense: time per photo, by threads',
    senseUnit: 'ms per photo on its first read (lower is better)',
    senseRow: (n) => `${n} threads`,
    senseApp: 'as Fotollection runs',
    senseModel: 'The model alone, 8 threads',
    senseWhy: 'The model takes about 30 % of a photo\'s time; the rest is reading and decoding the photo',
    senseFooter: (n, w, d) => `${n} photos (camera, phone and RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · read again from the cache: ${w} ms`,
  },
  vi: {
    ecgTitle: 'RollSense khi chạy nặng: 9.639 ảnh, từng giây',
    ecgUnit: 'Một laptop Windows đọc mọi tấm ảnh trên tất cả các nhân; mỗi dải là một chỉ số trên cùng trục phút',
    rows: { cpu: 'Bộ xử lý', ghz: 'Xung nhịp', mem: 'Bộ nhớ của lượt đọc', temp: 'Nhiệt độ', gpu: 'Card đồ hoạ' },
    vals: { cpu: (v) => `bận ${v} %`, ghz: (v) => `trung bình ${v} GHz`, mem: (v) => `tối đa ${v} MB`, temp: (v) => `trung bình ${v} °C`, gpu: (v) => `tối đa ${v} %` },
    phases: { all: 'Cả 9.639 ảnh, 8 luồng', ladder: '965 ảnh: 2, 4, 8 luồng', model: 'Chỉ mô hình' },
    min: 'phút',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · đo mỗi giây, vẽ mỗi 15 giây`,
    senseTitle: 'RollSense: thời gian mỗi ảnh theo số luồng',
    senseUnit: 'ms mỗi ảnh ở lần đọc đầu (càng thấp càng tốt)',
    senseRow: (n) => `${n} luồng`,
    senseApp: 'như Fotollection chạy',
    senseModel: 'Chỉ mô hình, 8 luồng',
    senseWhy: 'Mô hình chiếm khoảng 30 % thời gian của mỗi ảnh; phần còn lại là đọc và giải mã ảnh',
    senseFooter: (n, w, d) => `${n} ảnh (máy ảnh, điện thoại và RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · đọc lại từ cache: ${w} ms`,
  },
  cs: {
    ecgTitle: 'RollSense pod zátěží: 9 639 fotek, sekundu po sekundě',
    ecgUnit: 'Jeden notebook s Windows čte každou fotku na všech jádrech; každý pruh je jedna veličina ve stejných minutách',
    rows: { cpu: 'Procesor', ghz: 'Takt', mem: 'Paměť průchodu', temp: 'Teplota', gpu: 'Grafická karta' },
    vals: { cpu: (v) => `vytížení ${v} %`, ghz: (v) => `průměrně ${v} GHz`, mem: (v) => `nejvýše ${v} MB`, temp: (v) => `průměrně ${v} °C`, gpu: (v) => `nejvýše ${v} %` },
    phases: { all: 'Všech 9 639 fotek, 8 vláken', ladder: '965 fotek: 2, 4, 8 vláken', model: 'Jen model' },
    min: 'min',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · měřeno každou sekundu, kresleno po 15 s`,
    senseTitle: 'RollSense: čas na fotku podle počtu vláken',
    senseUnit: 'ms na fotku při prvním čtení (méně je lépe)',
    senseRow: (n) => `${n} vlákna`,
    senseApp: 'jak běží Fotollection',
    senseModel: 'Jen model, 8 vláken',
    senseWhy: 'Model zabere asi 30 % času fotky; zbytek je čtení a dekódování fotky',
    senseFooter: (n, w, d) => `${n} fotek (fotoaparát, telefon a RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · znovu z mezipaměti: ${w} ms`,
  },
  de: {
    ecgTitle: 'RollSense unter Last: 9.639 Fotos, Sekunde für Sekunde',
    ecgUnit: 'Ein Windows-Laptop liest jedes Foto auf allen Kernen; jeder Streifen ist ein Messwert über dieselben Minuten',
    rows: { cpu: 'Prozessor', ghz: 'Takt', mem: 'Speicher des Durchgangs', temp: 'Temperatur', gpu: 'Grafikkarte' },
    vals: { cpu: (v) => `${v} % ausgelastet`, ghz: (v) => `im Mittel ${v} GHz`, mem: (v) => `höchstens ${v} MB`, temp: (v) => `im Mittel ${v} °C`, gpu: (v) => `höchstens ${v} %` },
    phases: { all: 'Alle 9.639 Fotos, 8 Threads', ladder: '965 Fotos: 2, 4, 8 Threads', model: 'Nur das Modell' },
    min: 'Min.',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · jede Sekunde gemessen, alle 15 s gezeichnet`,
    senseTitle: 'RollSense: Zeit pro Foto nach Threads',
    senseUnit: 'ms pro Foto beim ersten Lesen (weniger ist besser)',
    senseRow: (n) => `${n} Threads`,
    senseApp: 'wie Fotollection läuft',
    senseModel: 'Nur das Modell, 8 Threads',
    senseWhy: 'Das Modell braucht etwa 30 % der Zeit eines Fotos; der Rest ist Lesen und Dekodieren',
    senseFooter: (n, w, d) => `${n} Fotos (Kamera, Handy und RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · erneut aus dem Cache: ${w} ms`,
  },
  fr: {
    ecgTitle: 'RollSense en charge : 9 639 photos, seconde par seconde',
    ecgUnit: 'Un portable Windows lit chaque photo sur tous ses cœurs ; chaque bande est une mesure sur les mêmes minutes',
    rows: { cpu: 'Processeur', ghz: 'Fréquence', mem: 'Mémoire de la passe', temp: 'Température', gpu: 'Carte graphique' },
    vals: { cpu: (v) => `occupé à ${v} %`, ghz: (v) => `${v} GHz en moyenne`, mem: (v) => `${v} Mo au plus`, temp: (v) => `${v} °C en moyenne`, gpu: (v) => `${v} % au plus` },
    phases: { all: 'Les 9 639 photos, 8 threads', ladder: '965 photos : 2, 4, 8 threads', model: 'Le modèle seul' },
    min: 'min',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · mesuré chaque seconde, tracé toutes les 15 s`,
    senseTitle: 'RollSense : temps par photo selon les threads',
    senseUnit: 'ms par photo à la première lecture (plus bas est mieux)',
    senseRow: (n) => `${n} threads`,
    senseApp: 'comme Fotollection tourne',
    senseModel: 'Le modèle seul, 8 threads',
    senseWhy: 'Le modèle prend environ 30 % du temps d\'une photo ; le reste est la lecture et le décodage',
    senseFooter: (n, w, d) => `${n} photos (appareil, téléphone et RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · relues depuis le cache : ${w} ms`,
  },
  hu: {
    ecgTitle: 'RollSense terhelés alatt: 9639 fotó, másodpercről másodpercre',
    ecgUnit: 'Egy windowsos laptop minden fotót minden magon olvas; minden sáv egy mérés ugyanazokon a perceken',
    rows: { cpu: 'Processzor', ghz: 'Órajel', mem: 'Az olvasás memóriája', temp: 'Hőmérséklet', gpu: 'Videokártya' },
    vals: { cpu: (v) => `${v} % terhelés`, ghz: (v) => `átlagosan ${v} GHz`, mem: (v) => `legfeljebb ${v} MB`, temp: (v) => `átlagosan ${v} °C`, gpu: (v) => `legfeljebb ${v} %` },
    phases: { all: 'Mind a 9639 fotó, 8 szál', ladder: '965 fotó: 2, 4, 8 szál', model: 'Csak a modell' },
    min: 'perc',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · másodpercenként mérve, 15 mp-enként rajzolva`,
    senseTitle: 'RollSense: idő fotónként, szálak szerint',
    senseUnit: 'ms fotónként az első olvasáskor (a kevesebb a jobb)',
    senseRow: (n) => `${n} szál`,
    senseApp: 'ahogy a Fotollection fut',
    senseModel: 'Csak a modell, 8 szál',
    senseWhy: 'A modell egy fotó idejének kb. 30 %-a; a többi a fotó beolvasása és dekódolása',
    senseFooter: (n, w, d) => `${n} fotó (fényképező, telefon és RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · újra a gyorsítótárból: ${w} ms`,
  },
  pl: {
    ecgTitle: 'RollSense pod obciążeniem: 9639 zdjęć, sekunda po sekundzie',
    ecgUnit: 'Jeden laptop z Windows czyta każde zdjęcie na wszystkich rdzeniach; każdy pas to jeden pomiar w tych samych minutach',
    rows: { cpu: 'Procesor', ghz: 'Taktowanie', mem: 'Pamięć przebiegu', temp: 'Temperatura', gpu: 'Karta graficzna' },
    vals: { cpu: (v) => `zajęty w ${v} %`, ghz: (v) => `średnio ${v} GHz`, mem: (v) => `najwyżej ${v} MB`, temp: (v) => `średnio ${v} °C`, gpu: (v) => `najwyżej ${v} %` },
    phases: { all: 'Wszystkie 9639 zdjęć, 8 wątków', ladder: '965 zdjęć: 2, 4, 8 wątków', model: 'Sam model' },
    min: 'min',
    ecgFooter: (d) => `ThinkPad X1 (i7-10610U, 15 W) · Windows 10 · ${d} · mierzone co sekundę, rysowane co 15 s`,
    senseTitle: 'RollSense: czas na zdjęcie według wątków',
    senseUnit: 'ms na zdjęcie przy pierwszym odczycie (mniej znaczy lepiej)',
    senseRow: (n) => `${n} wątki`,
    senseApp: 'tak działa Fotollection',
    senseModel: 'Sam model, 8 wątków',
    senseWhy: 'Model zajmuje około 30 % czasu zdjęcia; reszta to odczyt i dekodowanie zdjęcia',
    senseFooter: (n, w, d) => `${n} zdjęć (aparat, telefon i RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · ponownie z pamięci podręcznej: ${w} ms`,
  },
  ja: {
    ecgTitle: '負荷時のRollSense：9,639枚を1秒ごとに',
    ecgUnit: 'Windowsノート1台がすべてのコアで全写真を読み込み、各帯は同じ時間軸上の1つの指標です',
    rows: { cpu: 'プロセッサー', ghz: 'クロック', mem: '読み込みのメモリー', temp: '温度', gpu: 'グラフィックカード' },
    vals: { cpu: (v) => `使用率${v}%`, ghz: (v) => `平均${v} GHz`, mem: (v) => `最大${v} MB`, temp: (v) => `平均${v} °C`, gpu: (v) => `最大${v}%` },
    phases: { all: '全9,639枚、8スレッド', ladder: '965枚：2・4・8スレッド', model: 'モデルのみ' },
    min: '分',
    ecgFooter: (d) => `ThinkPad X1（i7-10610U、15 W）· Windows 10 · ${d} · 1秒ごとに計測、15秒ごとに描画`,
    senseTitle: 'RollSense：スレッド数ごとの1枚あたりの時間',
    senseUnit: '初回読み込みの1枚あたりms（低いほど良い）',
    senseRow: (n) => `${n}スレッド`,
    senseApp: 'Fotollectionの実行時',
    senseModel: 'モデルのみ、8スレッド',
    senseWhy: 'モデルは1枚の時間の約30%で、残りは写真の読み込みとデコードです',
    senseFooter: (n, w, d) => `${n}枚（カメラ、スマートフォン、RAW）· ThinkPad i7-10610U · Windows 10 · ${d} · キャッシュから再読み込み：${w} ms`,
  },
  ko: {
    ecgTitle: '부하 상태의 RollSense: 9,639장을 1초마다',
    ecgUnit: 'Windows 노트북 한 대가 모든 코어로 모든 사진을 읽으며, 각 띠는 같은 시간 축 위의 측정값 하나입니다',
    rows: { cpu: '프로세서', ghz: '클럭', mem: '읽기의 메모리', temp: '온도', gpu: '그래픽 카드' },
    vals: { cpu: (v) => `사용률 ${v}%`, ghz: (v) => `평균 ${v} GHz`, mem: (v) => `최대 ${v} MB`, temp: (v) => `평균 ${v} °C`, gpu: (v) => `최대 ${v}%` },
    phases: { all: '9,639장 전체, 8스레드', ladder: '965장: 2·4·8스레드', model: '모델만' },
    min: '분',
    ecgFooter: (d) => `ThinkPad X1(i7-10610U, 15 W) · Windows 10 · ${d} · 1초마다 측정, 15초마다 그림`,
    senseTitle: 'RollSense: 스레드 수별 사진 한 장당 시간',
    senseUnit: '첫 읽기의 사진 한 장당 ms(낮을수록 좋음)',
    senseRow: (n) => `${n}스레드`,
    senseApp: 'Fotollection 실행 방식',
    senseModel: '모델만, 8스레드',
    senseWhy: '모델은 사진 한 장 시간의 약 30%이고, 나머지는 사진을 읽고 디코딩하는 시간입니다',
    senseFooter: (n, w, d) => `${n}장(카메라, 휴대폰, RAW) · ThinkPad i7-10610U · Windows 10 · ${d} · 캐시에서 다시 읽기: ${w} ms`,
  },
}

/** The load test as strips on one time axis, the phases as quiet bands behind them. */
export function senseCharts(lang, f, { sense, doc, bar, esc, W, LEFT, PLOT }) {
  const s = SENSE_T[lang] ?? SENSE_T.en
  const out = {}
  const tel = sense.telemetry
  const end = sense.phases.at(-1).end
  const L = 150
  const PW = W - L - 10
  const x = (m) => L + (PW * m) / end
  const rowH = 54
  const gap = 18
  const y0 = 92
  const run = sense.phases.find((p) => p.key === '1-pass-all-8')
  const during = tel.filter((p) => p.m >= run.start && p.m <= run.end)
  const avg = (k) => during.reduce((a, p) => a + (p[k] ?? 0), 0) / during.length
  const maxOf = (k) => Math.max(...tel.map((p) => p[k] ?? 0))
  const strips = [
    { k: 'cpu', lo: 0, hi: 100, top: '100 %', v: s.vals.cpu(f(avg('cpu'), 0)) },
    { k: 'ghz', lo: 0, hi: 4, top: '4 GHz', v: s.vals.ghz(f(avg('ghz'), 1)) },
    { k: 'mem', lo: 0, hi: 1000, top: `${f(1000, 0)} MB`, v: s.vals.mem(f(maxOf('mem'), 0)) },
    { k: 'temp', lo: 40, hi: 100, top: '100 °C', v: s.vals.temp(f(avg('temp'), 0)) },
    { k: 'gpu', lo: 0, hi: 100, top: '100 %', v: s.vals.gpu(f(maxOf('gpu'), 1)) },
  ]
  const bottom = y0 + strips.length * (rowH + gap) - gap
  let body = `<text class="t3" x="0" y="30">${esc(s.ecgUnit)}</text>`
  // The three parts of the run as bands, named above the strips.
  const groups = [
    { a: run.start, b: run.end, label: s.phases.all },
    { a: sense.phases.find((p) => p.key === '2a-k1-2').start, b: sense.phases.find((p) => p.key === '2c-k1-8').end, label: s.phases.ladder },
    { a: sense.phases.find((p) => p.key === '4a-synthetic-1').start, b: sense.phases.find((p) => p.key === '4c-synthetic-8').end, label: s.phases.model },
  ]
  groups.forEach((g, i) => {
    body += `<rect class="band" x="${x(g.a)}" y="${y0 - 6}" width="${x(g.b) - x(g.a)}" height="${bottom - y0 + 12}" rx="3"/>`
    // The middle band's name a line higher, so it never meets the narrow last one's.
    const last = i === groups.length - 1
    body += `<text class="t3" x="${last ? x(g.b) : x(g.a) + 4}" y="${y0 - (i === 1 ? 28 : 12)}" text-anchor="${last ? 'end' : 'start'}">${esc(g.label)}</text>`
  })
  strips.forEach((st, i) => {
    const top = y0 + i * (rowH + gap)
    const y = (v) => top + rowH - (rowH * (Math.min(st.hi, Math.max(st.lo, v)) - st.lo)) / (st.hi - st.lo)
    body += `<text class="t2" x="0" y="${top + 26}">${esc(s.rows[st.k])}</text>`
    body += `<text class="t1 num" x="0" y="${top + 46}">${esc(st.v)}</text>`
    body += `<line class="axis" x1="${L}" x2="${L + PW}" y1="${top + rowH}" y2="${top + rowH}"/>`
    body += `<line class="axis" x1="${L}" x2="${L + PW}" y1="${top}" y2="${top}" stroke-dasharray="2 4"/>`
    body += `<text class="t3 num" x="${L - 6}" y="${top + 4}" text-anchor="end">${esc(st.top)}</text>`
    // A strip breaks where its measure has no value (the pass's memory between two runs).
    let d = ''
    let pen = false
    for (const p of tel) {
      const v = p[st.k]
      if (v === null || v === undefined) { pen = false; continue }
      d += `${pen ? 'L' : 'M'}${x(p.m).toFixed(1)} ${y(v).toFixed(1)}`
      pen = true
    }
    body += `<path class="ln" d="${d}"><title>${esc(`${s.rows[st.k]}: ${st.v}`)}</title></path>`
  })
  // The minutes, every 15.
  for (let m = 0; m <= end; m += 15) {
    body += `<text class="t3 num" x="${x(m)}" y="${bottom + 18}" text-anchor="middle">${f(m, 0)}</text>`
  }
  body += `<text class="t3" x="${L + PW}" y="${bottom + 34}" text-anchor="end">${esc(s.min)}</text>`
  const h = bottom + 58
  body += `<text class="t3" x="0" y="${h - 4}">${esc(s.ecgFooter(sense.date))}</text>`
  out['bench-ecg'] = doc(s.ecgTitle, strips.map((st) => `${s.rows[st.k]}: ${st.v}`).join('. '), h, body)

  // The time per photo by threads, and the model's share of it.
  {
    const rows = [
      ...sense.pass.ladder.map((r) => ({ name: s.senseRow(r.threads), v: r.msCold, cls: 's1', note: r.threads === 8 ? s.senseApp : '' })),
      { name: s.senseModel, v: sense.model.find((m) => m.threads === 8).ms, cls: 's2', note: '' },
    ]
    const max = Math.max(...rows.map((r) => r.v)) * 1.3
    let b = `<text class="t3" x="0" y="30">${esc(s.senseUnit)}</text>`
    const top = 52
    rows.forEach((r, i) => {
      const y = top + i * 34 + (i === rows.length - 1 ? 12 : 0)
      const w = Math.max(4, (PLOT * r.v) / max)
      b += `<text class="t2" x="${LEFT - 10}" y="${y + 15}" text-anchor="end">${esc(r.name)}</text>`
      b += bar(r.cls, LEFT, y, w, 22, `${r.name}: ${f(r.v, 0)} ms`)
      b += `<text class="t1 num" x="${LEFT + w + 8}" y="${y + 16}">${f(r.v, 0)} ms</text>`
      if (r.note) b += `<text class="t3" x="${LEFT + w + 64}" y="${y + 16}">${esc(r.note)}</text>`
    })
    const yEnd = top + rows.length * 34 + 12 - 8
    b += `<line class="axis" x1="${LEFT}" x2="${LEFT}" y1="${top - 4}" y2="${yEnd}"/>`
    const hh = yEnd + 48
    b += `<text class="t3" x="0" y="${hh - 22}">${esc(s.senseWhy)}</text>`
    b += `<text class="t3" x="0" y="${hh - 4}">${esc(s.senseFooter(f(sense.pass.ladderPhotos, 0), f(sense.pass.all8.msWarm, 1), sense.date))}</text>`
    out['bench-sense'] = doc(s.senseTitle, rows.map((r) => `${r.name}: ${f(r.v, 0)} ms`).join('. '), hh, b)
  }
  return out
}
