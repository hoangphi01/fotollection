# User Guide — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-30
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
5. **The Print settings panel** on the right holds, top to bottom: *Language*, *Print / export size* (chapter 5), *Date stamp* (chapter 8), *Info frame* (chapter 9), *Camera clocks* (chapter 11) and *Places* (chapter 10) once the library has them, then the export controls (chapter 12) and the version number (chapter 17).

<!-- shot: grid | The library, culled with stars and picks -->

**Language and look:** choose *English* or *Tiếng Việt* in the **Language** card at the top of the panel. The three buttons at the foot of the sidebar set the look: **Follow the system**, **Light** or **Dark** ([FR-020](PRD.md#fr-020--localization), [FR-052](PRD.md#fr-052--appearance-theme-title-bar-and-hints)).

<!-- chapter: cull -->

## 2. Stars, flags and filters

Mark photos as you look through them, then show only the ones you want ([FR-022](PRD.md#fr-022--culling-rate-flag-filter)). Stars and flags never change the photo file; they are saved in the project.

1. Select a photo in the gallery, or open it in the preview.
2. Press **1**–**5** to give it stars, **0** to clear them.
3. Press **P** to pick it, **X** to reject it, **U** to clear its flag. A rejected photo stays in the gallery, faded, until you filter it out; nothing is deleted.
4. In the preview, the **Rating & flag** card does the same with the mouse: click a star (click it again to clear), **Pick (P)** or **Reject (X)**.
5. Filter the gallery with the two lists at the right of the toolbar:
   - flags: **All**, **Picks**, **Unflagged** or **Rejects**;
   - stars: **Any rating**, or **≥ 1** to **≥ 5**.

   An active filter is outlined in the theme colour, and the count shows how many photos are shown out of how many. When nothing matches, **Show all photos** clears the filters.

<!-- shot: manual-filters | The flag filter open above the gallery: All, Picks, Unflagged, Rejects -->

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
- **Export follows the view** (chapter 12). From *Print basket*, *Export* writes exactly the basket, with the size estimate above the button ([FR-053](PRD.md#fr-053--export-for-social-media)). From *All photos* or a folder, photos set aside are never exported.

<!-- shot: estimate | The size estimate above the export button, sharing for Facebook -->

<!-- chapter: size -->

## 5. Print size and sharpness

Choose the size you will print at, and Fotollection crops every photo to it and tells you whether each one is sharp enough ([FR-026](PRD.md#fr-026--resolution-warning), [FR-043](PRD.md#fr-043--custom-print-sizes), [FR-045](PRD.md#fr-045--original-aspect-ratio)).

1. In the **Print / export size** card, keep **Print** selected (for **Share online**, see chapter 12).
2. Open the size list:
   - **Standard (CEWE / EU)**: the sizes of the photo kiosks at dm and Rossmann, like *10 × 15 cm · 2:3*;
   - **Original**: the photo's own ratio, with no crop;
   - **Custom**: sizes you added.

   Each size carries a badge for the photo selected in the gallery: **Sharp** (300 DPI or more), **Acceptable** (150–299 DPI, fine at arm's length) or **Pixelated** (under 150 DPI). Hover over it to see the DPI.
3. Under the list, a line says how large the selected photo prints sharp, and how large it is still acceptable.
4. Every photo is cropped to the size's ratio, **centred by default**. Choose what stays per photo in the preview (chapter 6).

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

1. In the **Date stamp** card, the switch turns it on or off for every photo.
2. Choose:
   - the **format**: `2026-07-14`, `14.07.2026` or `'26 07 14`, the film-camera style;
   - the **corner**: *Bottom right*, *Bottom left*, *Top right* or *Top left*;
   - the **colour**: **White** or **Retro orange**.
3. **Place on the date line** adds the town where the photo was taken, from its GPS, offline: as **City**, **City, HU** (with the country code) or **City, country**. A photo without GPS can get a place by hand (chapter 10).
4. **Line under the date**: a short text across the photo, up to 80 characters, printed in capitals without accents (for example *KY NIEM BUDAPEST*).

<!-- shot: stamp | Date stamp with the town from the photo's GPS -->

**One photo differently:** in the preview, the **Date stamp** card holds the same controls for that photo alone; it is then marked **This photo**. **Apply to all photos** makes this photo's stamp the one for every photo. In the panel, **Reset n photos with their own stamp** makes every photo follow the panel again. Both can be undone.

The date is the one from the camera, corrected by the camera clock (chapter 11) or set by hand (chapter 10).

<!-- chapter: frame -->

## 9. Info frame "Shot on"

A white or black bar under the photo with the camera's logo and settings, like the photos shared from phones ([FR-040](PRD.md#fr-040--exif-info-frame), [FR-065](PRD.md#fr-065--border-in-crop-to-fill)). It is off by default, and it is the same for every photo.

1. Turn on the **Info frame** card.
2. Choose how it fits the print:
   - **Crop to fill**: the print keeps its exact size, and the bar takes its place inside it; drag the photo in the preview to choose what stays. **Add a border** puts a 3 mm border in the frame colour on the top and sides, still inside the print size, like an instant photo;
   - **Fit whole photo**: the whole photo at the print size, framed like an instax print with a 5 mm border on three sides and the bar below, so the print comes out larger than the size chosen. A note says how large, so you can pick a paper that fits at the shop.
3. Choose **White** or **Black**, the date format, and the font: *Inter (built-in)*, or **Import OTF/TTF…** to use your own ([FR-042](PRD.md#fr-042--custom-fonts)).
4. Under **Show**, tick what the bar holds: *Logo*, *Camera*, *Lens*, *Focal length*, *Aperture*, *Shutter*, *ISO*, *Photo style*, *Date*, *Place*.
5. With the logo shown, choose its **Logo colours** (*Classic*, *Brand colours* or *One colour*) and its **Logo size** (double-click the slider for 100 %).

<!-- shot: frame | The “Shot on” info frame with the camera's logo -->

<!-- shot: manual-frame | The Info frame card: fit, border, colour, font and the fields to show -->

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
3. **Where:** click **Choose destination…** at the foot of the panel, and pick a folder.
4. **How many folders:** with several folders shown, choose **Together** (into the same date folders) or **Separately** (each in a subfolder with its name).
5. Above the button, an **estimate** says how large the files will be. Click **Export n photos**. A bar shows the progress, and **Cancel** stops it: the photos already exported stay, and a file half written is removed. A photo that cannot be exported is skipped, and the summary says how many failed.

<!-- shot: estimate | The size estimate above the export button, sharing for Facebook -->

The files go into one folder per day, named by the date (`2026-07-14`), and `undated` for photos without a date; shared files go into a `Share` folder. A file never overwrites another: a second copy gets `_1`, `_2`. Camera details are kept and the **GPS position is always removed**, so a shared photo does not give away where you live.

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
- Colours and sharpness are the operating system's neutral development, not a RAW editor's: Fotollection prepares prints, it does not replace Lightroom.
- **On Windows**, install Microsoft's free *Raw Image Extension* from the Microsoft Store first. Without it, RAW files are skipped and the hint says so. A RAW reader built into Fotollection, which needs no extension, is being built ([STATUS.md](STATUS.md)).

<!-- chapter: open-with -->

## 15. Opening a photo from Finder or Explorer

Fotollection can be the app that opens a photo you double-click ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). It never makes itself the default: you choose it.

- **Once:** right-click a photo → **Open With** → **Fotollection** (macOS), or **Open with** → **Fotollection** (Windows).
- **Always:** on a Mac, select a photo, press Cmd+I, choose Fotollection under *Open with* and click *Change All*. On Windows, choose **Open with** → **Choose another app**, pick Fotollection and tick *Always use this app*.

The photo opens in the preview straight away. Its folder becomes a tab in the sidebar, with only that folder's photos (not its subfolders), so **← →** go through the folder. Everything works as in any tab: stars, the print basket, crops, export. Click **×** on the tab when you are done; the photos stay on your disk. Opening another photo while Fotollection is running brings the window forward with that photo.

<!-- chapter: camera -->

## 16. Shooting with a camera connected

With a camera plugged in by cable, Fotollection shows what it sees and takes the photo ([FR-062](PRD.md#fr-062--live-view-from-a-camera), [FR-063](PRD.md#fr-063--tethered-capture-over-usb)). For now this works on Windows, with the Sony a6400. Other Sony bodies of the same years may work, but they are not checked yet.

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

The same popover shows the **cache**: the thumbnails and best-shot analyses Fotollection keeps so folders open fast, with its size against its 2 GB limit. Past the limit, the oldest go by themselves. **Clear cache** empties it at any time; nothing of yours is lost, and thumbnails come back as you browse.

<!-- chapter: shortcuts -->

## 18. Keyboard shortcuts

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
| Select mode | hold **Space** | Quick-preview; **← →** next, **Enter** tick, release to close |
| Select mode | **Esc** | Leave select mode |
| Everywhere | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** or **Cmd/Ctrl+Y** | Undo, redo (chapter 13) |
| Everywhere | **Cmd/Ctrl+N**, **Cmd/Ctrl+O**, **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S** | New project, open, save, save as (chapter 13) |

A focused field or slider keeps its own arrow keys: click the photo to give them back to browsing.
