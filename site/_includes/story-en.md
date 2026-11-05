## I. The starting line {#start}

One day I realised I had lived twenty years. My phone held tens of thousands of photos, and I had almost never opened them again. Things from five or six years ago were starting to fade: an afternoon, a face, the name of a street. The photos were still there, sitting quietly in a folder I never visited.

Then I came to Europe to study. Winters here are long and the dark comes early, so there are more quiet hours than I expected. In those hours I scrolled back through the old years and felt a soft kind of sadness: good memories, stories left unfinished, all of it behind a sheet of glass. My grandparents had thick albums, every photo held by paper corners, a line of handwriting on the back. My generation takes more pictures than any before it and keeps fewer of them. I thought: let me start there. I wanted to make something that turns those memories back into things you can hold.

I think of it as a marathon. Nobody runs forty-two kilometres in one breath, so I only asked what the first step was. In September 2026, the first step was a plan for a web app called "PhotoOrder". The problems I wrote down then still stand: people shoot thousands of photos and print almost none; prints say neither when nor where, and in a few years nobody remembers; labs crop photos to 2:3, 3:4 or square on their own, sometimes cutting off heads and horizons; and online tools want your photos uploaded, while their GPS data can point straight at your home.

## II. The first strides {#first-steps}

The project's first commit came on the afternoon of 23 October 2026, with three pages in it: a plan, a note on safety and a progress page, and not a line of code. Thirty-five minutes later the project had a new name, Fotollection, and a different direction altogether. I dropped the web app for a desktop app that works fully offline, with its core written in Rust. The reason is in the very first architecture decision: the people it is for take photography seriously, so speed and print quality come first, and a browser caps a tab's memory and keeps me from controlling how a photo is encoded.

A few rules were set down that same day, and they still hold. Place names are looked up on your own computer, from a bundled GeoNames dataset, so your coordinates are never sent to a server. The code is licensed under GPL-3.0, so nobody can turn it into a closed copy. The installers are not signed, because this is a free project with no budget for yearly certificates. And the notes on the code change in the same commit as the code itself, a habit that later kept the whole project from losing its way.

That evening the first slice ran from end to end: import, preview, date stamp, JPEG export. Then I did what every runner should do early: I timed it. I put the Rust build next to a version running in the browser, on the same 95 photos of 16 MP on the same machine. The result surprised me. Export ran at nearly the same speed, 18.0 photos a second against 17.9. My first guess, that the browser would be slow, was wrong, and I wrote down that word, wrong. The real reasons for a native app lie elsewhere: prints encoded at 4:4:4 rather than 4:2:0, memory that stays bounded even at five thousand photos, colour profiles that are kept, and files written straight to disk. On day one I learnt that a measured number is worth more than a belief.

## III. The first fall {#first-fall}

The first bug fix had nothing to do with photos: the test programs would not start on Windows because a system manifest was missing. A small stumble, and a reminder that every operating system is a different road surface.

Two days later the stumbles began to teach me more. Once, a red mark from the automatic checks sat unnoticed through three pushes, because a list in the project's notes was numbered "7." twice; from then on the notes are checked in the same step I always run before a commit. Once, opening a photo with the app closed did nothing at all: macOS handed the file over before the app had built its state. And the first "Shot on" frame read "NIKON CORPORATION NIKON D750", because the Make field in EXIF names a company, not a brand.

After falls like these I started writing down the lessons. Every bug that took more than one look goes into them: what was seen, why, the fix, and the rule from now on. They are the first thing I read whenever something breaks. Running a long way is not about never falling; it is about remembering where you fell.

## IV. A print must not look flat {#prints}

I had noticed it for a long time: photos printed through web editors often look flat, the colour dull and blotchy, with less depth than the original. On 24 October I put that suspicion to a pixel-by-pixel test. Against a typical web-editor export, Fotollection's colour error was ΔE2000 0.19 on average instead of 1.39, it kept 100 % of the fine detail instead of 13 %, it kept 99.9 % of the saturation of Display P3 photos instead of 86.8 %, and it kept the ICC colour profile on 20 photos out of 20. The causes are no mystery: a dropped colour profile, a photo shrunk and blown up again, colour halved. Leave those out, and the print keeps the photo.

