// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
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
  const [config, setConfigState] = useState<FrameConfig | null>(null)
  const [error, setError] = useState<string | null>(null)
  // The config each change builds on, so two quick changes never drop the first.
  const current = useRef<FrameConfig | null>(null)
  const setConfig = useCallback((next: FrameConfig) => {
    current.current = next
    setConfigState(next)
  }, [])

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
    [onRevision, setConfig],
  )

  // The number of the latest change: a reload asked for before it is dropped.
  const latest = useRef(0)
  useEffect(() => {
    const load = () => {
      const asked = latest.current
      return commands.getFrameConfig().then((result) => asked === latest.current && apply(result))
    }
    void load()
    // New cameras appear in the profile list after an import.
    const unlisten = events.libraryChanged.listen(() => void load())
    return () => void unlisten.then((fn) => fn())
  }, [apply])

  // Only the answer to the latest change replaces the settings on screen, as in
  // useRenderSettings: an older one arriving late would undo what was typed since.
  const setSettings = useCallback(
    async (change: (current: FrameSettings) => FrameSettings) => {
      const base = current.current
      if (!base) return
      const next = change(base.settings)
      setConfig({ ...base, settings: next }) // optimistic; Rust stores it
      const asked = ++latest.current
      const result = await commands.setFrameSettings(next)
      if (asked === latest.current) apply(result)
      else if (result.status === 'ok' && result.data) onRevision(result.data.revision)
    },
    [apply, onRevision, setConfig],
  )

  /** Changes some fields of one camera's profile, on top of its latest values. */
  const setProfile = useCallback(
    async (id: number, patch: Partial<CameraProfile>) => {
      const base = current.current
      const view = base?.cameras.find((c) => c.profile.id === id)
      if (!base || !view) return
      const profile = { ...view.profile, ...patch }
      setConfig({
        ...base,
        cameras: base.cameras.map((c) => (c.profile.id === id ? { ...c, profile } : c)),
      }) // optimistic; Rust stores it
      const asked = ++latest.current
      const result = await commands.setCameraProfile(profile)
      if (asked === latest.current) apply(result)
      else if (result.status === 'ok' && result.data) onRevision(result.data.revision)
    },
    [apply, onRevision, setConfig],
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
