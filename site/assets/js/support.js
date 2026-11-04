// SPDX-License-Identifier: GPL-3.0-only
// The "treat me" menu on the home page (#support): the chosen treat, or an amount of the
// visitor's own, becomes a dynamic VietQR code (EMVCo, Napas 247) with that amount and a
// transfer note, drawn as SVG in the page with qrcode.js (MIT). Nothing is sent anywhere;
// without this script the published static code stays.
;(function () {
  var box = document.querySelector('[data-treat]')
  var figure = box && box.querySelector('[data-treat-qr]')
  if (!box || !figure || typeof window.qrcode !== 'function') return
  var bin = box.dataset.bin
  var account = box.dataset.account
  var amountLabel = box.querySelector('[data-treat-amount]')
  var own = box.querySelector('[data-treat-own]')
  var locale = document.documentElement.lang === 'vi' ? 'vi-VN' : 'en-US'

  var tlv = function (id, value) {
    return id + String(value.length).padStart(2, '0') + value
  }
  // CRC-16/CCITT-FALSE over the payload up to and including "6304", as EMVCo asks.
  var crc16 = function (s) {
    var c = 0xffff
    for (var i = 0; i < s.length; i++) {
      c ^= s.charCodeAt(i) << 8
      for (var j = 0; j < 8; j++) c = c & 0x8000 ? ((c << 1) ^ 0x1021) & 0xffff : (c << 1) & 0xffff
    }
    return c.toString(16).toUpperCase().padStart(4, '0')
  }
  var payload = function (amount, note) {
    var merchant = tlv('00', 'A000000727') + tlv('01', tlv('00', bin) + tlv('01', account)) + tlv('02', 'QRIBFTTA')
    var body =
      tlv('00', '01') +
      tlv('01', amount ? '12' : '11') +
      tlv('38', merchant) +
      tlv('53', '704') +
      (amount ? tlv('54', String(amount)) : '') +
      tlv('58', 'VN') +
      (note ? tlv('62', tlv('08', note)) : '') +
      '6304'
    return body + crc16(body)
  }

  var NS = 'http://www.w3.org/2000/svg'
  var draw = function (text, label) {
    var qr = window.qrcode(0, 'M')
    qr.addData(text)
    qr.make()
    var n = qr.getModuleCount()
    var quiet = 4
    var d = ''
    for (var r = 0; r < n; r++)
      for (var c = 0; c < n; c++) if (qr.isDark(r, c)) d += 'M' + (c + quiet) + ' ' + (r + quiet) + 'h1v1h-1z'
    var svg = document.createElementNS(NS, 'svg')
    svg.setAttribute('viewBox', '0 0 ' + (n + 2 * quiet) + ' ' + (n + 2 * quiet))
    svg.setAttribute('role', 'img')
    svg.setAttribute('aria-label', label)
    svg.setAttribute('shape-rendering', 'crispEdges')
    var bg = document.createElementNS(NS, 'rect')
    bg.setAttribute('width', '100%')
    bg.setAttribute('height', '100%')
    bg.setAttribute('fill', '#fff')
    var path = document.createElementNS(NS, 'path')
    path.setAttribute('d', d)
    path.setAttribute('fill', '#000')
    svg.append(bg, path)
    figure.replaceChildren(svg)
  }

  var update = function () {
    var chosen = box.querySelector('input[name="treat"]:checked')
    if (!chosen) return
    var isOwn = chosen.value === 'own'
    var amount = isOwn ? Math.round(Number(own && own.value)) : Number(chosen.dataset.vnd)
    if (!Number.isFinite(amount) || amount < 1000 || amount > 100000000) amount = 0
    var note = chosen.dataset.note || 'Fotollection'
    draw(payload(amount, note), box.dataset.qrLabel)
    if (amountLabel) amountLabel.textContent = amount ? amount.toLocaleString(locale) + ' ₫ · ' + note : box.dataset.ownHint
  }

  box.addEventListener('change', update)
  // Typing an amount chooses "as you like".
  if (own)
    own.addEventListener('input', function () {
      var radio = box.querySelector('input[name="treat"][value="own"]')
      if (radio) radio.checked = true
      update()
    })
  update()
})()
