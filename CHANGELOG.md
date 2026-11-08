# Changelog

All notable changes to this project are documented in this file.

Each version opens with **Highlights**, a few one-line points shown on its GitHub release page; the full list follows.

Each MINOR series (0.13, 0.14…) opens with a rule and its title, above its first version.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Highlights

- New: **Looks**, one tap gives a photo a colour: eight looks of our own (C200 Natural, Warm old film, Clear, Cool, Faded and three black and white), each shown as your photo, with an **Amount**; **Save as look** keeps your own.
- New: **Lightroom presets** and **LUTs** in Pro mode, in a Presets & LUTs panel like Lightroom's.
- New: Simple mode shows the looks as a numbered strip under the photo: a number key, then ↑, and the photo is coloured and kept.
- UI update: a collage sheet's preview has Edit and Print tabs, and colours the whole sheet when no photo is picked.
- Fixes: RAW shadows without steps.

### Added

- **Looks** (FR-079): a strip of looks under the photo in every mode (in the photo preview a second row over the filmstrip, folding by its header; under a collage sheet; under Simple mode's photo), each look a tile of the photo through it; the black and white looks named *B&W* for short; a **Looks** card first among the colour cards: **Amount** sets how much of the whole look (shown, with the card's reset, only once the photo has a colour); **Save as look** keeps the photo's look under a name on this machine, listed in **Presets & LUTs** (on the left of Pro mode's photo preview, as Lightroom's Presets) to pick again or remove; the ↺ on the card's title gives the photo its original colour back (on a collage sheet with none picked, both photos). A pick copies the look into the photo, so its sliders show it and editing the list later changes no photo. In Simple mode the looks are a strip under the photo, each numbered: **0**–**9** chooses one (0 is Original), then **↑** keeps the photo; there the number keys choose a look instead of rating, as Simple mode shows no stars; its panel has the look's **Amount** (100 % by default) and **Faves** for the photo shown.
- **Import presets…** (FR-089, ADR-0045), Pro mode only: your own Lightroom presets (`.xmp`, `.lrtemplate`, or a `.zip` pack) become looks, read in Rust and never run; the sliders that match ours go to them, and curves, whites and blacks, the eight-hue mixer, colour grading and the black-and-white mixer are baked into the look's one table. A note lists what no look can hold (clarity, texture, dehaze, sharpening…). **Presets & LUTs** works as Lightroom's panel: groups that fold, a filter, **⋯** or a right click to rename, update with the current colour, move or remove, and a look shows on the photo while the pointer is over it.
- **Import LUT…** (FR-070), in **Presets & LUTs**, Pro mode only, the files it takes written under the button: a `.cube` (from LUMIX Lab, Resolve, Photoshop or anywhere) or a HaldCLUT PNG becomes a look of your own, read and checked in Rust and kept by its content, with its own **LUT strength**; a LUMIX LUT made for V-Log footage says so, as it is far too strong on a photo.

### Fixed

- **History** (FR-095) is about six rows high and scrolls inside, so a long one never pushes the column; a look picked says its name (*Look · Warm old film*), the amount its value.
- **The info frame's photo-style line** names the photo's look when the camera profile gives none (FR-040).
- **A RAW's look tiles** show its colour as the preview does, from its standard preview once it exists, not the camera's JPEG (2 to 9 ΔE apart, RESULT §34).
- **Security:** the release build's content security policy now lets the interface read its own photo protocol (`connect-src`), which the GPU look and the hover preview need and only release builds enforce; secret files (service-account keys, private keys) can no longer be added to the repository by mistake.
- **An app, not a web page:** text no longer highlights when clicked or dragged, in dialogs too; only text fields, a photo's facts and error messages can be selected and copied. A right click shows no web menu (Look Up, Translate, Inspect Element…) outside them, and the web view's keys that reload, print or search do nothing.
- **A zoomed photo stays on the stage** (FR-057): dragging, scrolling sideways or zooming about the pointer never moves it off the stage, and a drag that leaves the stage no longer selects the text of the panels beside it.
- **The colour cards no longer jump** when a tab is switched or a sheet's photo picked: the list of looks and each photo's look are drawn at once from the last answer.
- **The histogram is sharp on Windows** (FR-098): its curves are drawn at the display's own scale, so at 125 % or 150 % they no longer look soft; it is a little taller (80 px).
- **RAW shadows without steps:** the RAW tone curve is read between its bins, so dark gradients no longer show bands of three or four levels (35 gaps in the lifted shadows of the nine test cameras, now 1); RAW previews are made again once.

### Changed

- **Collage sheet preview in two tabs** (FR-079, FR-095), as the photo preview: **Edit** (the photos on the sheet, then the colour cards) and **Print and export** (size, postcards, cells, date stamp, export). The looks are a strip under the sheet, folding by its header as the photo preview's, the keys' footer beside it as tall as an open strip, open or folded; the keys are the few used most, every key behind ⓘ in the toolbar. With no photo picked, **Whole sheet** (key **0**) is the first row of the photos, and the colour cards change both photos together: the same look for both, one undo step, a note when their colours differed.

- **Website:** the footer shows the GPLv3 mark with "Fotollection is released under the GPL 3.0", as other free software does; the mark is served from the site.

- **Website:** the treats of *Buy me a coffee* look like the app's choices: raised while off, the chosen one in the theme's colour and pressed in.
- **Website and README:** support also through Ko-fi, a filled button in the theme's colour beside GitHub Sponsors, and in GitHub's *Sponsor* button; a plain link, nothing loaded from Ko-fi. The site's filled buttons now get darker under the pointer, as the app's do. The privacy policy names Ko-fi, GitHub Sponsors and the bank behind VietQR as the processors of support payments, unrelated to the app or your data.

---

## v0.13 — The interface

## [0.13.2] - 2026-11-07 — Develop, part 3

### Highlights

- New: the whole manual inside the app, in Settings, also with no internet
- New: clipping (J) and white balance from a grey spot (W) on collage sheets too
- A careful style pass over all three modes: one size, one spacing and one look for every control, checked by a machine on every view, in both languages and themes

### Added

- **The manual inside the app** (FR-110): Settings → *Manual*, or *Help → Fotollection Manual* on a Mac, shows the whole user guide inside the Settings window, with its pictures, in the app's language, also with no internet; a chapter picker with a search, *Back* and *Next* to turn the chapters.
- **`J` and `W` in Collage mode** (FR-098): in a sheet's preview, `J` (or ⚠ on the histogram) marks both photos' blown and blocked pixels, never the paper or the postcard card; with a photo picked, `W` (or the pipette) and a click on something grey in it set that photo's warmth and tint, as one step. A click on the other photo or the paper is refused.
- **Website:** the Performance page is more visual: every part opens with three numbers as tiles and a two-sentence lead, and two new diagrams show where a photo's preview comes from and how the photo on screen goes first.

### Changed: a careful style pass

Every screen of Simple, Collage and Pro was checked against the app's style rules (STYLE_RULE §4), and what differed was rebuilt from the same shared pieces, so one meaning has one look everywhere. A new set of UI tests now checks this on every view, in English and Vietnamese, light and dark, at the smallest and larger window sizes, so these faults cannot quietly come back.