In those days the most concrete finish line was the self-service printers in dm and Rossmann stores, and every size on the price list of the CEWE photo service there went into the app. The website was born at the same time, in English and Vietnamese, opening with a question I still find true: thousands of photos, only looked at on the plane home? On 25 October it moved to fotollection.org, and that same day brought seven releases, from v0.3.0 to v0.7.0: the print basket, a viewer with zoom, then camera RAW. Those were the fastest kilometres, with fresh legs on a new road.

## V. Turned away at the roadside {#refusal}

On 26 October I applied to the SignPath Foundation for free code signing of the Windows installer. The answer came the same day: they look for outside signs of public trust, such as stars, forks, contributors and independent articles, and the project did not show enough yet; I could apply again later. That refusal did not stop me. It only showed how much road was still ahead. The installers are still unsigned, and every release lists SHA-256 checksums, so anyone can check the file they downloaded.

The same day I made a mistake I still remind myself of. The website and the v0.7.0 notes said camera RAW opened with nothing to install, and on Windows that was not true. I found it myself, not a test. The lesson became a rule: never write "supports" before it has been tried on every system. And from then on I decided that RAW must be read by code inside Fotollection, not by something the user has to install.

## VI. Making my own shoes {#shoes}

A long-distance runner ends up wanting shoes that fit their own feet. On 27 October Fotollection got a RAW decoder of its own, DNG first. I wanted colour as good as the big programs, without anyone installing anything extra. On a Ricoh GR III DNG the new decoder took 355 ms, used 526 MB of memory, and came within ΔE2000 2.67 of the camera's own JPEG; the library I had used before needed 640 ms and 1.33 GB, and was 4.08 away.

That 2.67 has a story of its own. While my DNG was still about one unit off, I was sure the colour was to blame. Mapping the error showed the centre of the frame brighter than its edges: it was the lens's vignetting, not the colour. The lesson I wrote then: map the error by region before adding parameters. The same week v0.8.0 arrived, named "Places and Lightroom": camera clocks set right, dates and places set by hand, stars read from Lightroom. Then came two nearly silent days, the 28th and the 29th, the only real pause in the whole race.

## VII. Finding the rhythm {#rhythm}

By the end of October the app could do a great deal, and the more it did, the harder it was for someone new. On 31 October I split it into three modes. **Simple** is the shortest way from a folder to prints: look at each photo once, keep it or drop it, then export. **Collage** puts two photos on one print, the way a half-frame film camera put two frames on one negative, so a set costs half as much. **Pro** is everything built before.

That was when Fotollection found its rhythm. It does not try to be an all-purpose photo manager; it is a short, clear road from a phone or a memory card to a sheet of photo paper in your hand. v0.9.0 "Three modes" shipped on 2 November. The bugs of those days became rules too: a field that made the whole panel jump while typing gave the rule "nothing blinks, nothing jumps". A collage sheet exports in 214 ms, quicker than its two photos one by one (633 ms), and the grid of sheets redraws from thumbnails in 29 ms instead of 4,088.

## VIII. The first cheers, and the green photos {#cheering}

On 1 November real users spoke for the first time, and they showed me what no test had. Someone on v0.8.0 opened their folder and found half the photos shown as green tiles with fine horizontal stripes. I suspected the RAW decoder first, but the files were JPEGs. The fast JPEG decoder I relied on read one rare file layout wrongly and still reported success. v0.8.1 "Green photos fixed" put a reference decoder behind the fast one, and from then on there is a rule I count among the core ones: a fast decoder is trusted only on the layouts a test proves.

The same day came another report: with three photos ticked, the export button said "Export 12 photos". v0.8.3 fixed exactly that, and added a red Report button so people could tell me more. That day Fotollection also arrived on Linux. I have no Linux machine, but Linux has few good photo apps, so it is a place where Fotollection can be the obvious choice. The first people to cheer did not stand at the finish line; they stood along the road and pointed out where I was running wrong.

