# Recording a camera's USB traffic on Windows

> **Status:** Living · **Last reviewed:** 2026-10-26
>
> How to record what a maker's own app sends to a camera over USB, so its protocol can be written down in our own words ([ADR-0026](../adr/0026-cameras-through-the-os-usb-stack.md), clean-room rule in [README.md §4](README.md#4-clean-room-rule)). First use: the Sony a6400 with Imaging Edge Remote.

## 1. Install (once)

1. **Wireshark** from [wireshark.org](https://www.wireshark.org/download.html) (free, GPL; only used as a tool, nothing of it goes into Fotollection). During the install, **tick "USBPcap"** (it records USB). Restart Windows afterwards.
2. **The maker's tether app.** For Sony: **Imaging Edge Desktop** from Sony's site; its **Remote** app drives the camera.
3. **The repository:** `git pull` in the Fotollection folder, so `scripts\camera-capture.ps1` is there.

## 2. Prepare the camera (Sony a6400)

- **MENU → Setup → USB Connection → PC Remote.**
- **MENU → Network → PC Remote Function → Still Img. Save Dest. → PC+Camera** (if the menu has it).
- A charged battery, a memory card in, and a USB **data** cable (not a charge-only one).
- Plug the camera into the PC and turn it on. Close any app that opens by itself (Photos).

## 3. Record

1. Open **PowerShell as Administrator** (Start → type PowerShell → right-click → Run as administrator).
2. Go to the repository and start the recording:

   ```powershell
   cd C:\path\to\fotollection
   powershell -ExecutionPolicy Bypass -File scripts\camera-capture.ps1 -Name a6400
   ```

3. When it says *Recording*, open **Imaging Edge Remote** and do, slowly, with a few seconds between steps:
   1. connect to the camera;
   2. let the live view run about **5 seconds**;
   3. **shoot one photo** in JPEG;
   4. switch the file format to **RAW** and **shoot one photo**;
   5. **change ISO** once;
   6. close Remote.
4. Back in PowerShell, press **Ctrl+C**. It prints the file's name and size, for example `captures\a6400-20261027-101500.pcapng`.

Write down, roughly, the time of each step (for example "10:15:20 connect, 10:15:40 JPEG shot"): it makes the recording much faster to read.

## 4. Send it to the Mac

- Copy the `.pcapng` to the Mac (USB stick, cloud, or a shared folder) into **Downloads**, and say where it is.
- **Never commit it:** `captures/` and `*.pcapng` are in `.gitignore`. Only our notes in `docs/cameras/<maker>.md` and small hand-made test answers go into the repository.

## 5. If something goes wrong

| What you see | What to do |
|---|---|
| "Wireshark is not installed" | Install it (§1), tick USBPcap, restart |
| "No USBPcap interface found" | Reinstall Wireshark with USBPcap ticked, restart Windows, run PowerShell as Administrator. If USBPcap was installed on its own (for example with `winget install desowin.USBPcap`), copy `C:\Program Files\USBPcap\USBPcapCMD.exe` into `C:\Program Files\Wireshark\extcap\` |
| "Unable to load Npcap" | Wireshark installed without Npcap (for example by `winget --silent`): install it from [npcap.com](https://npcap.com/#download) |
| Remote does not find the camera | Check *USB Connection: PC Remote* and the cable; unplug, turn the camera off and on |
| The file is very large (over 1 GB) | Fine; zip it before copying (right-click → Send to → Compressed folder) |
