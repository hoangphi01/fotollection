# User Guide — Fotollection

> **Status:** Living · **Last reviewed:** 2026-10-25
>
> How to use the app, step by step, for the person at the keyboard: the source for the user manual and the website's help. It describes behaviour that has shipped or is on `main`; the reasons and the full rules are in [PRD.md](PRD.md), linked from each section. Update it in the same commit as any change a user would notice ([DOCS_GUIDE.md](DOCS_GUIDE.md)).

## 1. Choosing the photos to print

A trip gives hundreds of photos; only a handful go to the lab. Fotollection keeps that choice in two **holding areas**, next to the gallery ([FR-050](PRD.md#fr-050--print-basket)):

| Area | What it holds | Where it is |
|------|---------------|-------------|
| **Print basket** | The photos you will print | Sidebar, under *All photos*, with its count |
| **Set aside** | Photos you put away for now. They leave the gallery but are **never deleted or moved on disk** | Sidebar, under *Print basket*, with its count |

A photo is in the gallery, in the print basket, or set aside, never in two at once. Photos in the basket stay visible in the gallery, with a small basket badge in the top-right corner of their thumbnail; photos set aside do not appear in *All photos* or in the folders until you bring them back. Stars and pick/reject flags are separate: a pick does not have to be printed, and the basket is not a flag.

Both areas are saved in the project (`.fotollection`) and in the session, so they are still there after you quit and reopen the app. Every move can be undone with **Cmd/Ctrl+Z**.

### 1.1 Workflow A: decide photo by photo in the preview

1. Open a photo: double-click it, or select it and press **Enter** (or **Space**).
2. Look at it with the print settings applied.
3. Press **↑** to put it in the **print basket**: the next photo opens straight away.
4. Press **↓** to **set it aside**: it leaves the gallery, and the next photo takes its place.
5. Press **→** to move on without deciding, **←** to go back.
6. Repeat until the end of the trip. **Esc** closes the preview.

The **Print basket** button on the photo (next to *Crop*, the turn buttons and *Export*) does the same as ↑, and shows *In the basket* when the photo is already in it; click it again to take the photo out. A photo that was set aside shows **Back to the gallery** instead.

When you open the preview from the *Print basket* or *Set aside* view, ↑ and ↓ move the photo to the other area; a photo that stays where it is moves on to the next one.

### 1.2 Workflow B: tick photos in the gallery (select mode)

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

**Several folders:** ticks follow you from one folder tab to another ([FR-056](PRD.md#fr-056--folders-as-tabs)). Tick a few photos in one folder, click another folder in the sidebar and keep ticking; the bar shows the total and how many are in other folders, and *Print basket* moves them all at once.

In select mode, a double-click does not open the preview, and **Space works differently** (§1.3).

### 1.3 Quick-preview: look without opening (select mode only)

In select mode, **hold Space** over a photo to see it large, as it will print, with the gallery's print size, date stamp and info frame applied ([FR-059](PRD.md#fr-059--quick-preview)). **Release Space** to close it, like Quick Look on a Mac.

- It shows the photo under the mouse pointer; if the pointer is not over a photo, the last photo you ticked.
- The thumbnail appears at once and the full render replaces it a moment later.
- While Space is held: **← →** show the neighbouring photos, **Enter** ticks or unticks the photo shown, **Esc** closes.
- Nothing can be edited here. To change the crop, stamp or frame of one photo, open the normal preview (double-click, or **Enter** outside select mode).

Outside select mode, Space keeps opening the normal preview.

### 1.4 Looking again and exporting

- Click **Print basket** or **Set aside** in the sidebar to see only those photos. Change your mind with ↑ ↓ in the preview, or with select mode and **Back to the gallery**.
- Click **All photos**, or a folder, to return to the gallery.
- **Export follows the view.** From *Print basket*, *Export* writes exactly the basket, with the size estimate above the button ([FR-053](PRD.md#fr-053--export-for-social-media)). From *All photos* or a folder, photos set aside are never exported.

## 2. Looking closely: zoom

In the preview, the **zoom buttons** at the bottom right of the photo show the zoom level between **−** and **+**; the last button goes back to fit ([FR-057](PRD.md#fr-057--zoom-in-the-preview)). The same with the keyboard and the trackpad:

- **Pinch** on the trackpad, or **Cmd/Ctrl +** and **Cmd/Ctrl −**, zoom about the pointer, up to 800 %.
- **Double-click** the photo to go between fit and 200 %.
- Zoomed in, **drag** the photo, or scroll with two fingers, to move around it.
- **Cmd/Ctrl 0** or the fit button goes back to the whole photo.

The preview is first shown at a light size that any computer draws quickly. When you zoom past what it holds, a small spinner appears beside the zoom level, and a moment after you stop moving the part on screen is replaced by the **full-resolution** pixels, exactly as the exported file will have them. Only the visible part is loaded, so even a 32 MP photo stays light.

## 3. Opening a photo from Finder or Explorer

Fotollection can be the app that opens a photo you double-click ([FR-054](PRD.md#fr-054--open-photos-with-fotollection)). It never makes itself the default: you choose it.

- **Once:** right-click a photo → **Open With** → **Fotollection** (macOS), or **Open with** → **Fotollection** (Windows).
- **Always:** on a Mac, select a photo, press Cmd+I, choose Fotollection under *Open with* and click *Change All*. On Windows, choose **Open with** → **Choose another app**, pick Fotollection and tick *Always use this app*.

The photo opens in the preview straight away. Its folder becomes a tab in the sidebar, with only that folder's photos (not its subfolders), so **← →** go through the folder. Everything works as in any tab: stars, the print basket, crops, export. Click **×** on the tab when you are done; the photos stay on your disk. Opening another photo while Fotollection is running brings the window forward with that photo.

## 4. Camera RAW

Fotollection opens the RAW files of most cameras as they are, with no conversion to DNG: RW2 (Panasonic), CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus, OM System) and DNG ([FR-016](PRD.md#fr-016--raw-input)).

- Open the folder as usual. Thumbnails and best-shot suggestions appear as fast as for JPEG, because they use the small preview every camera stores inside its RAW files.
- The print preview develops the full photo the first time you open it, which takes about a second; changing a setting afterwards is instant. The export develops each RAW the same way.
- Colours and sharpness are the operating system's neutral development, not a RAW editor's: Fotollection prepares prints, it does not replace Lightroom.
- **On Windows**, install Microsoft's free *Raw Image Extension* from the Microsoft Store first. Without it, RAW files are skipped and the hint says so. A RAW reader built into Fotollection, which needs no extension, is being built ([STATUS.md](STATUS.md)).

## 5. Updating to a new version

Fotollection never goes online by itself ([FR-021](PRD.md#fr-021--manual-updates), [FR-061](PRD.md#fr-061--check-for-updates)).

1. Click the version number at the foot of the settings panel, then **Check for updates**. This is the only time the app goes online.
2. If a newer version exists, click **Download the installer**: your browser downloads the right one for your computer.
3. Install it over the current version. Your settings, camera profiles and projects stay.
   - **Mac:** open the disk image and drag Fotollection into Applications, then choose **Replace**.
   - **Windows:** run the `-setup.exe`; keep the first choice, *Uninstall before installing*. If you installed with the `.msi`, update with the `.msi`: mixing the two leaves two copies.

The same popover shows the **cache**: the thumbnails and best-shot analyses Fotollection keeps so folders open fast, with its size against its 2 GB limit. Past the limit, the oldest go by themselves. **Clear cache** empties it at any time; nothing of yours is lost, and thumbnails come back as you browse.

## 6. Camera clocks

A camera set to the wrong time, or still on the time zone of home, puts its photos in the wrong order next to your phone's, and stamps the wrong date ([FR-030](PRD.md#fr-030--camera-clock-offset)).

1. **From a photo (easiest):** open a photo from that camera, and in **Captured** click **Set this camera's clock from a photo…**. Pick the photo from your phone (or another camera) taken at the same moment, then **Correct the clock**.
2. **By hand:** in the settings panel, **Camera clocks** lists every camera in the library. Type how much to add, like `+1:02:00` or `−0:45`, and press Enter. The arrow button sets it back.

Every photo from that camera moves at once, including photos added later: the grid order, best-shot groups, day folders, the date stamp and the exported copy's date. The preview still shows what the camera's clock said, and the photo file is never changed. Two bodies of the same model keep their own clocks when the camera records its serial number. The correction is saved in the project.

## 7. Dates and places

**Set them by hand** ([FR-032](PRD.md#fr-032--manual-date-and-place)), for scanned prints, a camera without a clock, or a photo without GPS:

1. **One photo:** in the preview, click **Set the date…** in *Captured* or **Set the place…** in *Location*.
2. **Many photos:** in select mode, tick them and click **Date and place…** in the selection bar.
3. Type a date and time, and type a town or city to pick it (accents and spaces do not matter: `ha noi` finds Hà Nội). Fill in only what you want to change, then **Apply**. The arrow buttons go back to the camera's time and to the photo's own GPS.

The photos move in the grid, and the date stamp, the info frame, day folders and the exported copy's date follow. Everything is offline, every change is one step of Undo, and the photo file is never changed.

**Rename a place** ([FR-005](PRD.md#fr-005--location-stamp-offline)): the settings panel's **Places** card lists every place on your photos. Type the name you want printed, like `Prague` for `Praha 1` or `Grandma's`, and every stamp and frame with that place follows; the arrow button brings the original name back.

## 8. Shooting with a camera connected

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

## 9. Keyboard shortcuts

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
| Preview | **C** | Open the crop editor |
| Preview | **Cmd/Ctrl + / −**, trackpad pinch, the zoom buttons | Zoom in / out (§2) |
| Preview | **Cmd/Ctrl 0**, the fit button | Back to fit |
| Preview | double-click the photo | Toggle fit and 200 % |
| Preview | drag the photo (not zoomed) | Choose what the print keeps (*Crop to fill*) |
| Preview | **Esc** | Leave the crop editor, then close |
| Crop editor | **← →** | Straighten by one degree |
| Capture screen | **Space** | Take a photo |
| Capture screen | **Esc** | Close and show the session's tab |
| Select mode | click, **Shift**+click | Tick one photo, tick a range |
| Select mode | **Cmd/Ctrl+A** | Tick every photo shown |
| Select mode | **Enter** | Tick the selected photo |
| Select mode | hold **Space** | Quick-preview; **← →** next, **Enter** tick, release to close |
| Select mode | **Esc** | Leave select mode |
| Everywhere | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** | Undo, redo |
| Everywhere | **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S**, **Cmd/Ctrl+O** | Save, Save As, Open a project |

A focused field or slider keeps its own arrow keys: click the photo to give them back to browsing.