## IX. Saving strength for the long run {#long-run}

A long race asks for things nobody sees. The app works offline and sends nothing, so when something goes wrong, the only clue a user can read to me is a code. v0.9.1 gave every error its own code, like `ERR-IMG-0006`, with an error log kept on the computer itself, free of photos, places and file names, and a *Copy diagnostics* button to paste it into a report.

Some faults made me stop and catch my breath. Zooming into one RAW photo once froze every preview for five minutes, not because it was slow, but because a lock was held while a pool of threads waited for itself. The rule since: never hold a lock while parallel work runs. On 2 November I wrote down the direction for the core in one short sentence I want to keep to the end: simple software, never careless.

## X. Running through Prague {#prague}

On 3 November I wanted something to send to people far away: postcards printed from my own photos. One half of the print is the photo, the other the back of a postcard, with a stamp box, writing lines and text typed as if on a typewriter, laid out as in my own sketch. v0.9.4 brought postcards into Collage mode, followed by a smaller stamp box that the stamp hides, a fourth address line when one is needed, and a red mark when the text does not fit.

Then I put the app to a real test: a whole folder copied off a Lumix GX9 after a weekend in Prague, 396 JPEGs of 20 MP and 26 RAW files, 4.7 GB in all. The real trip broke two budgets that the test sets had never touched: export ran at only 7.6 photos a second, and memory climbed to 2.14 GB, past the 2 GB ceiling. I measured each step again. The encoder turned out to hold every coefficient of each photo only to make its file less than 1 % smaller, and half of the memory "left over" belonged to the benchmark itself. v0.9.7 exports those JPEGs at 10.1 a second, and the whole run stays under 1.31 GB. In the same days a user told me that exporting by day made them hunt for their photos, so since v0.9.6 everything goes into one folder by default, just as a USB stick taken to a print shop wants.

The night trams of Prague now sit on the home page, as a postcard addressed to the National Museum, with "Ahoj Praha!" on its lines. The memory of a weekend has become something you can put a stamp on and send.

## XI. A second wind {#second-wind}

On 5 November two things came together. v0.10.0 gave Fotollection its own RAW engine: every Bayer camera developed by our own code, opened neutral, with the camera's look one click away, and brightness, highlights and shadows worked on the sensor's own data, as a RAW editor does. That same evening I opened my Prague RAW files and the preview's spinner kept turning and never stopped.

I did not guess. A snapshot of every thread in the running app showed the knot: every worker was waiting for a develop, and the develop was waiting for a worker. That was fixed the same night, and then a second, quieter hang in the screen itself. Then I asked for more. In Lightroom, once a folder has loaded, any photo you click is simply there. I wrote down a sentence that changed how I think about the app: it is not good to use little memory, it is good to use memory well.

So since v0.11.0 Fotollection prepares a preview of every photo in the background once the thumbnails are done, keeps it on disk, and gives the photos around the one on screen a quarter of the computer's memory. On the same 422 photos from Prague, v0.9.0 took 274 ms to open a JPEG and 531 ms to open a RAW. v0.11.0 takes 27 ms for either. It felt like a rocket, and like a second wind in a long race.

## XII. How far still {#how-far}

Fourteen days, more than four hundred and thirty commits, twenty-four releases. Counted as a marathon, I have only passed the first kilometres. Ahead are v0.12.0, one window for every setting and a strip of photos under the preview to skim a trip, then v0.13.0 "Looks", with looks of one's own, and further on, version 1.0.

I often ask myself how far I can take Fotollection. Could it one day be like the open-source projects a whole community knows, the repositories tens of thousands of people trust every day? I do not know yet. What I know is that each photo printed is a memory no longer behind glass, like my grandparents' albums once were. If you have read this far, thank you for running a stretch of the road with me. The road is still long, and I am still running.
