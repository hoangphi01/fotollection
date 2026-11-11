// SPDX-License-Identifier: GPL-3.0-only
// The files a Flatpak of Fotollection installs beside the app (Flathub, docs/ROADMAP.md), made
// from the sources the .deb and the AppImage already use, so the two never drift:
//
//   node scripts/flatpak-files.mjs          writes src-tauri/linux/flatpak/
//   node scripts/flatpak-files.mjs --check  fails when they are out of date (check-docs.sh)
//
// org.fotollection.Fotollection.metainfo.xml: src-tauri/linux/com.hoangphi01.fotollection.metainfo.xml
// under the Flatpak's ID (Flathub's IDs sit on a domain the project controls), launched by
// its own desktop file, with the tags Flathub's newer AppStream reads and Ubuntu 22.04's does
// not (LESSONS: the .deb's file keeps to AppStream 0.15): the developer, the brand colours,
// the source, contribute and donation links, and what the app needs to be used.
// org.fotollection.Fotollection.desktop: the launcher, with the file types of
// src-tauri/tauri.linux.conf.json. The Flathub manifest itself is written by the owner.
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const ID = 'org.fotollection.Fotollection'
const OLD = 'com.hoangphi01.fotollection'
const out = path.join(root, 'src-tauri/linux/flatpak')

const source = readFileSync(path.join(root, `src-tauri/linux/${OLD}.metainfo.xml`), 'utf8')
const linux = JSON.parse(readFileSync(path.join(root, 'src-tauri/tauri.linux.conf.json'), 'utf8'))
const mimes = [...new Set((linux.bundle?.fileAssociations ?? []).map((f) => f.mimeType).filter(Boolean))]

const replace = (text, from, to) => {
  if (!text.includes(from)) throw new Error(`flatpak-files: "${from}" is no longer in the metainfo`)
  return text.replace(from, to)
}

let meta = source
meta = replace(meta, '<!-- AppStream metadata (Linux software centres, Flathub, the VNLF App Explorer): shipped in the .deb and the AppImage at /usr/share/metainfo/. scripts/release.sh adds each release on top. -->',
  `<!-- Made by scripts/flatpak-files.mjs from ${OLD}.metainfo.xml: edit that file, then run the script. The Flatpak's AppStream metadata (Flathub). -->`)
meta = replace(meta, `<id>${OLD}</id>`, `<id>${ID}</id>`)
meta = meta.replace(/\n {2}<!-- Tags that appstream 0\.15[^\n]*-->/, '')
meta = replace(meta, '<developer_name>hoangphi01</developer_name>', `<developer id="org.fotollection">
    <name>hoangphi01</name>
  </developer>`)
meta = replace(meta, '<launchable type="desktop-id">Fotollection.desktop</launchable>', `<launchable type="desktop-id">${ID}.desktop</launchable>
  <provides>
    <id>${OLD}</id>
  </provides>`)
meta = replace(meta, '  <url type="bugtracker">https://github.com/hoangphi01/fotollection/issues</url>', `  <url type="bugtracker">https://github.com/hoangphi01/fotollection/issues</url>
  <url type="vcs-browser">https://github.com/hoangphi01/fotollection</url>
  <url type="contribute">https://github.com/hoangphi01/fotollection/blob/main/CONTRIBUTING.md</url>
  <url type="donation">https://ko-fi.com/hoangphi01</url>
  <url type="help">https://fotollection.org/manual/</url>`)
// The logo's colours, as the website's (site/assets/css/site.css): teal in light, wood in dark.
meta = replace(meta, '  <categories>', `  <branding>
    <color type="primary" scheme_preference="light">#2c836d</color>
    <color type="primary" scheme_preference="dark">#c98a52</color>
  </branding>
  <requires>
    <display_length compare="ge">768</display_length>
  </requires>
  <supports>
    <control>keyboard</control>
    <control>pointing</control>
  </supports>
  <categories>`)

const desktop = `[Desktop Entry]
Type=Application
Name=Fotollection
GenericName=Photo printing
Comment=Look through a trip's photos and prepare them for printing
Comment[vi]=Xem lại ảnh của chuyến đi và chuẩn bị để in
Exec=fotollection %F
Icon=${ID}
Terminal=false
Categories=Graphics;Photography;
Keywords=photo;print;RAW;Lightroom;preset;LUT;
MimeType=${mimes.join(';')};
StartupWMClass=fotollection
`
const files = { [`${ID}.metainfo.xml`]: meta, [`${ID}.desktop`]: desktop }

if (process.argv.includes('--check')) {
  const stale = Object.entries(files).filter(([name, text]) => !existsSync(path.join(out, name)) || readFileSync(path.join(out, name), 'utf8') !== text)
  if (stale.length) {
    console.error(`  ✗ ${stale.map(([n]) => n).join(', ')} out of date: run node scripts/flatpak-files.mjs`)
    process.exit(1)
  }
  console.log('flatpak-files: up to date')
} else {
  mkdirSync(out, { recursive: true })
  for (const [name, text] of Object.entries(files)) writeFileSync(path.join(out, name), text)
  console.log(`flatpak-files: wrote ${Object.keys(files).map((n) => `src-tauri/linux/flatpak/${n}`).join(', ')}`)
}
