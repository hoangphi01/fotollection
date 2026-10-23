// SPDX-License-Identifier: GPL-3.0-only
import { open } from '@tauri-apps/plugin-dialog'
import { useCallback, useState } from 'react'

/**
 * Folder picker (FR-001 entry point). Import itself lands in Phase 1;
 * for now the chosen folders are only listed.
 */
export function useOpenFolder() {
  const [folders, setFolders] = useState<string[]>([])

  const openFolder = useCallback(async () => {
    try {
      const picked = await open({ directory: true, multiple: false })
      if (typeof picked === 'string') {
        setFolders((prev) => (prev.includes(picked) ? prev : [...prev, picked]))
      }
    } catch (error) {
      // Outside the Tauri shell (plain `vite dev`) the dialog plugin is unavailable.
      console.warn('folder dialog unavailable', error)
    }
  }, [])

  return { folders, openFolder }
}
