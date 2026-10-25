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

## 3. Camera RAW

Fotollection opens the RAW files of most cameras as they are, with no conversion to DNG: RW2 (Panasonic), CR2 and CR3 (Canon), NEF (Nikon), ARW (Sony), RAF (Fujifilm), ORF (Olympus, OM System) and DNG ([FR-016](PRD.md#fr-016--raw-input)).

- Open the folder as usual. Thumbnails and best-shot suggestions appear as fast as for JPEG, because they use the small preview every camera stores inside its RAW files.
- The print preview develops the full photo the first time you open it, which takes about a second; changing a setting afterwards is instant. The export develops each RAW the same way.
- Colours and sharpness are the operating system's neutral development, not a RAW editor's: Fotollection prepares prints, it does not replace Lightroom.
- **On Windows**, install Microsoft's free *Raw Image Extension* from the Microsoft Store first. Without it, RAW files are skipped and the hint says so.

## 4. Keyboard shortcuts

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
| Select mode | click, **Shift**+click | Tick one photo, tick a range |
| Select mode | **Cmd/Ctrl+A** | Tick every photo shown |
| Select mode | **Enter** | Tick the selected photo |
| Select mode | hold **Space** | Quick-preview; **← →** next, **Enter** tick, release to close |
| Select mode | **Esc** | Leave select mode |
| Everywhere | **Cmd/Ctrl+Z**, **Shift+Cmd/Ctrl+Z** | Undo, redo |
| Everywhere | **Cmd/Ctrl+S**, **Shift+Cmd/Ctrl+S**, **Cmd/Ctrl+O** | Save, Save As, Open a project |

A focused field or slider keeps its own arrow keys: click the photo to give them back to browsing.
