# User Guide — Fotollection

> **Status:** Living · **Last reviewed:** 2026-11-04
>
> How to use the app, step by step, for the person at the keyboard. It describes behaviour that has shipped or is on `main`; the reasons and the full rules are in [PRD.md](PRD.md), linked from each section. Update it in the same commit as any change a user would notice ([DOCS_GUIDE.md](DOCS_GUIDE.md)).
>
> **This is also the manual on the website** ([fotollection.org/manual](https://fotollection.org/manual/), [ADR-0031](adr/0031-manual-on-the-website-from-the-user-guide.md)). Keep [USER_GUIDE.vi.md](USER_GUIDE.vi.md) in step: the same chapters, anchors (`<!-- chapter: id -->` on the line before a chapter title, after a blank line) and screenshots (`<!-- shot: name | alt text -->`), with the app's own labels. Shoot changed screenshots again with `site/screenshots/build.sh`.

<!-- chapter: start -->

## 1. Getting started

Fotollection works on the folders of photos you already have. It reads them where they are, and never moves, renames or changes a file ([FR-001](PRD.md#fr-001--import-photos)).

1. **Open a folder:** click **Open folder** (the folder icon next to *Library* in the sidebar), or drop a folder anywhere on the window. Its subfolders come too. JPEG, PNG, HEIC and camera RAW are read (chapter 14).
2. **The sidebar** lists, under *Library*:
   - **All photos**: every photo from every folder;
   - **Print basket** and **Set aside**: the two holding areas (chapter 4);
   - **each folder**, with its count. Click a folder to show only its photos. Ctrl/⌘-click, or the **Select folders** button, shows several together. The **×** that appears on a folder removes it from the library; the files stay on disk.
3. **Refresh folders** (the circling arrows) picks up photos added to these folders since; your edits are kept.
4. **The gallery** fills the middle. The toolbar above it holds *Photos* / *Groups* and *Find best shots* (chapter 3), the photo count, *Select* (chapter 4) and the filters (chapter 2).
5. **The Print settings panel** on the right holds, top to bottom: *Print / export size* (chapter 5), *Date stamp* (chapter 8), *Info frame* (chapter 9), *Camera clocks* (chapter 11) and *Places* (chapter 10) once the library has them, then the export controls (chapter 12) and the version number (chapter 17).

<!-- shot: grid | The library, culled with stars and picks -->

**Three modes:** the buttons at the top of the sidebar switch between **Simple** (one photo at a time, ↑ keep, ↓ drop, then export: chapter 18), **Collage** (two photos on one print: chapter 19, or postcards: chapter 20) and **Pro**, the app this guide describes. Your photos, basket and settings are the same in every mode, and the mode is remembered on this computer. **Cmd/Ctrl+1**, **2** and **3** switch between them, also from a preview, which closes (not while a crop is being edited) ([FR-048](PRD.md#fr-048--simple-collage-and-professional-modes)).

**Language and look:** click the version number at the foot of any panel, then choose *English* or *Tiếng Việt* under **Language**; it is the same in every mode. The three buttons at the foot of the sidebar set the look: **Follow the system**, **Light** or **Dark** ([FR-020](PRD.md#fr-020--localization), [FR-052](PRD.md#fr-052--appearance-theme-title-bar-and-hints)).

<!-- chapter: cull -->

## 2. Stars, flags and filters

Mark photos as you look through them, then show only the ones you want ([FR-022](PRD.md#fr-022--culling-rate-flag-filter)). Stars and flags never change the photo file; they are saved in the project.

1. Select a photo in the gallery, or open it in the preview.
2. Press **1**–**5** to give it stars, **0** to clear them.
3. Press **P** to pick it, **X** to reject it, **U** to clear its flag. A rejected photo stays in the gallery, faded, until you filter it out; nothing is deleted.
4. In the preview, the **Rating & flag** card does the same with the mouse: click a star (click it again to clear), **Pick (P)** or **Reject (X)**.
5. Filter the gallery with the two lists at the right of the toolbar:
   - flags: **All**, **Picks**, **Unflagged**, **Rejects** or **Unusual files**;
   - stars: **Any rating**, or **≥ 1** to **≥ 5**.

   An active filter is outlined in the theme colour, and the count shows how many photos are shown out of how many. When nothing matches, **Show all photos** clears the filters.

   **Unusual files** ([FR-071](PRD.md#fr-071--image-facts)) are the ones outside the common path, marked with an **ⓘ** in the gallery: a CMYK or 16-bit JPEG, an Adobe RGB photo, a Fujifilm X-Trans RAW, a DNG kind the app does not develop itself yet. The preview's **Image** card says what each file is, under its size: for example *JPEG · 4:2:0 · Baseline · 8-bit · Display P3 · read by zune-jpeg* or *ARW · Bayer RGGB · 14-bit · read by ImageIO*. A camera RAW other than DNG shows its sensor once it has been opened in the preview. Give this line in a bug report.

<!-- shot: manual-filters | The flag filter open above the gallery: All, Picks, Unflagged, Rejects, Unusual files -->

What the filters show is also what *Export* writes (chapter 12). Stars and rejects from Lightroom and other apps come in by themselves (§10.1).

<!-- chapter: best-shot -->

## 3. Best shot

Bursts and near-identical shots are grouped, and the best of each is suggested with its reasons ([FR-041](PRD.md#fr-041--best-shot-suggestions)). Nothing is rejected until you choose, and nothing is ever deleted.

1. Click **Find best shots** in the toolbar. The view switches to **Groups** while the photos are analysed on this computer; **Stop** ends it early. Photos already analysed are not looked at again.
2. Each group shows its shots in a row. The suggested one is marked **Best**, and the group says **Best of n** with its reasons: *n % sharper*, *Less camera shake*, *Eyes open*, *Better exposure*, *Fewer clipped highlights*, *Less noise*. When the shots score almost the same, it says **Too close to call**.
3. **Compare** shows the group's shots large, side by side, with a bar for each measure (*Sharpness*, *Eyes*, *Steadiness*, *Exposure*, *Highlights*, *Noise*) and how many faces have their eyes closed. **Keep this one** keeps the shot you prefer.
4. **Keep best** keeps the suggested shot: it is picked and the others in the group are rejected. In a group that is too close to call, it opens *Compare* so you choose. **Keep best in n clear groups**, above the groups, does it for every group that is not too close to call.
5. Back in **Photos**, show only the **Picks** (chapter 2). **Cmd/Ctrl+Z** takes any of this back.

<!-- shot: manual-groups | A group of four shots of a cat: the sharpest is suggested as the best, 63 % sharper -->

<!-- shot: groups | Compare: the four shots of the cat side by side, with a bar per measure -->

<!-- chapter: choose -->

## 4. Choosing the photos to print

A trip gives hundreds of photos; only a handful go to the lab. Fotollection keeps that choice in two **holding areas**, next to the gallery ([FR-050](PRD.md#fr-050--print-basket)):

| Area | What it holds | Where it is |
|------|---------------|-------------|
| **Print basket** | The photos you will print | Sidebar, under *All photos*, with its count |
| **Set aside** | Photos you put away for now. They leave the gallery but are **never deleted or moved on disk** | Sidebar, under *Print basket*, with its count |

A photo is in the gallery, in the print basket, or set aside, never in two at once. Photos in the basket stay visible in the gallery, with a small basket badge in the top-right corner of their thumbnail; photos set aside do not appear in *All photos* or in the folders until you bring them back. Stars and pick/reject flags are separate: a pick does not have to be printed, and the basket is not a flag.

Both areas are saved in the project (`.fotollection`) and in the session, so they are still there after you quit and reopen the app. Every move can be undone with **Cmd/Ctrl+Z**.

### 4.1 Workflow A: decide photo by photo in the preview

1. Open a photo: double-click it, or select it and press **Enter** (or **Space**).
2. Look at it with the print settings applied.
3. Press **↑** to put it in the **print basket**: the next photo opens straight away.
4. Press **↓** to **set it aside**: it leaves the gallery, and the next photo takes its place.
5. Press **→** to move on without deciding, **←** to go back.
6. Repeat until the end of the trip. **Esc** closes the preview.

<!-- shot: swipe | A photo in the print basket, with the keys to keep going -->

The **Print basket** button on the photo (next to *Crop*, the turn buttons and *Export*) does the same as ↑, and shows *In the basket* when the photo is already in it; click it again to take the photo out. A photo that was set aside shows **Back to the gallery** instead.

When you open the preview from the *Print basket* or *Set aside* view, ↑ and ↓ move the photo to the other area; a photo that stays where it is moves on to the next one.

### 4.2 Workflow B: tick photos in the gallery (select mode)

1. Click **Select** in the toolbar, above the photos. It becomes **Done**, and a bar appears under the toolbar.
2. Click photos to tick them: a tick circle appears in the bottom-right corner, and the photo is outlined in the theme colour. Click again to untick.
   - **Shift+click** ticks every photo between the last one ticked and this one.
   - **Cmd/Ctrl+A** or **Select all** ticks every photo shown.
   - **Enter** ticks or unticks the selected photo.
3. In the bar, choose what to do with the ticked photos:
   - **Print basket**: put them in the basket.
   - **Set aside**: take them out of the gallery.
   - **Back to the gallery**: only in the *Print basket* and *Set aside* views; brings them back.
   - **Clear**: untick everything.
   - **Export**: while photos are ticked, the export button at the foot of the panel says *Export 3 selected photos* and writes only those; untick them all and it exports the whole view again.
4. Click **Done**, or press **Esc**, to leave select mode. Ticks are temporary: what counts is the area each photo is in.

<!-- shot: select | Select mode: ticked photos, the print basket and the set-aside pile -->

**Several folders:** ticks follow you from one folder tab to another ([FR-056](PRD.md#fr-056--folders-as-tabs)). Tick a few photos in one folder, click another folder in the sidebar and keep ticking; the bar shows the total and how many are in other folders, and *Print basket* moves them all at once.

<!-- shot: tabs | Three folders as tabs: photos ticked in two of them, and the close button on a tab -->

In select mode, a double-click does not open the preview, and **Space works differently** (§4.3).

### 4.3 Quick-preview: look without opening (select mode only)

In select mode, **hold Space** over a photo to see it large, as it will print, with the gallery's print size, date stamp and info frame applied ([FR-059](PRD.md#fr-059--quick-preview)). **Release Space** to close it, like Quick Look on a Mac.

- It shows the photo under the mouse pointer; if the pointer is not over a photo, the last photo you ticked.
- The thumbnail appears at once and the full render replaces it a moment later.
- While Space is held: **← →** show the neighbouring photos, **Enter** ticks or unticks the photo shown, **Esc** closes.
- Nothing can be edited here. To change the crop, stamp or frame of one photo, open the normal preview (double-click, or **Enter** outside select mode).

Outside select mode, Space keeps opening the normal preview.

### 4.4 Looking again and exporting

- Click **Print basket** or **Set aside** in the sidebar to see only those photos. Change your mind with ↑ ↓ in the preview, or with select mode and **Back to the gallery**.
- Click **All photos**, or a folder, to return to the gallery.
- **Export follows the view** (chapter 12), unless photos are ticked: then only they go out (§4.2). From *Print basket*, *Export* writes exactly the basket, with the size estimate above the button ([FR-053](PRD.md#fr-053--export-for-social-media)). From *All photos* or a folder, photos set aside are never exported.

<!-- shot: manual-share-fill | Share online with Whole photo in a frame on: Square 1:1 and a white fill -->

<!-- shot: estimate | The size estimate above the export button, sharing for Facebook -->

<!-- chapter: size -->

## 5. Print size and sharpness

Choose the size you will print at, and Fotollection crops every photo to it and tells you whether each one is sharp enough ([FR-026](PRD.md#fr-026--resolution-warning), [FR-043](PRD.md#fr-043--custom-print-sizes), [FR-045](PRD.md#fr-045--original-aspect-ratio), [FR-067](PRD.md#fr-067--3-mm-border-on-every-print)).

1. In the **Print / export size** card, keep **Print** selected (for **Share online**, see chapter 12).
2. Open the size list:
   - **Standard (CEWE / EU)**: the sizes of the photo kiosks at dm and Rossmann, like *10 × 15 cm · 2:3*;
   - **Original**: the photo's own ratio, with no crop;
   - **Custom**: sizes you added.

   Each size carries a badge for the photo selected in the gallery: **Sharp** (300 DPI or more), **Acceptable** (150–299 DPI, fine at arm's length) or **Pixelated** (under 150 DPI). Hover over it to see the DPI.
3. Under the list, a line says how large the selected photo prints sharp, and how large it is still acceptable.
4. Every photo is cropped to the size's ratio, **centred by default**. Choose what stays per photo in the preview (chapter 6).
5. **3 mm border** leaves a border 3 mm wide on every side, **White** or **Black** as you choose under the switch, cut into the photo: the print keeps the exact size of the shop's kiosk. With **Original**, no pixel is lost: the border goes round the whole photo, the short edge stays 10 cm and the long edge grows by 6 mm. Photos for social media never get it. With the info frame on, the border takes the frame's colour (chapter 9).

<!-- shot: manual-size | The print size list, a sharpness badge on each size -->

**A size of your own:** choose **Custom size…**, type a name (for example *Instax wide*), the width and height, and the unit (in or cm), then **Save size**. Each edge must be between 2 and 100 cm. The bin button next to the list deletes the custom size selected.

<!-- chapter: crop -->

## 6. Crop and straighten

Each photo is cropped to the print size's ratio, centred. Choose what stays, level a horizon or turn a photo, one photo at a time ([FR-006](PRD.md#fr-006--crop-to-print-ratio), [FR-044](PRD.md#fr-044--per-photo-overrides)).

1. **Quickly:** in the preview, **drag the photo** to move it inside the print.
2. **Precisely:** click **Crop** above the photo, or press **C**. In the crop editor:
   - **drag** to move the crop, and **double-click** to centre it again;
   - **zoom** with the slider, a pinch, the scroll wheel or **+** / **−**, up to 8×;
   - **Straighten** levels the horizon, ±15°: the slider, or **← →** by 1° (**⌥ ← →** by 0.1°). Double-click the slider to go back to 0°;
   - the labels show the printed size and the DPI, which turns orange or red when zooming in leaves too few pixels.
3. **Done** (or **Enter**) keeps it, **Cancel** (or **Esc**) leaves it as it was, and **Reset** centres the crop and sets the angle back to 0°.
4. **Turn** a photo a quarter with **Rotate left** and **Rotate right** above the photo, or **[** and **]**, in the gallery too.

<!-- shot: crop | Straightening a horizon in the crop editor -->

In the preview's size card, *Crop* says **Centred**, with **Adjust**, or **Custom · zoom n×**, with **Reset**. When the info frame is set to *Fit whole photo* (chapter 9) the whole photo is printed, so there is nothing to crop.

<!-- chapter: zoom -->

## 7. Looking closely: zoom

In the preview, the **zoom buttons** at the bottom right of the photo show the zoom level between **−** and **+**; the last button goes back to fit ([FR-057](PRD.md#fr-057--zoom-in-the-preview)). The same with the keyboard and the trackpad:

- **Pinch** on the trackpad, or **Cmd/Ctrl +** and **Cmd/Ctrl −**, zoom about the pointer, up to 800 %.
- **Double-click** the photo to go between fit and 200 %.
- Zoomed in, **drag** the photo, or scroll with two fingers, to move around it.
- **Cmd/Ctrl 0** or the fit button goes back to the whole photo.

The preview is first shown at a light size that any computer draws quickly. When you zoom past what it holds, a small spinner appears beside the zoom level, and a moment after you stop moving the part on screen is replaced by the **full-resolution** pixels, exactly as the exported file will have them. Only the visible part is loaded, so even a 32 MP photo stays light.

<!-- chapter: stamp -->

## 8. Date stamp

A date in the corner of the print, like a film camera's, taken from when the photo was shot ([FR-004](PRD.md#fr-004--date-stamp), [FR-033](PRD.md#fr-033--retro-film-date-stamp-and-stamp-presets)). It is on by default.

1. In the **Date stamp** card, the switch turns it on or off for every photo. The card is the same in Simple, Collage and Professional mode, with every choice below.
2. Choose:
   - the **format**: `2026-07-14`, `14.07.2026` or `'26 07 14`, the film-camera style;
   - the **corner**: *Bottom right*, *Bottom left*, *Top right* or *Top left*;
   - the **colour**: **White** or **Retro orange**;
   - the **size**: **Small**, **Medium** or **Large**, the same three in every mode. The size follows the print: larger on a larger print, and on a collage sheet each photo's stamp is half the sheet's, as two share it.
3. **Place on the date line** adds the town where the photo was taken, from its GPS, offline: as **City**, **City, HU** (with the country code) or **City, country**. A photo without GPS can get a place by hand (chapter 10).
4. **Line under the date**: a short text across the photo, up to 80 characters, printed in capitals without accents (for example *KY NIEM BUDAPEST*).

<!-- shot: stamp | Date stamp with the town from the photo's GPS -->

**One photo differently:** in the preview, the **Date stamp** card holds the same controls for that photo alone; it is then marked **This photo**. **Apply to all photos** makes this photo's stamp the one for every photo. In the panel, **Reset n photos with their own stamp** makes every photo follow the panel again. Both can be undone.

The date is the one from the camera, corrected by the camera clock (chapter 11) or set by hand (chapter 10).

<!-- chapter: frame -->

## 9. Info frame "Shot on"

A white or black bar under the photo with the camera's logo and settings, like the photos shared from phones ([FR-040](PRD.md#fr-040--exif-info-frame), [FR-065](PRD.md#fr-065--border-in-crop-to-fill), [FR-067](PRD.md#fr-067--3-mm-border-on-every-print)). It is off by default, and it is the same for every photo.

1. Turn on the **Info frame** card.
2. Choose how it fits the print:
   - **Crop to fill**: the print keeps its exact size, and the bar takes its place inside it; drag the photo in the preview to choose what stays. With **3 mm border** on (in the print size card, chapter 5), a 3 mm border in the frame colour runs on the top and sides, still inside the print size, like an instant photo;
   - **Fit whole photo**: the whole photo at the print size, framed like an instax print with a 5 mm border on three sides and the bar below, so the print comes out larger than the size chosen. A note says how large, so you can pick a paper that fits at the shop. With **3 mm border** on, the whole photo is fitted inside the size chosen instead, and the print does not grow.
3. Choose **White** or **Black**, the date format, and the font: *Inter (built-in)*, or **Import OTF/TTF…** to use your own ([FR-042](PRD.md#fr-042--custom-fonts)).
4. Under **Show**, tick what the bar holds: *Logo*, *Camera*, *Lens*, *Focal length*, *Aperture*, *Shutter*, *ISO*, *Photo style*, *Date*, *Place*.
5. With the logo shown, choose its **Logo colours** (*Classic*, *Brand colours* or *One colour*) and its **Logo size** (double-click the slider for 100 %).

<!-- shot: frame | The “Shot on” info frame with the camera's logo -->

<!-- shot: manual-frame | The Info frame card: fit, colour, font and the fields to show -->

**The camera's details:** in the preview, under the frame card, change what the bar says for this camera: **Camera name**, **Lens** (for a manual lens the camera cannot see), **Focal (mm)**, **Aperture F** and **Photo style**, and the **Logo**: the bundled brand logo, **Text**, an **Image** (a PNG of your own) or **None**. A field left empty uses the photo's EXIF. These are saved for every photo from that camera.

<!-- chapter: dates-places -->

## 10. Dates and places

**Set them by hand** ([FR-032](PRD.md#fr-032--manual-date-and-place)), for scanned prints, a camera without a clock, or a photo without GPS:

1. **One photo:** in the preview, click **Set the date…** in *Captured* or **Set the place…** in *Location*.
2. **Many photos:** in select mode, tick them and click **Date and place…** in the selection bar.
3. Type a date and time, and type a town or city to pick it (accents and spaces do not matter: `ha noi` finds Hà Nội). Fill in only what you want to change, then **Apply**. The arrow buttons go back to the camera's time and to the photo's own GPS.

<!-- shot: manual-place-picker | Setting the place of a photo: typing three letters finds the town -->

The photos move in the grid, and the date stamp, the info frame, day folders and the exported copy's date follow. Everything is offline, every change is one step of Undo, and the photo file is never changed.

**Places from your phone** ([FR-031](PRD.md#fr-031--borrow-gps-from-a-phone-or-gpx-track)): a camera without GPS (a Lumix GX9, for example) can take the place of a photo your phone took of the same scene a few minutes apart. Set the camera's clock first if it is off (chapter 11). In the settings panel's **Places** card, click **Find places**, choose how far apart in time (5, 10 or 30 minutes) and **Look for places**. Sure matches get their place at once; the others are shown next to the phone photo, ticked, so untick the wrong ones and confirm. Sure matches are also filled in after every best-shot analysis. A borrowed place shows a pin with a tick on the thumbnail and *Borrowed* in the preview; it never replaces a place you set or the photo's own GPS, and Undo takes it back.

**Rename a place** ([FR-005](PRD.md#fr-005--location-stamp-offline)): the settings panel's **Places** card lists every place on your photos. Type the name you want printed, like `Prague` for `Praha 1` or `Grandma's`, and every stamp and frame with that place follows; the arrow button brings the original name back.

<!-- shot: manual-places | The Places card in the settings: every place on your photos, each with a name to print -->

### 10.1 Stars from Lightroom and other apps

When a folder you open was culled in Lightroom, Bridge, Capture One or darktable, the stars and rejects they saved in XMP sidecar files (`IMG_0416.xmp` or `P1000123.RW2.xmp` next to the photo) come in with the photos ([FR-058](PRD.md#fr-058--lightroom-compatibility-through-xmp)). The sidebar says how many. Your own stars and flags in a Fotollection project win over the sidecar, and the sidecar is never changed. Lightroom Classic writes sidecars only for RAW files, and only when *Automatically write changes into XMP* is on (or after *Metadata → Save Metadata to File*).

<!-- chapter: clocks -->

## 11. Camera clocks

A camera set to the wrong time, or still on the time zone of home, puts its photos in the wrong order next to your phone's, and stamps the wrong date ([FR-030](PRD.md#fr-030--camera-clock-offset)).

1. **From a photo (easiest):** open a photo from that camera, and in **Captured** click **Set this camera's clock from a photo…**. Pick the photo from your phone (or another camera) taken at the same moment, then **Correct the clock**.
2. **By hand:** in the settings panel, **Camera clocks** lists every camera in the library. Type how much to add, like `+1:02:00` or `−0:45`, and press Enter. The arrow button sets it back.

Every photo from that camera moves at once, including photos added later: the grid order, best-shot groups, day folders, the date stamp and the exported copy's date. The preview still shows what the camera's clock said, and the photo file is never changed. Two bodies of the same model keep their own clocks when the camera records its serial number. The correction is saved in the project.

<!-- chapter: export -->

## 12. Exporting

Export writes new files for the lab or for sharing; your originals are never changed ([FR-008](PRD.md#fr-008--original-quality-export), [FR-009](PRD.md#fr-009--export-by-date), [FR-010](PRD.md#fr-010--metadata-privacy-on-export), [FR-053](PRD.md#fr-053--export-for-social-media)).

1. **What is exported** is what the gallery shows: the folder or folders selected, the filters (chapter 2), never the photos set aside. From the **Print basket** view, exactly the basket (chapter 4).
2. **For print or for sharing:** at the top of the *Print / export size* card, choose **Print** (full resolution, with the crop, stamp and frame of each photo) or **Share online**, then a share size:
   - **Original size, lighter file**: the same pixels, about half the file size or less;
   - **Facebook · 2048 px** or **Instagram · 1080 px**: the size each site keeps, so it is not shrunk again;
   - **Custom size**: 640–6000 px on the long edge.

   Shared files are in sRGB, so they look the same on every screen.

   **Whole photo in a frame**, under the share size, keeps every photo whole for a feed of one shape ([FR-075](PRD.md#fr-075--fit-a-shared-photo-whole-into-a-ratio)): pick a ratio (**Square · 1:1**, **Portrait · 4:5**, **3:4** or **2:3**, **Landscape · 3:2** or **16:9**, **Story · 9:16**, or **Custom ratio…** with its two numbers), then the fill: **White**, **Black** or **Custom**, with a colour picker and its code. Each photo sits in the middle and nothing is cut off; on Instagram the frame is 1080 px wide. Prints never get it: they keep their print size.
3. **Where:** click **Choose destination…** at the foot of the panel, and pick a folder. Above it, choose **One folder** (every file straight into it) or **By day** (a folder for each day, such as `2026-07-14`); the app remembers the choice on this computer.
4. **How many folders:** with several folders shown, choose **Together** (all together) or **Separately** (each in a subfolder with its name).
5. Above the button, an **estimate** says how large the files will be. Click **Export n photos**. A bar shows the progress, and **Cancel** stops it: the photos already exported stay, and a file half written is removed. A photo that cannot be exported is skipped, and the summary says how many failed.

<!-- shot: estimate | The size estimate above the export button, sharing for Facebook -->

With **By day**, the files go into one folder per day, named by the date (`2026-07-14`), and `undated` for photos without a date. Shared files go into a `Share` folder either way. A file never overwrites another: a second copy gets `_1`, `_2`. Camera details are kept and the **GPS position is always removed**, so a shared photo does not give away where you live.

**One photo:** in the preview, **Export** above the photo goes to the **Export this photo** button, with the same destination.

<!-- chapter: projects -->

## 13. Projects, autosave and undo

Everything you do (stars, flags, the basket, crops, stamps, dates and places, camera clocks) is saved by itself as you go, and comes back when you open the app again ([FR-023](PRD.md#fr-023--projects-autosave-and-undo)). A project keeps one piece of work under a name, to open again later or on another computer.

1. The **Project** block at the top of the sidebar shows the project's name; a dot means *Edited*. Double-click the name to rename it.
2. Its four buttons: **New project**, **Open project…**, **Save** and **Save as…**. The same are in the *File* menu on a Mac, and on **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S** and **Shift+Cmd/Ctrl+S**.
3. A project is a `.fotollection` file. It holds your edits and settings, with the fonts and logos it uses, but not the photos: it points to their folders, so a project copied together with its photos opens the same on another computer. When its folders were moved or renamed, **n folders could not be found** offers **Locate…** for each, or **Later**.
4. Leaving a project with changes asks **Save changes to "…"?**: **Save**, **Don't save** or **Cancel**.

<!-- shot: manual-project | The Project block: the name, and the buttons to start, open and save a project -->

**Undo** takes back the last change with **Cmd/Ctrl+Z**, and **redo** brings it back with **Shift+Cmd/Ctrl+Z** (or **Cmd/Ctrl+Y**), up to 200 steps. On a Mac they are in the *Edit* menu too.

<!-- chapter: raw -->

## 14. Camera RAW

Fotollection opens the RAW files of most cameras as they are, with no conversion to DNG: RW2 (Panasonic), CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus, OM System) and DNG ([FR-016](PRD.md#fr-016--raw-input)).

- Open the folder as usual. Thumbnails and best-shot suggestions appear as fast as for JPEG, because they use the small preview every camera stores inside its RAW files.
- The print preview develops the full photo the first time you open it, which takes about a second; changing a setting afterwards is instant. The export develops each RAW the same way.
- Colours and sharpness are a neutral development (the system's on macOS, with its lens corrections; Fotollection's own elsewhere), not a RAW editor's: Fotollection prepares prints, it does not replace Lightroom.
- **On Linux**, RAW opens through the reader built into Fotollection, with nothing to install.
- **On Windows**, RAW also opens through the reader built into Fotollection, with nothing to install ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md)). If Microsoft's *Raw Image Extension* is installed, the system's codec is a last fallback for a file the built-in reader cannot open.

<!-- chapter: open-with -->

## 15. Opening a photo from Finder, Explorer or your file manager

Fotollection can be the app that opens a photo you double-click ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). It never makes itself the default: you choose it.

- **Once:** right-click a photo → **Open With** → **Fotollection** (macOS), or **Open with** → **Fotollection** (Windows). On Linux, with the `.deb` installed, right-click → **Open With** → **Fotollection** in Files, Dolphin or Nemo; the AppImage is not listed there unless a tool such as AppImageLauncher registers it.
- **Always:** on a Mac, select a photo, press Cmd+I, choose Fotollection under *Open with* and click *Change All*. On Windows, choose **Open with** → **Choose another app**, pick Fotollection and tick *Always use this app*.

The photo opens in the preview straight away. Its folder becomes a tab in the sidebar, with only that folder's photos (not its subfolders), so **← →** go through the folder. Everything works as in any tab: stars, the print basket, crops, export. Click **×** on the tab when you are done; the photos stay on your disk. Opening another photo while Fotollection is running brings the window forward with that photo.

<!-- chapter: camera -->

## 16. Shooting with a camera connected

With a camera plugged in by cable, Fotollection shows what it sees and takes the photo ([FR-062](PRD.md#fr-062--live-view-from-a-camera), [FR-063](PRD.md#fr-063--tethered-capture-over-usb)). For now this works on Windows, with the Sony a6400; the Linux version has no camera capture ([ADR-0034](adr/0034-linux-lean.md)). Other Sony bodies of the same years may work, but they are not checked yet.

1. **On the camera:** *MENU → Setup → USB Connection → PC Remote*. Plug in a data cable and turn the camera on. Close Imaging Edge Remote, since only one app can drive the camera at a time.
2. **Click the camera button** next to *Open folder* in the sidebar. The capture screen opens.
3. **The capture screen:**
   - **The live view.** If a print size is chosen, its shape is drawn over the live view, so you see what stays after *Crop to fill*.
   - **The settings bar** under the live view shows ISO, shutter speed, aperture and file format.
   - **The panel** steps ISO and shutter speed with − / +, and switches between JPEG and RAW (the camera may keep the old format for one more shot). Set the aperture on the camera or the lens.
   - **Capture**, or **Space**, takes the photo. The camera focuses first, so a shot takes about a second. If it cannot focus, the screen says so. One photo at a time: pressing again (or holding Space) while a shot is on its way does nothing.
   - **The shots** line up under the live view, newest first. Click one to open it in the preview.
4. **Close the screen** (Esc). You are in the session's tab, with all its shots. A notice above the grid offers:
   - **Save all…**: choose a folder. The photos move there, nothing is overwritten, and your crops, stamps and ratings stay. If the folder already holds files (Pictures, for one), the photos go into a new folder named after the session inside it.
   - **Delete photos**.

   Until you choose, the photos stay in Fotollection's own data. Closing the tab asks first, and so does **quitting the app**: *Save all…*, *Delete photos* or *Cancel*. If Fotollection ever closes without asking (a crash), the session comes back as a tab the next time it starts.

   Unplugging the camera or turning it off also closes the screen and shows the session's tab.

**RAW shots** are read by Fotollection's built-in RAW reader, so they show in the tab like JPEG ([ADR-0029](adr/0029-raw-through-rawler-then-our-own-decoder.md)); on Windows this is being checked on a PC without Microsoft's Raw Image Extension.

<!-- chapter: updates -->

## 17. Updating to a new version

Fotollection never goes online by itself ([FR-021](PRD.md#fr-021--manual-updates), [FR-061](PRD.md#fr-061--check-for-updates)).

1. Click the version number at the foot of the settings panel, then **Check for updates**. This is the only time the app goes online.
2. If a newer version exists, click **Download the installer**: your browser downloads the right one for your computer.
3. Install it over the current version. Your settings, camera profiles and projects stay.
   - **Mac:** open the disk image and drag Fotollection into Applications, then choose **Replace**.
   - **Windows:** run the `-setup.exe`; keep the first choice, *Uninstall before installing*. If you installed with the `.msi`, update with the `.msi`: mixing the two leaves two copies.
   - **Linux AppImage:** the button downloads the new AppImage; make it executable (`chmod +x`) and use it in place of the old file.
   - **Linux `.deb`:** the button opens the release page; download the new `.deb` and run `sudo apt install ./Fotollection_*_amd64.deb` (`_arm64.deb` on ARM).

The same popover holds the **Language** choice (chapter 1) and shows the **cache**: the thumbnails and best-shot analyses Fotollection keeps so folders open fast, with its size against its 2 GB limit. Past the limit, the oldest go by themselves. **Clear cache** empties it at any time; nothing of yours is lost, and thumbnails come back as you browse.

**Something wrong, or an idea?** Click the version number at the foot of the panel, then the red **Report a problem or an idea**: it opens the project's page for problems and ideas on GitHub, in your browser ([FR-069](PRD.md#fr-069--report-a-problem)). Say what you did and what you saw, then click **Copy diagnostics** and paste it into the report: it holds your version, your system and the last error codes.

**An error code.** Every error message ends with a code such as `ERR-IMG-0006`. The first letters say what kind of problem it is: `IMG` the photo file itself, `FS` a file or folder the app could not read or write, `EXT` a codec missing from the system, `DRV` the camera's connection, `CFG` a setting, font or logo, `EXP` the export, `NET` *Check for updates*, `RUS` the app itself. The × on an error hides it; it comes back only if the same thing goes wrong again on a new try. Each code is explained, with what to do, in [ERROR_CONTROL.md](ERROR_CONTROL.md). The app keeps a small log of these errors on your computer (no photos, places or file names) only so *Copy diagnostics* can quote them; it sends nothing.

<!-- chapter: simple -->

## 18. Simple mode

*New in v0.9.0.* The shortest way from a folder to prints: look at each photo once, keep it or drop it, then export what you kept ([FR-048](PRD.md#fr-048--simple-collage-and-professional-modes), [FR-050](PRD.md#fr-050--print-basket)).

1. Click **Simple** at the top of the sidebar, and open a folder if the library is empty.
2. The first photo fills the middle, **as it will print**, with your print size, date stamp and border.
3. Press **↑** or click **Keep**: the photo goes into the print basket, and the next one comes.
4. Press **↓** or click **Drop**: the photo is set aside, never deleted, and the next one comes.
5. **← →** step back and forward without deciding. **Cmd/Ctrl+Z** undoes the last keep or drop. The culling keys of the preview work too: **0**–**5** stars, **P** / **X** / **U** flags, **[** / **]** quarter turns.
6. The bar above the photo counts where you are, and how many you kept and dropped.
7. In the panel on the right, choose **Print** or **Share online**, the **print size**, the **3 mm border** and the **date stamp** (format, corner, colour, size, place and your own line, as in Pro), then **Choose destination…** and **Export**: it saves the photos you kept.

<!-- shot: simple | Simple mode: the photo as it will print, Keep and Drop, and Export of the kept photos -->

A photo kept in Simple mode is in the print basket in Pro too, so you can switch to Pro at any time to crop or frame a photo, and come back.

<!-- chapter: collage -->

## 19. Collage mode

*New in v0.9.0.* Two photos on one print, like a half-frame film camera put two frames on one negative: a 10 × 15 print holds two 10 × 7.5 photos, so a set costs half as much ([FR-035](PRD.md#fr-035--multi-photo-layouts)).

1. Click **Collage** at the top of the sidebar. Choose what to put on sheets in the sidebar, usually the **Print basket**.
2. The photos are paired in order, 1 + 2, 3 + 4…, and each sheet shows as it will print. An odd last photo leaves the other half white.
3. **Sheet size**: the print size, 10 × 15 for example; Original is not offered, as a sheet needs a fixed size. The sheet is split in half across its long edge and turns to suit its first photo: two landscape photos one above the other, or two portrait photos side by side. The **turn** button on a sheet, or **R**, turns it the other way. **White** or **Black** under *3 mm border* colours the paper round the photos as well as the border.
4. **Two photos per sheet**:
   - **Fill each half**: no white at all. Each photo fills its half (7.5 × 10 on a 10 × 15) and the two meet edge to edge; crop each photo (step 7) to choose what stays, by dragging and zooming. The settings below, the cut line included, are then hidden.
   - **Photo ratio**: each photo's own (**Photo**), the sheet's (**Sheet**, 2:3 on a 10 × 15), **4:3** or **1:1**. The rest of the half stays white.
   - **Photo size**: the app suggests the largest size that leaves 5 mm of white round the photo, for example 6 × 9 cm for a 2:3 photo in a 7.5 × 10 half. Type another size if you like; with the **lock** on, one edge gives the other. **Use the suggestion** goes back.
   - **Cut line**: a thin line between the photos, to cut the print in two. Off, there is no line, and the photos keep the white between them.
   - The note under them gives the photo's size, the part left inside the **3 mm border** (chapter 5) and its sharpness.
5. **Reorder:** drag a photo onto another photo to swap them, or onto an empty half to send it to the end. **← →** choose a photo, **⌥/Alt ← →** move it one place. Reordering starts every sheet the way its first photo suits, and a photo added later joins at the end.
   - **Auto turn** (the wand above the sheets), for prints you cut in two: every sheet turns landscape and each photo turns to stand, so no standing photo sits next to a lying one. **Revert** brings every sheet and photo back to its own orientation. Each is one undo step. The **ⓘ** button next to the count lists the keys.
6. **Choose the photos to print**, as in the gallery (chapter 4): **Select** ticks whole sheets with a click, both photos at once, framed as one (**Cmd/Ctrl+A** all, **Enter** the chosen sheet), and the bar puts their photos in the **Print basket**, sets them aside or brings them back. While sheets are ticked, the export button says *Export 2 selected sheets* and writes only those. Photos in the basket carry its badge. Choose **Print basket** in the sidebar to make sheets of those photos only.
7. **Preview:** double-click a sheet, or press **Enter** or **Space**, to open it large in the preview, as a photo opens from the gallery. There:
   - the sheet opens with no photo chosen, exactly as it prints. Click a photo (or **1**, **2**) to choose it, and click it again (or **Esc**) to let go;
   - **← →** go to the other sheets; with a photo chosen, **↑** puts it in the print basket and **↓** sets it aside (or the basket button next to it), and the zoom buttons or **Cmd/Ctrl + / −** zoom in;
   - **Crop** (or **C**, or the crop button next to a photo) crops and straightens the chosen photo for its place on the sheet; this never changes its print, and Reset centres it;
   - **Turn the sheet** (**R**) turns it the other way;
   - **Rotate photo** (**⇧R**) turns the chosen photo a quarter in its half, on this sheet only, and the sheet stays as it is (**Turn the sheet** turns the sheet): when a landscape photo shares a sheet with a portrait one, it then fills its half instead of sitting small in the middle. Its own print does not change;
   - the panel lists the two photos, holds the same *Two photos per sheet* and *Date stamp* cards, and **Export this sheet** saves this one sheet. Under *Line under the date*, turn **Same on both photos** off to type a line for each photo; the rest of the stamp stays the same for both;
8. **Export** in the panel writes one file per sheet, named after its two photos (for example `P1000123+P1000124.jpg`), in the first photo's day folder when you export by day.

<!-- shot: collage | Collage mode: sheets as they will print, with the photo ratio, the suggested size and the cut line -->

<!-- shot: manual-collage-preview | A sheet’s preview: the sheet large, its two photos, the Two photos per sheet card and Export this sheet -->

Every change is one undo step. The date stamp prints on each photo, with its place and your own line if you chose them; the info frame is not used on sheets. Collage mode also makes postcards: chapter 20.

<!-- chapter: postcards -->

## 20. Postcards

*New in v0.9.4.* A kiosk print becomes a postcard you can post: one photo on one half, the back of a postcard on the other ([FR-073](PRD.md#fr-073--postcard-sheets)).

1. In Collage mode, turn on the **Postcard** switch in the bar above the sheets, next to **Select** (off goes back to two photos a sheet). Each photo now makes its own sheet: 50 photos are 25 sheets of two, or 50 postcards. The **Postcard** card appears in the panel.
2. **Held**: **Landscape** puts the photo on the left and the card on the right; **Portrait** puts the card on top and the photo below. A dashed **stamp box** sits at the top right, smaller than most stamps so the stamp hides it, and the band across the top stays free for more stamps or a long label printed at the post office. The **writing lines** sit together at the foot: **3 lines** or **4 lines** for the message, then 3 for the address, one under the other on a landscape card and side by side on a portrait one (message left, address right).
3. The photo half works as before: ratio, size, *Fill each half*, the 3 mm border, the date stamp, **Crop** and **Rotate photo** in the preview. A postcard has no cut line and does not turn; Auto turn is hidden.
4. **Text on the postcard**, the card under it, or leave the lines blank to write by hand. Choose a postcard (click it, or open its preview) and type its **Message** (up to 400 characters, on the top lines) and its **Address** (up to 200 characters; on its 3 lines, or 4 smaller ones in the same room when it is long). **B**, *I* and U next to each set that part bold, italic or underlined on every card. The same card sets how every postcard prints its text: **Font**, the typewriter face *Xanh Mono*, the handwriting face *Patrick Hand* or one of your own (**Import OTF/TTF…**, as in chapter 9), which has the letters of every European language written in Latin script (Greek or Cyrillic come from the app's other font), so a copied address prints as typed; **Size**, small, medium or large; **Ink**, **Black** (white on black paper), **Blue**, **Sepia** or **Red**; and **Credit line (optional)**, a small line at the foot of every card, for example *Photo © Anna Schmidt*. What does not fit its lines is left out of the print, as the preview shows, and a red **!** marks the card in the grid and in its preview, and the part that is cut: shorten it, or choose a smaller size.
5. **Export** writes one file per postcard, named after its photo (for example `P1000123-postcard.jpg`).

**Writing by hand.** A print's face is a coating that many pens do not take. Use a fine permanent marker (Sharpie Ultra Fine, edding 400, Stabilo OHPen S), try it in the stamp box first, and let it dry for about a minute; ballpoint pens often skip and gel or fineliner pens smear. Typed text has none of these problems.

**Posting it.** A 10 × 15 print is 15.2 × 10.2 cm: Deutsche Post takes postcards from 14 × 9 to 23.5 × 12.5 cm and 150–500 g/m² (photo paper is about 240), and Magyar Posta's rules give 9 × 14 to 12 × 23.5 cm (to check at the counter). Česká pošta allows postcards only up to 14.8 cm long, so there it goes as a letter. The post may print a barcode along the bottom edge, which can rub off a glossy print; ask at the counter when in doubt.

**Glossy paper and the post.** The postmark on the stamp itself is fine, as a stamp is paper, but where it runs onto the glossy coating its ink may smear before it dries, and sorting machines can scuff the photo's face. If the shop lets you choose, print postcards on **matt** paper; use a self-adhesive stamp, as a gummed one may not hold on the coating; or post it in an envelope, as a letter. Try one card before sending a whole set.

<!-- shot: manual-postcard | Collage mode with Postcard on: trams by night in Prague, each print one photo filling its half and a postcard back with a stamp box and writing lines -->

<!-- shot: postcard-night | A postcard’s preview: a night tram, the typed message and address to the National Museum, the Postcard text card -->

<!-- chapter: shortcuts -->

## 21. Keyboard shortcuts

| Where | Key | Action |
|-------|-----|--------|
| Gallery | **← → ↑ ↓** | Move the selection |
| Gallery | **Enter**, **Space**, double-click | Open the preview |
| Gallery, preview | **0**–**5** | Stars |
| Gallery, preview | **P** / **X** / **U** | Pick / reject / clear the flag |
| Gallery, preview | **[** / **]** | Turn a quarter left / right |
| Preview | **← →** | Previous / next photo |
| Preview | **↑** | Put in the print basket, next photo |
| Preview | **↓** | Set aside, next photo |
| Simple mode | **↑** / **↓** | Keep (print basket) / drop (set aside), next photo |
| Simple mode | **← →** | Previous / next photo, without deciding |
| Collage mode | **← →**, **↑ ↓** | Choose a photo, a sheet above or below |
| Collage mode | **⌥/Alt ← →** | Move the chosen photo one place |
| Collage mode | **Enter**, **Space**, double-click | Open the sheet's preview |
| Collage mode | **R** | Turn the sheet |
| Collage mode, sheet preview | **⇧R** | Rotate the chosen photo in its half |
| Sheet preview | **← →** / **1**, **2** | Other sheets / choose a photo |
| Sheet preview | **C** / **R** | Crop the chosen photo / turn the sheet |
| Sheet preview | **↑** / **↓** | Put the chosen photo in the print basket / set it aside |
| Preview | **C** | Open the crop editor (chapter 6) |
| Preview | **Cmd/Ctrl + / −**, trackpad pinch, the zoom buttons | Zoom in / out (§7) |
| Preview | **Cmd/Ctrl 0**, the fit button | Back to fit |
| Preview | double-click the photo | Toggle fit and 200 % |
| Preview | drag the photo (not zoomed) | Choose what the print keeps (*Crop to fill*) |
| Preview | **Esc** | Leave the crop editor, then close |
| Crop editor | **← →**, **⌥ ← →** | Straighten by one degree, by a tenth |
| Crop editor | **↑ ↓** (with **Shift**, further) | Move the crop up or down |
| Crop editor | **+** / **−** | Zoom in / out |
| Crop editor | **0** | Centre the crop and set the angle back to 0° |
| Crop editor | **Enter** | Done |
| Capture screen | **Space** | Take a photo |
| Capture screen | **Esc** | Close and show the session's tab |
| Select mode | click, **Shift**+click | Tick one photo, tick a range |
| Select mode | **Cmd/Ctrl+A** | Tick every photo shown |
| Select mode | **Enter** | Tick the selected photo |
| Select mode (Pro) | hold **Space** | Quick-preview; **← →** next, **Enter** tick, release to close |
| Select mode | **Esc** | Leave select mode |
| Everywhere | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** or **Cmd/Ctrl+Y** | Undo, redo (chapter 13) |
| Everywhere | **Cmd/Ctrl+1**, **2**, **3** | Simple, Collage, Pro (chapter 1) |
| Everywhere | **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S** | New project, open, save, save as (chapter 13) |

A focused field or slider keeps its own arrow keys: click the photo to give them back to browsing.
