## Install

Fotollection is free and works offline. The installers are not signed with a paid certificate yet ([why](https://github.com/hoangphi01/fotollection/blob/main/docs/adr/0009-unsigned-distribution.md)), so the first launch needs one extra step:

- **macOS 11 or later** (Apple Silicon and Intel): open the `.dmg` and drag Fotollection to Applications. On the first launch macOS blocks it; open **System Settings → Privacy & Security** and click **Open Anyway**.
- **Windows 10/11:** run the `-setup.exe` (or the `.msi`). If SmartScreen says "Windows protected your PC", click **More info → Run anyway**.

Installing over an older version keeps your settings, camera profiles and projects. Check a download against `SHA256SUMS.txt` with `shasum -a 256 <file>` (macOS) or `certutil -hashfile <file> SHA256` (Windows).
