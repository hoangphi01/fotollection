// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import {
  commands,
  events,
  type CameraProfile,
  type FrameConfig,
  type FrameSettings,
} from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

type Result<T> =
  | { status: 'ok'; data: T }
  | { status: 'error'; error: { code: string; message: string } }

/**
 * Info-frame settings and camera profiles (FR-040). Every change returns the new config
 * and settings revision, so previews refresh.
 */
export function useFrameConfig(onRevision: (revision: number) => void) {
  const [config, setConfig] = useState<FrameConfig | null>(null)
  const [error, setError] = useState<string | null>(null)

  const apply = useCallback(
    (result: Result<FrameConfig | null>) => {
      if (result.status === 'ok') {
        if (result.data) {
          setConfig(result.data)
          onRevision(result.data.revision)
        }
        setError(null)
      } else {
        setError(errorText(result.error))
      }
    },
    [onRevision],
  )

  useEffect(() => {
    const load = () => commands.getFrameConfig().then(apply)
    void load()
    // New cameras appear in the profile list after an import.
    const unlisten = events.libraryChanged.listen(() => void load())
    return () => void unlisten.then((fn) => fn())
  }, [apply])

  const setSettings = useCallback(
    async (change: (current: FrameSettings) => FrameSettings) => {
      if (!config) return
      const next = change(config.settings)
      setConfig({ ...config, settings: next }) // optimistic; Rust stores it
      apply(await commands.setFrameSettings(next))
    },
    [config, apply],
  )

  const setProfile = useCallback(
    async (profile: CameraProfile) => apply(await commands.setCameraProfile(profile)),
    [apply],
  )
  const pickLogo = useCallback(
    async (cameraId: number) => apply(await commands.pickCameraLogo(cameraId)),
    [apply],
  )
  const clearLogo = useCallback(
    async (cameraId: number) => apply(await commands.clearCameraLogo(cameraId)),
    [apply],
  )

  return { config, error, setSettings, setProfile, pickLogo, clearLogo }
}

export type FrameConfigApi = ReturnType<typeof useFrameConfig>
