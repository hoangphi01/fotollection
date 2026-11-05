### Why memory does not grow with the folder

Fotollection keeps a few bytes per photo in its library (13 MB for 5,035 photos), writes thumbnails to a disk cache instead of holding them, keeps a bounded number of previews in memory, and works on only as many full-size photos at once as fit its budget. The grid on screen is virtual: only the tiles you can see exist. To test it, the same 95 photos were linked 53 times into a folder of 5,035 files:

| | 95 photos | 5,035 photos |
|---|---|---|
| Import and EXIF | 5 ms, 7 MB | 383 ms, 13 MB |
| 400 px thumbnails to the disk cache | 47.8 photos/s | 46.4 photos/s |
| Export, 6 at once | 18.0 photos/s | 15.4 photos/s, 0 failed |
| Peak memory of the whole run | 1.35 GB | 1.45 GB |

53 times more photos cost 0.1 GB more. The long export is a little slower per photo only because it ran for more than five minutes, and a laptop's processor slows down under sustained load.
