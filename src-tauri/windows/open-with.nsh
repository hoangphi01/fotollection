; SPDX-License-Identifier: GPL-3.0-only
; "Open with" for photos on Windows (FR-054, ADR-0024), hooked into Tauri's NSIS installer.
;
; Tauri's own file associations set each extension's default value, which would make
; Fotollection the default viewer for every JPEG. The user chooses that, not us: these hooks
; only list Fotollection under each type's OpenWithProgids, with one ProgID for photos, and
; the uninstaller removes exactly what was added. SHCTX follows the install mode
; (current user or all users), like Tauri's own keys.

!define FOTOLLECTION_PROGID "Fotollection.Photo"

!macro FOTOLLECTION_FOR_EACH_TYPE MACRO
  !insertmacro ${MACRO} "jpg"
  !insertmacro ${MACRO} "jpeg"
  !insertmacro ${MACRO} "jpe"
  !insertmacro ${MACRO} "png"
  !insertmacro ${MACRO} "heic"
  !insertmacro ${MACRO} "heif"
  !insertmacro ${MACRO} "hif"
  !insertmacro ${MACRO} "rw2"
  !insertmacro ${MACRO} "cr2"
  !insertmacro ${MACRO} "cr3"
  !insertmacro ${MACRO} "nef"
  !insertmacro ${MACRO} "arw"
  !insertmacro ${MACRO} "raf"
  !insertmacro ${MACRO} "orf"
  !insertmacro ${MACRO} "dng"
!macroend

!macro FOTOLLECTION_ADD_TYPE EXT
  WriteRegStr SHCTX "Software\Classes\.${EXT}\OpenWithProgids" "${FOTOLLECTION_PROGID}" ""
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\SupportedTypes" ".${EXT}" ""
!macroend

!macro FOTOLLECTION_REMOVE_TYPE EXT
  DeleteRegValue SHCTX "Software\Classes\.${EXT}\OpenWithProgids" "${FOTOLLECTION_PROGID}"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\${FOTOLLECTION_PROGID}" "" "Photo"
  WriteRegStr SHCTX "Software\Classes\${FOTOLLECTION_PROGID}" "FriendlyTypeName" "Photo"
  WriteRegStr SHCTX "Software\Classes\${FOTOLLECTION_PROGID}\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr SHCTX "Software\Classes\${FOTOLLECTION_PROGID}\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe" "FriendlyAppName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  !insertmacro FOTOLLECTION_FOR_EACH_TYPE FOTOLLECTION_ADD_TYPE
  ; Tell Explorer the associations changed (SHCNE_ASSOCCHANGED).
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  !insertmacro FOTOLLECTION_FOR_EACH_TYPE FOTOLLECTION_REMOVE_TYPE
  DeleteRegKey SHCTX "Software\Classes\${FOTOLLECTION_PROGID}"
  DeleteRegKey SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