- **Everywhere**
  - Toolbar buttons are all 32 px squares with 16 px icons, the same height as every field and menu.
  - A button that is switched off still says why when you point at it (undo with nothing to undo, *Paste* with nothing copied); before, its hint never showed.
  - Small buttons use one text size; every list (folders, Settings' sections, export presets, History, menus) has the same row: icon, 8 px, words; the row under the pointer and the chosen row look the same in every list.
  - A filled button gets darker under the pointer, never lighter, and its words stay readable in both themes.
  - The arrows to the previous and next photo or sheet are the same in the photo preview and the sheet preview, drawn on the black stage's colours and shown while the pointer is over it.
  - Names cut short (a file name, a card's title, a long custom size) show in full when you point at them.
- **Gallery and folder list**
  - *Photos / Groups* and the light/dark buttons are the same switch as in Settings.
  - The folder list's blocks line up on one 12 px edge; *Project* and *Library* are headed like every label.
  - A missing folder is a note with a *Locate…* button, and a project error a red note, instead of link-like lines.
  - The selection bar stays on one line, its buttons showing only their icons in a narrow window instead of wrapping.
  - *Filters* is a toolbar button like *Select* beside it, and both look "on" the same way.
  - The small marks on thumbnails (no date, place borrowed, flags, stars, closed eyes) say what they mean when you point at them.
- **Photo preview**
  - Stars, *Pick* and *Reject* are 32 px buttons that press in when on; *Reject* is no longer red, as red is kept for removing things.
  - The zoom buttons are part of the toolbar.
  - The histogram's ⚠ and pipette are flat, as the Navigator's choices; arming **W** shows its instruction inside the histogram card, so nothing below moves.
  - *Apply to all* under Faves and the date stamp is always there, greyed out until the photo has its own setting, so the first slider step never makes the card grow.
  - *No EXIF date* is a warning badge with its icon.
- **Collage mode**
  - The sheet preview has a toolbar above the sheet, as the photo preview does: undo and redo, copy, paste and reset for the chosen photo, *Crop*, *Turn the sheet*, *Rotate photo* and zoom, never over the sheet.
  - Its cards follow the photo preview's order: the sheet's photos, its size, the chosen photo's colour, then the Collage cards; the chosen photo is marked as every chosen row is.
  - *Revert* (turns) is the red reset button, as every reset is; the turn button on a sheet is the full 32 px.
- **Postcards**
  - Bold, Italic and Underline sit under the text field with the character count, so the label is never squeezed.
  - Text that will not fit on the print is a red note.
  - The fonts are named shortly (*Typewriter*, *Handwriting*), the typeface on hover.
  - Under the address, the other cards' state is a note with *Use for them* and *Use for all N*, whose words wrap instead of being cut.
  - *Recent addresses* stay listed on every card, the card's own marked.
  - The ink colours show as dots, each name on hover.
- **Settings, export and dialogs**
  - The export's choices sit under a *Change the export's choices* button, as words that wrap.
  - *Report a problem* keeps its red, from the same block as every red button; *Up to date* is green; facts in *RAM & cache* sit closer.
  - *Don't save* is red, as every discard; the date and place dialog, the place borrowing and the camera screen use the same fields, switches and ticks as the panels.
- **Accessibility (WCAG 2.2 AA)**
  - Every slider's handle has its name for screen readers; the version badge's name says what it shows; the preview's tabs are announced as a group of buttons.

### Fixed

- **A long custom print size** no longer pushes its delete button out of the card.
- **Recent addresses** no longer list the same address several times (each pause while typing was saved as one); an address is remembered once its field is left.
- **Recent addresses** no longer disappear after one is picked.
- **The postcard's "Use for N postcards" lines** are no longer cut at the panel's edge.
- **The postcard ink names** (*Black*, *Sepia*) are no longer cut in Vietnamese and narrow panels.

## [0.13.1] - 2026-11-07 — Develop, part 2

### Highlights

- New: copy, paste and reset a photo's settings, to one photo or every ticked one, as in Lightroom
- New: a histogram above the Navigator, as in Lightroom, that follows your sliders and tells what is blown or blocked
- New: a welcome on the first launch, five pages of the basics over pictures of the app

### Added

- **Copy, paste and reset settings**, as in Lightroom: copy a photo's colour, RAW look, crop position or date stamp (**⇧⌘C**), paste them to another photo (**⇧⌘V**) or to every ticked photo at once (**⌥⇧⌘V**, or *Paste settings* in the selection bar), and reset a photo to the shared settings (**⇧⌘R**). Each is one step of Undo, named in History. The buttons sit after undo and redo above the photo, and in Collage mode in the picked photo's *Settings* card.
- **White balance from a grey spot** (**W**, or the pipette on the histogram): click something that should be neutral and warmth and tint follow, as one step.
- **The left column in a narrow window** now makes room for itself, so the photo fits between the two columns; a button at the start of the toolbar shows it and one on its head, *Overview*, hides it; the toolbar scrolls sideways when its buttons no longer fit.
- **Clipping shown on the photo** (**J**, or the ⚠ on the histogram): blown parts in red, blocked shadows in blue, following your sliders.
- **A welcome** the first time Fotollection opens after it is installed or updated: five pages of the basics, each over a picture of the app, with *Show at every start* and *Start*; Settings → General shows it again.
- **A histogram** of red, green, blue and light, as Lightroom's: at the top of the left column in the window layout, heading the panel in the smaller window, and in Collage mode for the picked photo. It shows the photo with its colour, never the date stamp or frame, follows a slider as you drag, and says how much is blown or blocked.

### Fixed

- **A click on the photo** no longer adds a *Crop* step to History when the photo was not dragged.

### Changed

- **Website:** a *Listed on Unikorn.vn* badge beside VNLF's, both in the page's style; the footer's blog link and *Report a problem* take the site's depth (the red button with a burgundy lip, as the app's); Lightroom and Adobe get their trademark note where Lightroom is first named; the before-and-after picture follows the mouse as it passes, and no longer starts dragging the picture.
- **The develop view's toolbar** starts with undo and redo, apart from the turn buttons, which now show an arrow on a square so they are never taken for undo; *Back to the gallery* has its own icon too.
- **Website:** two new sections on the home page: what Fotollection is made for, in four moments (after a trip, a family album, postcards and themed sets, sharing without losing colour), and how it compares with Lightroom Classic, darktable and RawTherapee.
- **Keep best** now also sets the other shots of each group aside, so the gallery keeps only the shots worth working on, from every folder; *Set aside* holds the others, ready to come back.
- **All photos is now the Workspace:** the photos of every folder still in play, all but those set aside, which is where the best shots of several folders are edited together.
- **The progress strip** while photos are imported or the best shots are looked for is as tall as the heads of the two side columns, its line level with theirs.
- **Website:** the Vietnam Linux Family listing is a badge in VNLF's green beside the download buttons, and the Community card links to the project on VNLF Social too; the badges at the top have the app's depth.
- **Website:** every page has its own notes, numbered from 1 as Apple's are, and every abbreviation (SIMD, EXIF, RAW, MP, p95, HEIC, DPI, GPS, SHA-256, APT…) is explained in a note at its first use.
- **Website:** the before-and-after picture at the top follows a finger on a phone or tablet, and long badges wrap inside their box on a phone.
- **Website:** the Performance page's *Why Fotollection is fast* sets each choice beside what it measures, and its text names Fotollection throughout.
- **Website:** the soft glow at the top of the Performance, Manual and Story pages no longer stops at a hard edge under the title.

## [0.13.0] - 2026-11-06 — The new look

### Highlights

- New: an export dialog as Lightroom's: presets, a quality slider that says what prints well, subfolders, file names, a size limit and metadata choices
- UI update: a new look in Be Vietnam Pro, controls with depth, one height for every control, red for every remove and reset
- UI update: on macOS the title bar takes the app's own colours
- New: the app tells you at start when a newer version is out, with a red warning if yours should not be used any more
- Fixes: *New project* no longer drops unsaved work after a restart

### Added

- **Linux software centres show Fotollection** with its description in English and Vietnamese and its screenshots (AppStream metadata in the `.deb` and the AppImage), as on the VNLF App Explorer and CaramOS's App Center.
- **Website:** a new landing page: the old screen turns into the new one at the top, a *What's new* grid of the new look, *Make it yours* with both themes and languages, *Lightroom's keys* with the develop view and the export dialog, and a *Community* section with VNLF App Explorer, CaramOS and GitHub; its buttons and cards have the app's depth, and every link preview says it is listed on VNLF.
- **Open folder** after an export (FR-093): the finished export is a green box with a button that opens the folder the photos went into.
- **Reorder the folders by dragging** (FR-107), as browser tabs: press and drag, or hold and drag; **Alt+↑ ↓** from the keyboard. The order is saved with the project.
- **Check for updates at start** (FR-103): once when the app opens, a small window tells you a newer version is out, with *What's new* and *Download*; nothing is downloaded by itself. Below a version the release marks as no longer safe, a red warning says why. Turn it off in Settings → Updates.
- **New project asks for its name** (FR-023): a small dialog, *Create* or *Cancel*.
- **An export dialog, as Lightroom's** (FR-093): *Export* opens it, with presets on the left (prints, Facebook, Instagram, original size, custom) and every choice on the right: where the photos go, by day or in one folder, the quality, the size and the names. Tick *Don't ask again* and *Export* exports at once; Settings → *Export* holds the same choices and *Ask every time*. The folder, the quality and the rest are kept for next time.
- **Everything of Lightroom's Export dialog that a print needs** (FR-009, FR-010, FR-104, FR-105): put the export in a **subfolder**; choose what happens to a file already there (**add a number**, **skip**, **replace**); **name the files** as Lightroom's templates (date, original name, your own text and a number), with a live example; **limit the file size** for a lab's upload; keep the **camera details**, the **date only** or **nothing** in the files, GPS always removed.
- **Export quality on a slider** (FR-092), from 60 to 100, prints at **90** by default (95 before: a third larger for a difference no one sees on paper, so a print could weigh more than the camera's own JPEG): the size estimate follows it, and a badge says what it is good for, measured on real prints: *Prints beautifully* from 90, *Prints well* from 75, below that *Screens only*.

### Fixed

- **New project no longer drops unsaved work** (FR-023): an untitled project with photos always asks *Save changes?* first. After a restart it was taken as saved, so *New project* removed its folders without asking.

### Changed

- **Select mode's tick** is a raised circle in the theme's colour, as large as a folder tab's ✕.
- **No flat buttons left on their own** (FR-099): *Select all* (in the theme's colour), dialogs' *Cancel*, *More*, *Show all*, the keys ⓘ and the rest are raised like every button.
- **The smallest text has room for Vietnamese marks** (FR-099): the folder list's tip and other 11 px text no longer crowd on two lines.
- **A seamless title bar on macOS** (FR-052): the traffic lights sit on the app's own colours, as in VS Code, with the project's name in the middle.
- **One height for every control** (FR-099): buttons, toggles and the toolbar are as tall as a text field or a dropdown (32 px), so a row of mixed controls lines up; the Settings gear is a real button, and the author's name sits under the app's, a size smaller; the logo beside them is as tall as the gear and opens fotollection.org. The project's and the library's buttons sit in one raised bar each. A card's header keeps its height when a reset button appears, and sliders read better in the dark theme. A card opened at the foot of a panel scrolls into view. Cards, dialogs, menus and notices stand off the window with the same lip and light as the buttons. Spacing comes in three steps everywhere: 12 px between rows of controls, 8 px between buttons side by side, 4 px between the rows of a list. Reset buttons are red as the remove buttons; menus and dropdowns mark the chosen item as the folder list does; the size estimate is a badge beside the print verdict. Every column's head is 64 px, so the lines under them meet as one line across the window in the gallery, Simple, Collage, the Settings window, the export dialog and the photo preview.
- **Every remove button is red** (FR-099): clearing the filters, the cache or a selection, discarding a camera session, removing a folder, a font, a print size or the logo, stopping an export: deep red with white ink, small icons and a folder tab's round ✕ included, everywhere including Settings.
- **The preview's panel in tabs** (FR-095), as Lightroom's tools: *Edit* (**E**), *Crop* (**R**), *Print and export*, *Info* (**I**), one at a time, so the panel stays short. In a window under 1400 px the left column steps aside and **Tab** lays it over the photo.
- **A new look begins** (FR-099): the interface is set in Be Vietnam Pro, drawn for Vietnamese, on Windows, macOS and Linux alike.

## [0.12.0] - 2026-11-06 — Settings & Develop

### Highlights

- New: one Settings window, as on an iPad: the gear beside the author's name, or ⌘,
- New: memory and cache to manage, with Default and Auto for your computer; memory given back when it runs short
- New: a collage sheet says its pixels and how sharp it prints
- New: a develop view a Lightroom user can read: its keys, a filmstrip, the whole window with Navigator and History
- New: before and after side by side, split or one above the other
- Fixes: zooming into a portrait or turned photo shows it the right way up again

### Added

- **Settings window:** every setting of this computer in one place, opened by the gear at the top of the sidebar, **⌘,** (Mac) or **Ctrl+,** (Windows, Linux), or the version number (*What's new*): *General* (language, theme), *RAM & cache*, *Updates*, *Report a problem* and *What's new*, where **Show all** opens a version's whole entry, one at a time. The version's popover is gone.
- **RAM & cache:** the share of the computer's memory for previews (15, 25 or 40 %), the disk cache's limit (2, 5, 10 or 20 GB) and what it holds, *Prepare previews in the background*, *Clear cache*; **Default**, and **Auto**, which chooses from the computer's memory and free disk; *For this computer* shows the processor, the graphics card, the memory and the free disk it goes by. Each control says what it does to speed. Sizes of a gigabyte and more read in GB.
- **The photo preview's tools** (FR-095): one row above the photo, so nothing covers it: crop, rotate, basket, *Before*, undo and redo, export, then zoom, where you are in the set, the filmstrip and **ⓘ** with every key. Lightroom's keys work: **R** crops, **\\** shows the photo before its colour and back, **Z** or a click goes between fit and 1:1, **⌘[ ⌘]** rotate, **⇧⌘E** exports. Zoom now counts the photo's own pixels: *Fit*, then 100 % is 1:1. The colour cards come first, as in Lightroom.
- **Fill the app window** (FR-095): the photo preview can fill the whole window instead of opening as a dialog (Settings → General → *Photo preview*, or **F**), with Lightroom's left column: the **Navigator**, the photo small with the zoomed part outlined (click or drag it; Fit, Fill, 1:1, 2:1), and the photo's **History**, each change named (*Warm / cool +8*, *Crop*): click one to go back to it, ⌘Z to undo that. **Tab** hides the side panels.
- **Before and after, as in Lightroom** (FR-096): beside *Before*, a menu of views: side by side (**Y**), one photo split by a line you drag (**⇧Y**), top and bottom (**⌥Y**) and split top and bottom. Both sides zoom together and stay sharp at 1:1.
- **Filmstrip:** in Pro mode, the photo's preview shows the view's photos in a strip under the photo, with their rating, basket and RAW marks: one click opens any of them. The button beside the photo count hides it, for low screens; the capture screen's shots use the same strip.
- **Collage:** the *Sheet size* card shows the sheet's pixels and the DPI and quality it prints at, as sharp as its less sharp photo.

### Changed

- **Memory:** when the computer runs short of memory, the previews give most of theirs back by themselves, and take it again only once memory is free.

### Fixed

- **Zoom:** since v0.11.0, zooming into a photo taken upright, or turned in the app, showed its sharp part sideways and stretched once the photo had opened from its prepared preview. It now shows the photo as it is.

## [0.11.1] - 2026-11-06

### Highlights

- Fixes: instant previews keep the photo on screen, and never mix up photos after another project is opened
- Fixes: fast browsing through RAW photos no longer queues up work for photos already left behind

### Fixed

- **Previews:** a project opened while a photo was still loading could show the previous project's photo in place of one of its own.
- **Previews:** loading the next photos ahead could push the photo on screen out of memory, so zooming into it or moving a slider decoded it again.
- **Speed:** browsing quickly through RAW photos left a develop running for each photo passed; now only the photo on screen goes first, and photos browsed past are dropped before they develop. Work that does not say otherwise (an analysis, the collage grid) no longer jumps ahead of the photo on screen.
- **Speed:** a photo being prepared in the background and opened at the same moment is no longer made twice; two writes of one preview no longer clash.
- **Cache:** the 5 GB limit now holds while previews are being prepared, not only after, and previews go before thumbnails, so the gallery keeps its thumbnails.
- **Cache:** previews made by v0.11.0 are made again once, under a new name that a later change to the RAW develop can retire.
- **Sliders:** the wider press area of v0.11.0 no longer moves the label above a slider by 2 px.

## [0.11.0] - 2026-11-05

### Highlights

- New: any photo opens at once, RAW too: previews are prepared in the background and kept, as in Lightroom
- UI update: colour cards in the previews only; sliders take a near press and the arrow keys
- Fixes: a RAW preview that could hang for good

### Added

- **Instant previews:** after a folder's thumbnails, a preview of every photo is prepared in the background (*Preparing previews*) and kept in the cache, so any photo opens in about 30 ms instead of 0.3 s for a JPEG and 0.4 s for a RAW; → and ← load the next photos while you look. The preview uses up to a quarter of the computer's memory, and the cache's limit is 5 GB (was 2 GB).

### Changed

- **Speed:** the photo on screen is developed before anything working in the background (previews being prepared, the export's size estimate, an export).

- **Colour:** the colour cards are in the previews only, where the photo is seen large; the gallery's panel keeps the print size, the date stamp, the frame and export. *Apply this look to all photos* still sets everyone's look.
- **Sliders:** a press just above or below a slider still takes it, and a click on its name lets the arrow keys move it (by 1, by 10 with Shift), as in a RAW editor.

### Fixed

- **Previews:** preparing previews in the background and *Find best shots* at the same time no longer hang each other.
- **Preview:** a photo opened while the one before was still developing no longer waits for it, and the spinner can no longer turn for good.

- **Camera RAW:** opening a RAW no longer hangs for good after moving on from one that was still loading; the export's size estimate and the preview waited for each other.
- **Stability:** two rare hangs found by a sweep and removed: finding borrowed places while a best-shot analysis starts, and opening a project while a setting is changed.

## [0.10.0] - 2026-11-05

### Highlights

- New: RAW photos edited on the camera's own data: highlights pulled down bring back detail near white, shadows lift without banding, warmth changes the white balance
- New: RAW photos open neutral; a *RAW colour* card makes them look like the camera's JPEG in one click
- Faster, cleaner RAW: Fotollection's own develop for Canon, Nikon, Sony and more, 0.2–0.5 s a photo
- Colour sliders drawn on the graphics card while dragging; exports about a fifth faster

### Fixed

- **Colour cards:** a slider's knob is no longer cut off at the bottom while the pointer is over it, at the end of the *More* sliders.
- **Toolbar:** the icons of Auto turn, Revert, Select and Find best shots sit in the centre of their buttons when the window is narrow and their labels hide; they were 2 px to the left.
- **Collage:** Auto turn and Revert answer in about half a second on a trip's 500 photos, where they took 3 to 5 seconds, and their button shows a spinner while they work.
- **Camera RAW:** a RAW file cut short (an interrupted copy, a damaged card) is refused with an error instead of being shown and exported with made-up stripes where its data is missing.
- **Collage:** the colour cards change the photo picked on a sheet, in the panel and in the sheet's preview, and are hidden while no photo is picked (a click beside the sheets, or Esc). They used to change the shared look, which a photo with a look of its own ignored, so only one photo of a sheet seemed to follow.

### Changed

- **Camera RAW editing:** on a RAW photo, brightness, highlights, shadows, warmth and tint now work on the camera's own data, before the picture is finished, as in a RAW editor: pulling the highlights down brings back detail near white, lifting the shadows bands less, and warmth changes the white balance itself.
- **Camera RAW colour:** a RAW photo now opens neutral, with the sensor's own colours and the camera's exposure, and a new *RAW colour* card turns it **like the camera's JPEG** in one click, for one photo or with *Apply to all RAW photos*. The camera's look now comes closer to the camera's own JPEG on several cameras. Projects saved before this version keep the camera's look, so prints you made stay the same.
- **Camera RAW:** photos from Canon, Nikon, Sony and other cameras with a standard colour filter are now developed by Fotollection's own engine, as DNG already was: a full develop takes 0.2–0.5 s, faster than before on nearly every sample camera. They come out cleaner too: stuck pixels and the fine maze pattern of many sensors are removed before the colours are rebuilt, fine detail shows far less false colour, and edges between strong colours no longer fringe. Fujifilm X-Trans follows; on a Mac, Panasonic and Olympus photos are still developed by macOS, which corrects their lenses' distortion.
- **Preview:** dragging a colour slider is drawn on the graphics card where the computer has a capable one, about ten times quicker a step, so the photo follows the slider; the date stamp, the frame and the border are never tinted. The app decides by itself, and Rust's render takes over when the drag ends, so what settles is what prints.
- **Export:** large photos export about a fifth faster: more of them are worked on at once where memory allows (26 MP JPEGs, 8.9 → 10.8 photos a second).

## [0.9.9] - 2026-11-05

### Highlights

- New: colour for every photo or one photo: Faves (brightness, contrast, warm / cool, saturation…) and Tools (fade, toned shadows and highlights, black and white through a filter)
- New: RAW photos badged in the gallery, and a RAW only / No RAW filter

### Added

- **Preview and gallery:** a **RAW** badge on RAW photos, in the theme colour on the grid's tiles so they show while scrolling, and on *This photo*, which shows the format of other files (JPEG, HEIC, PNG).
- **Gallery:** the filters are one **Filters** button, with a new choice, **RAW only** or **No RAW**; the button says how many filters are on.
- **Colour** (FR-079): two cards between the print size and the date stamp, in Pro, Collage and the previews. **Faves** shows brightness, contrast, warm / cool and saturation, **More** adds highlights, shadows, tint and vibrance; **Tools**, folded at first, holds fade, a tint for the shadows and one for the highlights, and black and white through a yellow, orange, red or green filter. In a photo's preview they change that photo only, with *Apply this look to all photos*; hold **Before** or `\` to compare. The date stamp, the frame and the border are never tinted, and a photo with nothing set prints exactly as before.

## [0.9.8] - 2026-11-04

### Highlights

- New: in the preview, scroll up or down with two fingers to zoom in and out
- UI update: every panel spaced and sized the same way in every mode; the tools panel lines up with the folder list
- UI update: This photo holds the camera and image facts, and folds from its title
- Fixes: tooltips stay inside the window on Windows and at the bottom of the preview

### Changed

- **Preview:** scrolling up or down with two fingers (or the mouse wheel) now zooms the photo about the pointer, fingers up to zoom in, in the photo and the sheet previews; sideways, it moves the zoomed photo.
- **Panels:** one spacing and one control size in every card of Pro, Simple, Collage and both previews: every select and choice is the same height, every label the same style, and an option switched off leaves no gap. The tools panel now has the folder list's thin overlay scroll bar, so both sides are the same width.
- **Preview:** cards are spaced as in the panel, 12 px apart, and *Export* is a card like the others.
- **Preview:** **This photo** now also says what the file is, and takes the place of the Camera and Image cards, so the panel is shorter; click its title to fold it away.

### Fixed

- **Preview:** a tooltip of two or three lines, such as the zoom buttons', is no longer cut off at the bottom of the window: it opens above its button when there is no room below.
- **Windows:** the tooltips of the preview's buttons (crop, turn, zoom) and of best shot no longer run under the title bar: every tooltip now opens below its button (above it when there is no room), in the same style.
- **Preview:** the *Print / export size* card no longer packs its controls tight; it is spaced like the same card in the panel and like *Date stamp* beside it.

## [0.9.7] - 2026-11-04

### Highlights

- New: a handwriting face for postcards, Patrick Hand, as easy to read on a print as the typewriter
- New: type an address once, then use it for every postcard of the set in one click; recent addresses to pick from
- UI update: the preview opens with the photo at a glance: camera, lens, exposure, place, date and size
- Faster: large photos export faster with about half the memory (20 MP JPEGs: 10 a second, under 1 GB)

### Changed

- **Export of large photos:** prints of 20 MP camera JPEGs export at about 10 photos a second instead of 5–7.6, and an export of a whole trip stays under 1.4 GB instead of passing 2 GB. Print files are under 1 % larger, the picture the same. Very large photos (50 MP and up) export fewer at a time, so memory stays within bounds.

### Added

- **Postcards, one address for a set:** under the address typed on a card, **Use for N postcards without an address** puts it on every other card shown that has none, in one undoable step (or on all of them, replacing theirs, or only on the ticked ones). A card without an address offers the **Recent addresses** typed on this computer, to pick with one click or forget with ×; they never go into a project file.
- **The photo at a glance:** the preview's first card, **This photo**, shows the camera and lens (with the names you gave them for the info frame), the exposure as the frame prints it, where and when the photo was taken, and its size in pixels, megapixels and megabytes.
- **Postcards:** a handwriting face, **Handwriting (Patrick Hand)**, in the *Font* list next to the typewriter. It has every letter of Vietnamese and of Europe's Latin-script languages, and its strokes are at least as thick as the typewriter's at the medium size, the thinnest a test print showed readable, even at the small size. A size looks the same in both faces.

## [0.9.6] - 2026-11-04

### Highlights

- New: export into one folder, or a folder per day as before: a switch above Choose destination
- New: share a photo whole in a square or any ratio, the rest white, black or a colour of your own, nothing cut off

### Added

- **Whole photo in a frame** for *Share online*: under the share size, a switch fits every photo whole into a frame of the ratio you pick (Square 1:1, Portrait 4:5, 3:4 or 2:3, Landscape 3:2 or 16:9, Story 9:16, or your own), centred, with the rest filled in white, black or a colour you choose. A portrait goes on a square feed without losing its top and bottom, and without a trip to Photoshop. On Instagram the frame is 1080 px wide (1080 × 1080 for a square). Prints keep their print size.

### Changed

- **Export:** a **One folder / By day** switch above *Choose destination…*. **One folder**, now the default, puts every file straight into the chosen folder, so photos exported together are found together; **By day** keeps a folder for each day (`2026-07-14`), as every export did before. The app remembers the choice on this computer. It works in every mode, for prints, shared files (in `Share`), collage sheets and postcards.

## [0.9.5] - 2026-11-04

### Highlights

- UI update: a smaller stamp box on postcards, hidden by the stamp you stick on
- Fixes: bold text on postcards prints as one solid stroke, no longer a doubled line
- UI update: a long address gets a fourth line, and a red ! marks a postcard whose text is cut

### Fixed

- **Postcards:** bold text no longer shows doubled strokes (clearest on M and W in the typewriter face); each stroke is thickened into one.

### Changed

- **Postcards:** a long address gets a fourth line, in the room of its three (closer and a little smaller), and the address takes at most 4 lines. When a message or an address does not all fit, a red **!** marks the card in the grid and in its preview, and the part that is cut.
- **Postcards:** the dashed stamp box is smaller, 20 × 25 mm, so the stamp you stick on hides it. The manual now says what glossy paper means at the post (the postmark may smear off the stamp, a gummed stamp may not hold) and suggests matt paper or a self-adhesive stamp.

## [0.9.4] - 2026-11-04

### Highlights

- New: postcards in Collage mode: one photo and a postcard back (stamp box, writing lines) on each print, with your text typed in a typewriter face or room to write by hand

### Added

- **Postcards** (Collage mode): the **Postcard** switch above the sheets turns each sheet into one photo and the back of a postcard, held landscape (photo left) or portrait (photo below), with a dashed stamp box at the top right and room beside it for more stamps or a long post-office label, and the writing lines together at the foot: 3 or 4 for the message and 3 for the address, side by side on a portrait card. Type each card's message and address in the panel or its preview, in Xanh Mono or one of your own fonts (an address in Greek or Cyrillic prints too), small, medium or large, in black, blue, sepia or red, bold, italic or underlined per part, with a small credit line under every card if you like; or leave the lines for handwriting, and the panel says which pens write on photo paper. Each card exports as `<photo>-postcard.jpg`.

## [0.9.3] - 2026-11-03

### Highlights

- Fixes: Display P3 and Adobe RGB photos show their true colours in the app, no longer washed out
- New: the preview says what each file is: chroma subsampling, colour profile, sensor, the decoder that reads it; unusual files are marked in the gallery and have their own filter

### Fixed

- **Display P3 and Adobe RGB photos** (iPhone photos, cameras set to Adobe RGB) no longer look washed out in the app: thumbnails, previews, zoom and the crop view now convert them to the screen's sRGB, as share exports always did. Thumbnails made before are made again.
- **iPhone HEIC photos on Windows** keep their Display P3 profile when Windows' HEIF codec decodes them, so they are converted like on a Mac instead of showing pale.
- **Straightened photos** shrunk more than 2× (a preview of a large crop, a share export) no longer turn fine detail such as fences or fabric into bands.
- **PNG:** 16-bit images no longer show bands in smooth gradients (they are dithered to 8 bits), a PNG that declares its gamma or primaries (`gAMA`, `cHRM`) shows in its true colours, and semi-transparent pixels are mixed onto white as light mixes, no longer too dark.

### Changed

- **Share exports** are downsized in linear light, so fine detail keeps its brightness (a thin black-and-white pattern stays the grey it looks like, no longer darker). About a quarter slower: 18 instead of 23 photos a second for 16 MP photos on an Apple M3.

### Added

- **Image facts** in the preview's Image card: a JPEG's chroma subsampling (4:2:0…), coding, bit depth and colour profile (sRGB, Display P3, Adobe RGB…), a RAW's sensor (Bayer RGGB, X-Trans) and bit depth, an HEIC's colour space and depth, the kind of DNG, and the decoder that reads the file. Read with the EXIF at import, which now opens each file once instead of twice.
- **Unusual files**, a new choice in the flag filter, and an ⓘ in the gallery: CMYK or 16-bit JPEGs, Adobe RGB photos, X-Trans RAWs and DNG kinds the app does not develop itself yet.

## [0.9.2] - 2026-11-03

### Highlights

- Fixes: zooming into a camera RAW no longer freezes every preview after it; the first sharp zoom of a RAW comes at once
- UI update: the language is chosen in the version popover, the same in every mode, so the Print panel has one card less

### Fixed

- **Zooming into a camera RAW** (ARW, NEF, CR3, DNG…), mostly on Windows: after a zoom the next photo's preview could spin for good, and every preview after it, until the app was restarted. Two RAW develops asked at once waited for each other. They now always finish.
- **The first sharp zoom of a RAW** no longer develops the file a second time: the full image the preview was made from is kept for zoom (0.7 s → 0.04 s for a 24 MP ARW on a Mac).
- **A preview that cannot be made** shows why, with its error code (for example ERR-IMG-0006 for a file cut short), instead of a spinner that never stops.

### Changed

- **Language** moved from the top of the Print settings panel to the popover behind the version number at the foot of every panel, next to *Check for updates* and the cache. Simple and Collage modes can now change it too.

## [0.9.1] - 2026-11-02

### Highlights

- Fixes: JPEGs cut short are refused instead of showing half a photo; 16-bit grey and CMYK JPEGs show right
- Fixes: DNG files whose sensor area starts at an odd row keep their colours
- New: every error message ends with a code such as ERR-IMG-0006, to quote in a report
- New: Copy diagnostics, next to Report a problem, copies your version and the last error codes; the small error log behind it stays on your computer, without photos, places or file names

### Fixed

- **JPEG files cut short** (a card pulled out, a copy that stopped) are refused with a message instead of showing a grey half photo.
- **16-bit grey JPEGs** (lossless) show the picture instead of noise.
- **CMYK JPEGs** read the same on both decoders; the fallback one no longer turns them negative.
- A grey or CMYK colour profile is no longer attached to an exported RGB photo, where it made labs read the colours wrong.
- Thumbnails are made once more after the update, so the wrong ones of such files go.
- **DNG files whose sensor area starts at an odd row or column** (some cameras and converters) keep their colours: red was read as blue, and each pixel took its neighbour's black level. A black level given per row or per column is now read too.
- The notices over the photos (an error, the camera's temporary folder, the progress of thumbnails) are evenly spaced blocks; the progress bar no longer sticks to the notice above it. An error can be dismissed with its × button.
- Files skipped at import show their error code in the list too.
- Scrolling to the end of the versions list, a search list or the compare window no longer scrolls the app behind it.
- **On Windows, a RAW from a camera not read yet** says so, and that Microsoft's free Raw Image Extension may open it, instead of a general "could not be decoded".

### Added

- **Error codes:** every error message ends with its code, such as `ERR-IMG-0006`, to quote when reporting a problem; the dictionary of codes is in `docs/ERROR_CONTROL.md`.
- **Copy diagnostics,** next to *Report a problem* (click the version at the foot of the panel): copies your version, your system and the last error codes, to paste into the report. The app keeps a small error log on your computer for this (5 files of 1 MB at most), with no photos, places or folder names in it, and sends nothing.

## [0.9.0] - 2026-11-02 — Three modes

### Highlights

- New: three modes, Simple, Collage and Pro, switched at the top of the sidebar or with Cmd/Ctrl+1, 2, 3
- New: Collage, two photos on one print like a half-frame camera, with a suggested size, a cut line and Auto turn for prints you cut in two
- New: Simple, one photo at a time, ↑ keep, ↓ drop, then export
- New: a 3 mm border on every print, white or black, and a small, medium or large date stamp
- Website: a Manual in English and Vietnamese, and a Sponsor on GitHub button

### Added

- **Three modes**, switched at the top of the sidebar or with Cmd/Ctrl+1, 2 and 3, and remembered on this computer: **Simple**, **Collage** and **Pro**, the app as you know it. Your photos, basket and settings are the same in every mode.
- **Collage mode**, inspired by half-frame film cameras: two photos on one print, so a 10 × 15 holds two 10 × 7.5 photos and a set costs half as much. Photos pair in order and each sheet shows as it will print; drag a photo onto another to swap them, turn a sheet, crop each photo for its place, and open a sheet like a photo for a large preview with zoom, crop and Export this sheet. Fill each half for no white at all, the two photos edge to edge and each cropped by you; or choose the photo ratio, take the suggested size (6 × 9 cm for a 2:3 photo, with 5 mm of white round it) or type your own with the ratio locked, and switch the cut line on to cut the print in two, or off for a half-frame strip. In a sheet's preview, **Rotate photo** (⇧R) turns the chosen photo a quarter in its half, so a landscape photo next to a portrait one fills its half; its own print does not change. The line under the date can differ per photo: turn *Same on both photos* off and type one for each. A change of setting shows on the sheets at once. Select mode and the print basket work as in the gallery, and ↑ / ↓ in a sheet's preview. Export writes one file per sheet.
- **Collage, choosing and turning:** **Select** ticks whole sheets, both photos at once, with one tick and one frame round each sheet; the bar counts sheets, and Export writes only the ticked sheets (*Export 2 selected sheets*). *Rotate photo* turns a photo in its half and keeps the sheet. **Auto turn** lays every sheet landscape with its two photos standing, for prints you cut in two, and **Revert** brings sheets and photos back to their own orientation; the keys of the toolbar, here and in the gallery, sit behind an **ⓘ** button.
- **Simple mode:** one photo at a time, as it will print. **↑ Keep** puts it in the print basket, **↓ Drop** sets it aside, and the next photo comes at once; two large buttons do the same with the mouse. The panel keeps only the print size, the 3 mm border, the date stamp (the same card as Pro: format, corner, colour, size, place and your own line) and Export, which sends the basket.
- **3 mm border on every print:** the *3 mm border* switch moves to the print-size card and works with or without the info frame. Without a frame, a border 3 mm wide runs round the photo, white or black (on a collage sheet, the paper round the photos too); with Fit whole photo, the whole photo now fits inside the size you chose instead of making the print larger. The print always keeps the exact kiosk size. A border you had in Crop to fill stays on.
- **Date stamp size:** Small, Medium or Large, in every mode and for one photo in the preview, sized to the print: larger on a larger print, and half on each photo of a collage sheet. Stamps you had stay Medium.
- **Website:** a Manual page, in English and Vietnamese, with screenshots of the real app, in 20 chapters that follow the way you work: getting started, stars and filters, best shot, the print basket, print size, crop and straighten, zoom, the date stamp, the info frame, dates and places, camera clocks, export, projects and undo, RAW, Open with, shooting with a camera, updates, Simple mode, Collage mode, and every shortcut. The chapter list opens from a round button bottom left.
- **Website:** a "Three modes" section on the landing page with Collage and Simple; every screenshot taken again with the mode switch in the sidebar.
- **Website:** the print-basket and best-shot screenshots show a cat photographed by the author on a Lumix GX9, with a softer, a shaken and a darker copy beside it for best shot.

### Fixed

- **3 mm border:** photos exported for social media no longer get it; Pro's photo preview shows the switch again; the size badges and the DPI follow the photo inside the border; dragging the photo in the preview keeps pace with the pointer.
- **Collage:** cropping a photo on a sheet no longer changes its print crop (sheets have a crop of their own); reordering in one folder keeps the others' order; a sheet's quality counts the photo's crop and straightening.
- **Modes:** Pro's star and flag filters no longer hide photos in Simple and Collage; select mode, Quick-preview and an open preview no longer carry over to another mode; Cmd/Ctrl+1/2/3 work from a preview (it closes) and wait only while a crop is being edited; "Open with" switches to Pro, where the preview is.
- **No more jumps while you type or switch:** typing the line under the date no longer makes the panel jump or the text slip back; the notes and figures in the panels and previews keep their place while they refresh; ← → in the preview no longer reflow the inspector; a zoomed photo stays sharp and Quick-preview stays sharp when a setting changes; a crop in progress is no longer reset by another change; two quick edits to a camera profile both stick.
- **Previews:** a photo or sheet with a black border keeps a visible edge on the black stage; a click around a sheet lets go of the chosen photo.
- **Simple:** choose prints or social media, as in Pro; "Every photo is sorted" shows once each photo is kept or dropped, with a way to look through them again; a photo with its own date stamp can be reset from the panel.

### Changed

- **Website:** every page has its own link preview (title, description and image) when shared: Download, Performance, Manual and Code signing, in English and Vietnamese.
- **Website:** a red *Report a problem* button in the footer of every page, to the GitHub issue forms, as the app's own *Report* button (v0.8.3).
- **Website:** a Download page (fotollection.org/download) with the three download buttons under its title, every installer, the first-launch steps, the Linux commands with a Copy button, the table of features by system, what is new in the released versions (no longer the unreleased one) and which Linux distributions run it; the home page keeps only its download buttons, and the Download button at the top leads there.
- **Website:** "Quality" moves from the top bar to the footer, to make room for "Manual".
- **Website:** a "Sponsor on GitHub" button in "Buy me a coffee", so you can support from any country by card, monthly or once.

## [0.8.3] - 2026-11-01 — Export what you ticked

### Highlights

- Fixed: with photos ticked, Export now writes only those photos and says how many
- New: a red Report button under the version, to tell us about a problem or an idea

### Fixed

- **Export with ticked photos:** while photos are ticked in select mode, the export button says *Export 3 selected photos* and writes only those, with the size estimate for them, instead of every photo shown. Untick them all and it exports the whole view again (user feedback).

### Added

- **Report a problem:** a red *Report a problem or an idea* button in the versions popover (click the version at the foot of the settings panel) opens the project's GitHub page for problems and ideas in your browser. The app itself sends nothing.

## [0.8.2] - 2026-11-01 — Linux

### Highlights

- New: Fotollection for Linux, as an AppImage and a `.deb`, for x86_64 and ARM64
- Linux has every photo feature except camera capture, and HEIC comes later

### Added

- **Linux (64-bit, glibc 2.35 or later):** an AppImage that runs on Ubuntu 22.04+, Debian 12+, Linux Mint 21+, Fedora 36+, Arch and others, and a `.deb` for Ubuntu, Debian and Mint that adds Fotollection to *Open With* for JPEG, PNG and camera RAW. Culling, best shot, print sizes, crop, the date stamp, the info frame, places, RAW and export all work as on macOS and Windows. Not on Linux yet: HEIC photos (they are skipped with a message saying so) and shooting with a camera over USB (no button). *Check for updates* in the AppImage downloads the new AppImage. On computers with Nvidia graphics, the app avoids WebKit's blank-window problem by itself.

## [0.8.1] - 2026-11-01 — Green photos fixed

### Highlights

- Fixed: some JPEG photos showed, and exported, as green stripes
- Thumbnails are made once more after the update, so the green ones go

### Fixed

- **Green, striped photos:** JPEGs saved in a rare layout (one scan for each colour, written by some editors and online tools) showed as green tiles with fine stripes in the gallery and the preview, and exported that way. They now open with their real colours; the usual JPEGs from cameras and phones open as fast as before. Every thumbnail is made once more after the update, so no green one stays.

## [0.8.0] - 2026-10-27 — Places and Lightroom

### Highlights

- New: dates and places by hand, renamed places, and places borrowed from your phone's photos of the same scene
- New: stars and rejects come in from Lightroom, Bridge, Capture One and darktable (XMP sidecars)
- New: camera clocks, a 3 mm border in Crop to fill, and Check for updates
- New: RAW built into the app, with our own reader for DNG; shoot with a Sony a6400 plugged in (Windows)
- Not yet tried on Windows in this build: RAW without the Raw Image Extension, and the latest camera fixes

### Added

- **Shoot with a camera plugged in (FR-062, FR-063), Windows, Sony a6400 first:**
  1. Set the camera's *USB Connection* to *PC Remote* and plug it in.
  2. Click the camera button next to *Open folder*.
  3. The capture screen shows the camera's live view with the print size's shape over it, and its ISO, shutter speed, aperture and file format. *Capture* (or Space) takes a photo, and the shots line up under the live view.

  The photos go to a temporary tab. Close the screen, look them over, then *Save all* to the folder you choose, with your crops and stamps kept, or *Delete photos*. Quitting the app with shots not saved asks the same question, and a session left by a crash comes back as a tab. No driver to install. RAW shots open through the built-in RAW reader. Basic capture and live view were tried on Windows; the quit question, Save all and unplugging during a session are not yet tried there in this build.
- **Camera RAW built in on Windows (FR-016):** RAW photos open without Microsoft's Raw Image Extension, through a RAW reader inside Fotollection, with colours matched to the camera's own JPEG. On macOS the system's reader stays first, and the built-in one opens what it cannot. Not yet tried on a Windows PC without the extension in this build: if a RAW photo does not open, install Microsoft's free Raw Image Extension, which still works as before.
- **Our own RAW reader, DNG first (FR-016):** DNG photos straight from a camera's sensor (tried on a Ricoh GR III file) now open through a reader written for Fotollection, the same on macOS and Windows; other DNG, such as Apple ProRAW, still open through the reader they used before. A DNG develops in about a third of a second with less memory, and its colours, tone and corner brightness follow the camera's own JPEG more closely than before. Other RAW formats follow one by one.

- **Camera clocks (FR-030):** a camera set to the wrong time no longer puts its photos in the wrong order. Open one of its photos and pick your phone's photo of the same moment, or type the correction in **Camera clocks** in the settings. Every photo from that camera moves, the date stamp and day folders follow, and the photo files are never changed.
- **Dates and places by hand (FR-032):** give a scanned print, or a photo without a date or GPS, its date and place. In the preview, *Set the date…* and *Set the place…*; for many photos, tick them and click *Date and place…*. Type a town or city to find it, offline (`ha noi` finds Hà Nội). The grid order, the date stamp, the info frame, day folders and the exported copy follow, and Undo takes it back.
- **Stars from Lightroom (FR-058):** opening a folder culled in Lightroom, Bridge, Capture One or darktable brings in the stars and rejects from their XMP sidecar files. Your Fotollection project still wins, and the sidecars are never changed.
- **Places from your phone (FR-031):** a camera without GPS takes the place of a phone photo of the same scene, taken a few minutes apart. Sure matches are filled in by themselves after a best-shot analysis; *Find places* in the *Places* card shows the less sure ones next to the phone photo to accept or skip. Borrowed places are marked on the thumbnail and in the preview, and Undo takes them back.
- **Rename a place (FR-005):** the settings panel's *Places* card lists the places on your photos; type the name you want printed ("Prague" instead of "Praha 1") and every print with that place follows. The place can now be shown on the info frame too.
- **Add a border in Crop to fill (FR-065):** a switch in the info frame leaves a 3 mm white (or black) border on the top and sides, inside the print. The print stays the exact size of the shop's kiosk, such as CEWE at dm and Rossmann, and still looks like an instant photo.
- **Check for updates (FR-061):** click the version number at the foot of the settings panel, then *Check for updates*. If a newer version exists, *Download the installer* opens it in your browser; installing it replaces the app and keeps your settings and projects. This is the only time the app goes online, and only when you click.

### Website

- **Correction:** the RAW feature card and the Performance page now say that on Windows RAW still needs Microsoft's free Raw Image Extension. They read as if RAW worked everywhere with nothing to install, which was true only on macOS. A RAW reader built into the app is in progress.

- A **speed** card among the features (1,000 photos in 0.12 s, 17 ms preview updates, a 0.4 s start from the second launch), and plainer wording for "Open with".
- The Performance page has a **Camera RAW** section: thumbnails in 9–32 ms and the full picture in 0.4–2.2 s for nine cameras, against Adobe DNG Converter's 1.2–1.8 s per file only to convert (a third-party estimate, cited).
- Every chart carries a footnote: figures are for reference and depend on your computer.
- The footnotes name the maker behind each RAW format (RW2, CR2, CR3, NEF, ARW, RAF, ORF) and say DNG is Adobe's open specification.
- The footer of every page shows the **total visits** in a badge like the GitHub star button, read from Google Analytics when the site is built and updated once a day; the page loads nothing extra for it.
- A shorter footer: the demo-photo credit lives only in the footnotes, and the analytics note no longer mentions the app.
- A **Code signing policy** page with the privacy policy (fotollection.org/code-signing), linked from the footer: how installers are built and published, and what the app sends over the network (only Check for updates, on click).
- The FAQ's "why Fotollection" answer says what is missing elsewhere: no app is designed for the photo printer, so you crop by hand or leave it to the shop's machine.

### Changed

- **The disk cache stays under 2 GB.** Thumbnails and best-shot analyses no longer pile up: past the limit the ones you have not looked at for longest are removed in the background, and unfinished files from a crash are cleaned. The version popover shows the cache's size and has **Clear cache**.
- The Windows MSI keeps the same upgrade code in every release, so a newer MSI always replaces the older one.

## [0.7.0] - 2026-10-25

### Highlights

- New: camera RAW opens as it is: RW2, CR2, CR3, NEF, ARW, RAF, ORF and DNG
- New: open photos from Finder or Explorer with Fotollection; its folder becomes a tab
- New: ticks follow you across folder tabs in select mode
- Fixes: the info frame no longer repeats the maker; projects open when double-clicked with the app closed
- Tested on Windows with the Raw Image Extension; never set as your default viewer

### Changed

- Everything in 0.7.0-beta.1 below, checked on Windows: "Open with", one window for a second photo, RAW with the Raw Image Extension, and nothing left in "Open with" after uninstalling.

### Website

- Camera RAW and "Open with" on the home page; the questions mention opening photos from Finder or Explorer; the Performance page gives the RAW speed and credits the CC0 samples from raw.pixls.us.

## [0.7.0-beta.1] - 2026-10-25

### Highlights

- New: camera RAW opens as it is: RW2, CR2, CR3, NEF, ARW, RAF, ORF and DNG
- New: open photos from Finder or Explorer with Fotollection; its folder becomes a tab
- New: ticks follow you across folder tabs in select mode
- Fixes: the info frame no longer repeats the maker; projects open when double-clicked with the app closed
- Beta: on Windows, install Microsoft's free Raw Image Extension, then try "Open with" on a photo

### Added

- **Open with Fotollection (FR-054):** right-click a photo in Finder or Explorer and choose Fotollection: it opens in the preview, its folder becomes a tab, and ← → go through the folder. Fotollection never makes itself your default viewer; opening another photo while it runs brings it forward with that photo.
- **Camera RAW (FR-016):** RW2, CR2, CR3, NEF, ARW, RAF, ORF and DNG open like any photo, with their date, camera and exposure for the stamp and the info frame. Thumbnails and best-shot suggestions use the preview inside each RAW, so a folder of RAW files browses as fast as JPEG; the print preview and the export develop the full photo. On Windows, install Microsoft's free *Raw Image Extension*.
- **Ticks follow the folder tabs (FR-056):** in select mode, photos ticked in one folder stay ticked when you switch to another folder; the selection bar counts them all and says how many are in other folders, and one click sends them all to the print basket.

### Changed

- The red close button of the preview deepens to burgundy under the pointer, so you can see it is the one you are about to click.
- Canon CR3 files are no longer read as HEIC (they had no date or camera details).
- **Vietnamese:** the app and the website speak plain Vietnamese: *tệp* for file, *ứng dụng* for app, *mã nguồn* for code, *hồ sơ màu* for colour profile, *không cần mạng* for offline, *phông chữ* for font.

### Website

- A **questions** section near the end of the home page: why Fotollection instead of Lightroom, the built-in Photos apps or IrfanView, how light and fast it is, and which systems it runs on, with the measured numbers.
- A new section, **Work like browsing the web**, shows folders as tabs with photos ticked in two folders.
- The footnotes credit the demo photos (CC0, Wikimedia Commons) by author, with a link to every file.
- **No ads** is now one of the privacy promises on the home page. The print-basket section no longer compares choosing photos to a dating app. The coffee note in the footnotes reads as a choice that helps the app grow.

### Fixed

- Double-clicking a Fotollection project while the app was closed could quit without opening it on macOS; the project now opens once the app is ready.
- The info frame no longer repeats the maker: "Shot on NIKON D750", not "NIKON CORPORATION NIKON D750" (also Ricoh, Olympus and other makers that write the company name in EXIF).

## [0.6.0] - 2026-10-25

### Highlights

- New: zoom buttons on the preview, and full detail when you zoom in
- New: a start-up screen with the logo and a loading bar
- Measured: after the first launch the app opens in about 0.4 s; a 32 MP photo zooms in about 0.1 s
- Website: footnotes for every trademark and the sources behind each measurement

### Added

- **Zoom buttons (FR-057):** − · zoom level · + · fit, at the bottom right of the preview; each tooltip shows its shortcut and the trackpad pinch.
- **Full detail on zoom (FR-057):** the preview stays light (1600 px) on any computer. When you zoom past it, the part on screen is replaced a moment later by the full-resolution pixels, exactly as the export will have them, with a small spinner while they load. Only the visible part is loaded, so a 32 MP photo stays light.
- **Start-up screen (FR-060):** the app opens on its logo and a loading bar, which follows the import while your last session reopens, then fades onto the finished workspace.

### Changed

- The zoom level moved from the photo counter into the zoom buttons.

### Fixed

- **Website:** the print-basket section was missing from fotollection.org because the site's build had been failing since it was added; it is back.

### Website

- Footnotes like Apple's on every page: what *free*, *open source* and *no account* mean, who owns each brand and camera logo shown, and the sources of ΔE2000, SSIM, JPEG, sRGB, Display P3, Adobe RGB, Rust and the web view. The navbar is lighter (print shops moved to the footer) and the footer says the app is made by a user, for users.

## [0.5.0] - 2026-10-25

### Highlights

- New: a print basket and a "set aside" area: ↑ and ↓ in the preview, or Select in the gallery
- New: Quick-preview: hold Space over a photo in select mode
- New: the export card estimates the size before you export
- Prints keep the camera details (never GPS); the website moves to fotollection.org

### Added

- **Print basket and set aside (FR-050):** two holding areas in the sidebar, under *All photos*. In the preview, **↑** puts a photo in the print basket and **↓** sets it aside (out of the gallery, never deleted), both moving on to the next photo; a *Print basket* button does the same. In the gallery, **Select** ticks photos (Shift+click for a range, Cmd/Ctrl+A for all) and a bar sends them to the basket, aside, or back. Open either area to look again; export follows the view. Saved in the project (schema v6) and undoable. See the [user guide](docs/USER_GUIDE.md).
- **Website:** a print-basket section shows the ↑ / ↓ keys on a zoomed screenshot of the preview, select mode in the gallery, and the size estimate above the export button.
- **Quick-preview (FR-059):** in select mode, hold **Space** over a photo to see it as it will print; release to close. ← → show the neighbours and Enter ticks the photo shown.

- **Export (FR-053):** the export card estimates how much lighter the files will be before you export, for example "Estimated: 17.0 MB → ≈ 1.4 MB (92 % lighter)", updated as the settings change, and reports the real figure when the export finishes.

### Changed

- **Prints keep the camera details (FR-010):** print exports now carry the camera, lens, exposure and capture date, like share exports; GPS is still always removed.
- **Preview:** the inspector starts with the same cards, in the same order, as the settings panel: *Print / export size*, *Date stamp*, *Info frame*, then *Export*; the photo's details follow.
- **Website:** the site moves to its own address, <https://fotollection.org/>; the old `hoangphi01.github.io/fotollection/` links redirect to it.

## [0.4.0] - 2026-10-25

### Highlights

- New: export for social media (original size, Facebook, Instagram or your own size), in sRGB, with the camera details and never GPS
- New: folders work like tabs, and several folders export together or separately
- New: export the photo you are looking at, straight from the preview, and zoom with a trackpad pinch or Ctrl/⌘ + −
- New: drag the photo to reframe it; *Fit whole photo* frames it like an instax print
- New: the Noritsu logo for film scans, and a Performance page on the website
- Fixes: long lens names no longer cover the logo, photos without EXIF get the file's date, Vietnamese wording

### Added

- **Noritsu logo (FR-040):** the info frame shows the Noritsu logo for film scans whose EXIF make is "NORITSU KOKI" (EZ Controller), from a public-domain file on Wikimedia Commons like the other bundled logos (ADR-0020).

- **Share online (FR-053):** the print size card, now *Print / export size*, has *Print* and *Share online*, so the export card only exports. Share makes light JPEGs in sRGB at the original size (about half the file, measured on compact-camera JPEGs), 2048 px for Facebook, 1080 px wide for Instagram (portraits stop at 3:4) or a long edge of your choice (640–6000 px). Photos in Display P3 or Adobe RGB are converted so they do not look washed out, and they are lightly sharpened after resizing. The files keep the camera, lens and exposure details and name Fotollection as the editing app, like Snapseed; **the location is never written**. They go into a `Share` folder, apart from prints. The choice is remembered (settings v8).
- **Export from the preview (FR-055):** the preview has an Export card, with the same controls as the settings panel, that exports only the photo you are looking at; the Export button on the photo jumps to it. Both places share the destination and the progress.
- **Folders as tabs (FR-056):** the library list starts with *All photos*; clicking a folder shows only that folder (grid, best-shot groups and export), like switching browser tabs. The select button above the list, or Ctrl/⌘-click, ticks several folders. When an export covers two folders or more, choose *Together* (the usual date folders) or *Separately* (one subfolder per folder). Folder counts no longer change with the star and flag filter.
- **Zoom in the preview (FR-057):** pinch on the trackpad or press Ctrl/⌘ + and − to zoom about the pointer (up to 800 %), Ctrl/⌘ 0 to fit; drag or scroll with two fingers to move around, double-click for 200 %. The zoom level shows next to the photo counter; click it to go back to fit.
- **Website:** the page's scroll bar is thin and in the theme colour, like the changelog box's.
- **Website:** a new *Performance* page gathers every measurement with its chart and explains the words behind them (ΔE, SSIM, JPEG colour depth, JPG or PNG, 4:4:4, colour profiles, DPI, why Facebook makes photos worse); the home page links to it.
- **Vietnamese:** the interface and the website say *cắt ảnh*, *xuất ảnh* and *xem trước* instead of the English crop, export and preview.
- **App:** scroll bars are thin and in the theme colour, like the website's; the language picker is narrower, so its label is never cut.
- **Website:** the print-shop heading shows the dm and Rossmann logos inline, in the theme colour (public-domain files from Wikimedia Commons; trademarks of their owners).
- **Development:** the Python tools (quality audits, icon and screenshot helpers) are managed with uv: `pyproject.toml` and `uv.lock` pin their packages, and `uv run` runs them.
- **Drag to reframe (FR-006):** in the preview, drag the photo to choose what the print keeps, without opening the crop editor; the print re-renders as you drag, and one drag is one undo step.
- **Fit whole photo (FR-040):** like an instax print, the whole photo keeps the print size and gets a 5 mm border on three sides plus the info bar below, so the print is larger than the chosen size (10 × 15 → about 11 × 17.9 cm); the print-size card says how large. Before, the photo shrank inside the box. *Crop to fill* still keeps the exact size.

### Fixed

- **Photos without EXIF (FR-004):** the date stamp uses the file's date, as the preview said it would; a photo with no date at all still prints its place and caption.
- **Info frame (FR-040):** the shutter speed shows for files that store it only as an APEX value (`ShutterSpeedValue`), as some edited photos do.
- **Preview:** a new setting or a drag no longer blanks the photo while the next render loads.
- **Website:** the back-to-top button works on iPhone (Safari jumped only a little).
- **Website:** the changelog box shows its "click to scroll" hint again, and its thin scroll bar now applies in Firefox too.
- **Hover hints** near the window's edge are no longer cut off.
- **Info frame (FR-040):** a long lens name (for example `30mm F1.4 DC DN | Contemporary 016`) no longer runs into the camera logo. The lens moves to its own line above the exposure settings, and a line that is still too long is set smaller.

## [0.3.1] - 2026-10-25

### Highlights

- New: click the version to see what each release brought
- UI update: the toolbar stays on one line, photo filters as dropdowns, larger sidebar icons
- Fixes: a minimum window width so the layout never breaks

### Added

- **Version history:** clicking the version at the foot of the settings panel opens a short list of every release with two lines on what it brought, read from the bundled CHANGELOG (no network).

### Fixed

- The toolbar above the photos stays on one line: the photo filters are two dropdowns, and the window can no longer be made narrower than 1,180 px.
- The project and library buttons in the sidebar are larger (16 px icons, as STYLE_RULE asks).

## [0.3.0] - 2026-10-25

### Highlights

- New: a logo size slider for the info frame, the same for every photo
- UI update: settings sections as cards, lines in the theme colour, aligned headers
- Fixes: arrow keys with the logo slider, date buttons overflowing, the same fonts on Windows as on macOS
- Website: coffee and download buttons beside the GitHub star

### Added

- **Website:** a coffee button next to the GitHub star, a round Download button in the menu; "What's new" moved out of the menu (it stays in the footer).
- **Logo size (FR-040):** a slider under the logo colours in the info frame settings scales the camera logo, from 50 % to 200 %, for every photo in the project.

### Changed

- **Lines in the theme colour:** borders are a pale teal in light mode and a pale wood in dark mode; section cards have a slightly stronger 1.5 px outline.
- **Sections as cards:** every settings section, in the settings panel and the preview, is a card with a tinted header holding its icon, title and master switch, so sections read apart at a glance.

### Fixed

- Dates and numbers in the interface use the bundled Geist Mono, so Windows shows them exactly like macOS (the rest of the interface already used the bundled Geist).
- With the logo size slider focused, ← → changed the photo as well as the size; now the slider keeps the keys, and clicking the photo hands them back to browsing.
- The date formats no longer overflow their buttons.
- The toolbar above the photos is now exactly as tall as the library and settings headers, so the three header lines line up.

## [0.2.0] - 2026-10-24

### Highlights

- New: System / Light / Dark theme switch
- New: Windows title bar in the app's colours, like VS Code
- New: remove a folder from the library, like closing a tab
- UI update: one accent per mode (teal light, wood dark), app-drawn hints, print badges in the preview
- Fixes: the best-shot highlight was clipped

### Added

- **Theme switch (FR-052):** System, Light or Dark, at the foot of the library column; the window follows it.
- **Windows title bar (FR-052):** drawn in the app's colours like VS Code's, instead of the white system strip.
- **Remove a folder (FR-001):** a close button on each folder, like a browser tab; the files stay on disk.
- **Website:** a "buy me a coffee" section with a VietQR code (Vietnamese banking apps only), a note on who makes Fotollection, a footer button to the author's blog, and larger screenshots in alternating rows, each beside its explanation (the best-shot one now shows the compare view).
- The preview shows the same Sharp / Acceptable / Pixelated badges and size advice as the settings panel (FR-026), and dialogs close with a small red button.
- **Website, "What's new":** the changelog in a scrollable box, with the latest release.
- The date stamp in the preview now has the same fields as the settings panel, including corner and colour, and comes before the info frame.

### Changed

- **Docs:** `docs/DOCS_GUIDE.md`, one checklist of what to update for each kind of change, including the website; `CLAUDE.md` and the docs index point to it.
- **One accent per mode:** light mode uses the logo's teal and dark mode its wood for every button, switch, checkbox and selection, never both at once (the website too).
- Hover hints are drawn in the app's own style instead of the system's square tooltips.

### Fixed

- The theme switch now shows which theme is selected.
- The highlight around the suggested best shot was clipped in the compare view and the groups strip.

## [0.1.0] - 2026-10-24

### Highlights

- First public preview for macOS and Windows
- Best-shot suggestions, film-style date stamp with offline place names, camera info frame
- Crop to print sizes with rotate and straighten, original-quality export
- English and Vietnamese

### Added

- **Best-shot suggestions (FR-041):** **Find best shots** groups bursts and re-framings of one moment, then suggests the best photo of each group with up to three reasons ("34 % sharper", "Eyes open", "Less camera shake"). The **Groups** view shows each group as a strip, best first; **Compare** puts the shots side by side with a bar per measure and marks closed eyes. **Keep best** picks the suggestion and rejects the rest, for one group or all clear groups at once; for groups marked "Too close to call", choose with **Keep this one**. Nothing is deleted and everything is undoable. It runs offline on bundled face models, in about 17 ms per photo (28 ms with faces); results are cached, so a second run is instant. `measure_folder --analyze` measures it.
- **Refresh folders (FR-001):** a button next to the library's folders picks up photos added to them since, without re-importing; ratings, flags, crops and stamps are kept.
- **Rotate photos (FR-006):** turn a photo left or right with the buttons next to Crop in the preview, or with `[` and `]` in the grid and the preview. Lossless, the crop turns with the photo, undoable, and kept in the project (project schema v4). Upside-down selfies are fixed in one key press.
- **Straighten (FR-006):** a slider in the crop editor levels a tilted horizon by up to ±15°, with a fine grid while you drag; in crop mode ← → level by 1° (⌥ for 0.1°), and move between photos again once you leave it. The crop stays as large as possible without empty corners, and only the cropped pixels are resampled. Undoable, kept in the project (project schema v5).
- **Print-quality audit (NFR-009):** `bench/quality/audit.py` compares every exported pixel with the original. The export keeps the colour profile, full detail and saturation, with a mean ΔE2000 of 0.19, against a typical web-editor export that loses 87 % of the fine detail and 13 % of a P3 photo's saturation. Shown in both READMEs and in RESULT §9.
- The line under the date updates the preview while you type (a short pause after the last key), in the settings panel and in the preview.
- **A photo's own date stamp (FR-044):** in the preview, the stamp controls (on/off, format, place, the line under the date) now change the open photo only, marked *This photo*; **Apply to all** makes them the setting for every photo. The settings panel offers **Reset** for photos with their own stamp. Undoable, and saved in the project (project schema v3).
- **Film-camera style imprint (FR-004, FR-005, FR-033, FR-036):** the text inside the photo is now a slanted 14-segment LCD face (DSEG14 Italic, OFL) with the soft glow of an LED date imprint. Switch on **Place on the date line** to add the town from the photo's GPS (`'25 09 21  DEBRECEN`, offline; city, city + country code, or city + country), and type a **line under the date** that runs across the photo in plain capitals. The date sits a little higher, keeping the caption's row free. The stamp's font choice is gone (user fonts are for the info frame); settings move to v5.
- Fixed: the accent-folding table was misaligned, so some accented letters searched or printed as the wrong letter (for example "đ" as "c").
- **Camera-brand logos in the info frame (FR-040):** Apple, Canon, Fujifilm, Hasselblad, Leica, Lumix, Nikon, Phase One, Ricoh, Samsung and Sony logos come with the app (public-domain renderings from Wikimedia Commons; trademarks of their owners) and are picked from the EXIF make. Choose *Classic* (black wordmarks, colour accents kept), *Brand colours* or *One colour*; on a black frame, dark parts turn white. Wordmarks are sized to the frame text. Cameras still on the default typeset name switch to their logo once (settings v4); your own PNG and typed names are kept.
- Switching off the date stamp or the info frame now folds its options away, so the settings panel and the preview stay short.
- README (English and Vietnamese): system requirements for running and building, with measured memory and cache sizes.
- **Rename a project in place (FR-023):** double-click its name in the sidebar. A saved project's file is renamed in its folder (never over another file); an untitled project uses the name for its first Save. Enter saves, Esc cancels.
- The sidebar, now a little wider, and the README credit the author: "Fotollection by @hoangphi01".
- **Better best-shot groups (FR-041):** similar shots are now matched by layout and colour instead of a dHash, and re-shooting closer (zooming) no longer splits a group. On 95 hand-labelled photos the app now finds 15 of 22 groups instead of 3, still with no wrong group, and its suggestion was acceptable in all 6 groups with a clear best. A shot with the face turned away no longer wins on background sharpness. Analyses are recomputed once (cache version 2). The README shows the full evaluation.
- The library toolbar adapts to the space between the sidebars: button labels shrink to icons first, then the filters wrap to a second row, so nothing is cut off on smaller windows.
- **HEIC import (FR-001):** iPhone photos (HEIC/HEIF) open, preview and export like JPEG, with their EXIF date and GPS. They are decoded by the operating system (ImageIO on macOS; on Windows, the HEIF and HEVC extensions from the Microsoft Store). Without the Windows extensions, the skipped-files list says what to install.
- **Project files like `.docx` (FR-023):** File → Save / Save As / Open / New (Cmd/Ctrl+S, Shift+Cmd/Ctrl+S, Cmd/Ctrl+O, Cmd/Ctrl+N), also in the sidebar. A `.fotollection` file carries the edits, print settings, info frame, camera profiles and the fonts and logos it uses, so it opens the same on another machine. Copying the project together with its photos keeps working; moved folders can be relinked. The window title shows unsaved changes, New and Open ask before discarding them, and double-clicking a `.fotollection` file opens it. The menu and title follow the interface language.
- **Vietnamese interface (FR-020):** choose English or Tiếng Việt at the top of the settings panel; it applies at once and is remembered. Every label, hint and error message is translated, and numbers use the language's decimal separator (`10 × 13,3 cm`).
- The info-frame controls (on/off, crop or fit, colour, date format, font, shown items) are now also in the preview, so the frame can be tuned without leaving it.

- **README in Vietnamese** (`README.vi.md`, the **VI** link at the top) and a reworked Performance section: four charts, each with a reference line (budget, 2 GB ceiling, browser) and a table under it. Charts are generated in English and Vietnamese by `bench/chart.mjs`; `bench/batch.sh` reproduces the 5,000-photo memory check.
- Simplified logo tile (no wordmark) for the sidebar, favicon and every app-icon size up to 64 px, so the icon stays readable in the menu bar, taskbar and Finder lists. `scripts/small-icons.py` regenerates them.
- `docs/RESULT.md`: the latest measured speed and memory in one page, including a 5,035-photo batch (peak 1.47 GB, flat versus 95 photos). `measure_folder` gains `--inflight N`, `--discard-output`, per-phase RSS and a preview time breakdown.
- **Original print size (FR-045):** keep each photo's own aspect ratio with no crop (10 cm on the short edge, e.g. 10 × 13.3 cm for 4:3). Works with the date stamp, the info frame and the DPI check.
- **Culling (FR-022):** rate photos 0–5 stars and flag them pick/reject with `0`–`5`, `P`, `X` and `U` in the grid or the preview, or click the stars and flags in the preview. Filter the library by flag and minimum rating; export writes exactly the filtered photos. Rejected photos are dimmed, never deleted, and every change is undoable.
- **PNG import (FR-001):** PNG photos and screenshots (8/16-bit, greyscale, palette) open, preview and export like JPEG. Transparency prints as white, and an embedded ICC profile is kept.
- `docs/EXTENSION.md`: registry of Fotollection's file types (project, preset, settings, fonts, logos, exports, inputs) with MIME types, UTIs, versions, limits and compatibility rules.
- **Rust vs browser benchmark:** `bench/run.sh <photo-folder>` measures export, preview and peak memory for the Rust core and for headless Chromium on the same photos, then draws `docs/images/bench.svg` for the README. Measured: export is a tie, the browser previews faster, and Rust uses 20% less memory and encodes 4:4:4.
- **Crop editor:** dims everything outside the crop, drag to move, wheel/pinch/slider zoom, arrow-key nudge, and a live label with the printed size in cm, pixel size and DPI band. Crops are stored as centre + zoom, so they survive print-size changes.
- **Session project (FR-023):** edits autosave atomically to an implicit project. The last session's folders are re-scanned and edits restored at launch. Undo/redo with Cmd/Ctrl+Z.
- **Custom fonts (FR-042):** import OTF/TTF, then choose them for the date stamp and the info frame. Files are validated and copied; removing a font falls back to Inter.
- **Custom print sizes (FR-043):** enter in cm or inches (e.g. 4 × 6 in), saved and usable everywhere.
- Crop editor edge labels (width on top, height on the left, in cm or in) and an aspect-ratio pill.
- The print-size picker shows each size's aspect ratio (`10 × 15 cm · 2:3`, `4 × 6 in · 2:3`).
- `settings.json` schema v2 with automatic migration from v1.
- **Info frame ("Shot on …", FR-040):**
  - A white or black bar with the brand, camera, lens, focal length, aperture, shutter, ISO, photo style and date.
  - The whole print keeps the selected size, with the photo cropped to fill or fitted whole.
  - Every item can be switched on or off, LUMIX Lab style.
  - Per-camera overrides for manual lenses, names and styles, remembered across sessions.
  - Optional user PNG logo per camera ([ADR-0014](docs/adr/0014-user-supplied-brand-logos.md)).
- `settings.json`, the first persisted file: render, frame and camera profiles, versioned and written atomically (DATABASE.md §6).
- EXIF now reads the lens, focal length, aperture, shutter and ISO.
- **First working slice (JPEG):**
  - Open a folder (or drop it on the window) to get a virtualized thumbnail grid with instant EXIF thumbnails.
  - Preview the exact print: centred crop for 8 print sizes, and a date stamp (3 formats, 4 corners, white or retro orange) inside the safe area.
  - Export at full resolution to `YYYY-MM-DD/` folders, with progress and cancel, never overwriting files and never writing GPS.
- Rust core: import validation (magic bytes, size, polyglot), allowlisted EXIF, columnar library, `JpegCodec` trait with pure-Rust codecs ([ADR-0012](docs/adr/0012-pure-rust-codecs-first.md)), render plan, T0/T1 thumbnail cache, T2 preview LRU, bounded parallel export.
- Print preview **dialog** with an inspector: capture time and source, camera, pixel and file size, GPS, effective print DPI with a quality band (FR-026), crop mode, and quick stamp controls.
- Loading feedback: skeleton and spinner thumbnails that fade in, plus a progress bar with percentage for reading photos and preparing thumbnails (`task-progress` event).
- Typed IPC events (`library-changed`, `thumb-ready`, `export-progress`, `export-finished`) and the `fotothumb://` image protocol.
- Phase 0 scaffold: a Cargo workspace (`fotollection-core` + the Tauri 2 shell), and a React + Tailwind + shadcn/ui + Lucide UI shell (library sidebar, empty state, print-settings panel) with typed IPC through `tauri-specta`.
- Retro app logo and generated platform icons.
- CI: build, lint and test on macOS and Windows; stale-bindings check; `cargo deny`, `npm audit` and DOM-API security gates.
- Reproducible machine setup: `scripts/bootstrap.sh` / `bootstrap.ps1`, pinned Rust toolchain and Node version, committed lockfiles.
- Documentation framework: PRD, architecture, optimization, style rules, security, testing, release, roadmap, status, glossary, and ADRs 0001–0007.
- `LICENSE`: GPL-3.0-only ([ADR-0008](docs/adr/0008-license-gpl3-with-relicensing-option.md)). Distribution policy: unsigned builds with checksums ([ADR-0009](docs/adr/0009-unsigned-distribution.md)).
- `docs/DEVELOPMENT.md` and `scripts/clean.sh`, a tiered cleanup of build artifacts (sweep, release, full, deep).
- Docs CI: markdownlint, link check, and the requirement-ID consistency check (`scripts/check-docs.sh`).

### Changed

- **Releases (ADR-0021):** one version for the whole app, following Semantic Versioning (0.x until the MVP is complete). `scripts/release.sh` prepares a release commit; when it reaches `main`, CI builds the macOS (universal) and Windows installers and publishes them on GitHub Releases with checksums and first-launch steps. There is no in-app updater: the app stays fully offline and shows its version in the settings panel (FR-021).
- **A look from the logo:** the app's buttons, switches and checkboxes take the logo's teal, the panels a faint cream (a warm brown in dark mode), and the selected photo sits in a wood-coloured frame. Surfaces next to photos stay near-neutral, and the preview stays on black, so colours are still judged correctly.
- **Square print sizes 10×10, 13×13 and 20×20 (FR-006):** with them, every size on the price list of the CEWE photo service at dm and Rossmann is built in.
- **Print-size advice (FR-026):** under the print-size picker, one sentence tells how large the selected photo prints sharp and still acceptable ("6240 × 4160 px (26 MP): sharp up to about 35 × 53 cm, acceptable up to 70 × 106 cm"), and every size in the list carries a badge: green Sharp, yellow Acceptable, red Pixelated.
- **Website, print shops:** a section shows that the CEWE kiosk sizes of dm and Rossmann are built in, with the three steps from trip to prints (Fotollection is not affiliated with them).
- **Website:** <https://hoangphi01.github.io/fotollection/> (English) and `/vi/` (Vietnamese), with download buttons for the latest release, screenshots of the real app on CC0 demo photos, three benchmark charts, link previews, self-hosted fonts, and Google Analytics as the only third-party script (the website only; the app has none). Its colours come from the logo: a cream background, teal, wood and leaf green.
- The Vietnamese place-format buttons are shorter ("Thành phố", "+ mã nước", "+ tên nước"), so they no longer overflow the settings panel.
- **Previews are faster: 26 → 17 ms**, now faster than the same work in Chromium. On-screen JPEGs (previews, thumbnails) use 4:2:0 and plain Huffman tables; exported prints keep 4:4:4 and optimized tables.
- Thumbnails no longer wait behind background work: there are separate interactive and background pools, and the core is optimized in dev builds.
- CSP now allows inline styles (scripts stay strict) so Radix overlays work in release builds ([ADR-0013](docs/adr/0013-csp-inline-styles.md)).
- New `docs/DATABASE.md`: the single source of truth for persisted data, with a **normalized (3NF) data model** (integer keys, one fact in one place, sparse delta rows, no derived data on disk), columnar in-memory layout (~42 B per photo), project-file schema v1, migrations, atomic writes, the preset model, the thumbnail cache, and criteria for adopting SQLite.
- PRD: FR-038 (Lightroom-style partial, stackable, linked print presets with `.fotopreset` sharing) and FR-039 (Photo Style looks modelled on Lumix GX9, including a built-in "C200 Natural"). [ADR-0011](docs/adr/0011-photo-style-looks.md) sets the scope boundary (global looks only).
- PRD expanded with 16 requirements (FR-022…FR-037) covering culling, projects with undo, mixed print sizes, resolution warnings, sRGB conversion, safe area and borders, clock correction, GPS borrowing, manual date and place, a retro LED date stamp, lab presets, captions, multi-photo layouts, sharpening and contact sheets. Also NFR-012/013 and [ADR-0010](docs/adr/0010-project-file-format.md) (project file format).
- Project renamed from PhotoOrder to **Fotollection**.
- Direction changed from a browser web app to an offline **Tauri 2 desktop app** with a Rust core ([ADR-0001](docs/adr/0001-desktop-tauri-over-web.md)).
- UI stack changed to React, shadcn/ui and Lucide ([ADR-0007](docs/adr/0007-ui-react-shadcn-lucide.md)).

### Removed

- `PLAN.md`. Its content was split into `docs/PRD.md`, `docs/ARCHITECTURE.md` and `docs/ROADMAP.md`.

## [0.1.0-beta.1] - 2026-10-24

Trial run of the release pipeline (macOS `.dmg` and Windows `-setup.exe`). Everything in it is released as 0.1.0.
