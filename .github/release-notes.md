## Install

Fotollection is free and works offline. The installers are not signed with a paid certificate yet ([why](https://github.com/hoangphi01/fotollection/blob/main/docs/adr/0009-unsigned-distribution.md)), so the first launch needs one extra step:

- **macOS 11 or later** (Apple Silicon and Intel): open the `.dmg` and drag Fotollection to Applications. On the first launch macOS blocks it; open **System Settings → Privacy & Security** and click **Open Anyway**.
- **Windows 10/11:** run the `-setup.exe` (or the `.msi`). If SmartScreen says "Windows protected your PC", click **More info → Run anyway**.
- **Linux** (64-bit, glibc 2.35 or later: Ubuntu 22.04+, Debian 12+, Mint 21+, Fedora 36+, Arch…; [which distributions](https://github.com/hoangphi01/fotollection#linux)):
  - **AppImage**, any of these distributions: `chmod +x Fotollection_*.AppImage`, then run it. Ubuntu 24.04 and newer need FUSE 2 once: `sudo apt install libfuse2t64`.
  - **`.deb`**, Ubuntu, Debian and Mint: `sudo apt install ./Fotollection_*_amd64.deb` (`_arm64.deb` on ARM).
  - Not on Linux yet: HEIC photos and camera capture over USB.

Installing over an older version keeps your settings, camera profiles and projects; to move to another computer, use Settings → Data → Export settings. Check a download against `SHA256SUMS.txt` with `shasum -a 256 <file>` (macOS), `certutil -hashfile <file> SHA256` (Windows) or `sha256sum -c SHA256SUMS.txt --ignore-missing` (Linux).
