// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useState } from 'react'
import {
  commands,
  events,
  type CameraSettings,
  type CameraShot,
  type CaptureFormat,
  type ExposureSetting,
} from '@/ipc/bindings'
import { errorText } from '@/lib/errors'

/** Connecting a camera works on Windows for now (ADR-0028). */
export const CAMERA_AVAILABLE = navigator.userAgent.includes('Windows')

type Phase = 'idle' | 'connecting' | 'connected'

/**
 * Tethered capture (FR-062, FR-063): the camera session, its live-view frame number, the
 * shots taken, and the session tabs that offer Save all.
 */
export function useCapture() {
  const [phase, setPhase] = useState<Phase>('idle')
  const [name, setName] = useState('')
  const [settings, setSettings] = useState<CameraSettings | null>(null)
  const [frame, setFrame] = useState(0)
  const [shots, setShots] = useState<CameraShot[]>([])
  // Presses sent and not answered yet: the Capture button shows them.
  const [pending, setPending] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [sessions, setSessions] = useState<number[]>([])

  useEffect(() => {
    const loadSessions = () =>
      commands.cameraSessions().then((result) => {
        if (result.status === 'ok') setSessions(result.data)
      })
    void loadSessions()
    const unlisten = [
      events.cameraFrame.listen((e) => setFrame(e.payload.seq)),
      events.cameraSettingsChanged.listen((e) => setSettings(e.payload.settings)),
      events.cameraShot.listen((e) => {
        setShots((prev) => [e.payload, ...prev])
        setPending((n) => Math.max(0, n - 1))
        setError(null)
      }),
      events.cameraShotFailed.listen((e) => {
        setPending((n) => Math.max(0, n - 1))
        setError(errorText(e.payload.issue))
      }),
      events.cameraStatus.listen((e) => {
        if (e.payload.connected) return
        setPhase('idle')
        setPending(0)
        setFrame(0)
        if (e.payload.issue) setError(errorText(e.payload.issue))
      }),
      // Session tabs come and go with the library (Save all, Delete photos, a new shot).
      events.libraryChanged.listen(() => void loadSessions()),
      events.documentChanged.listen(() => void loadSessions()),
    ]
    return () => {
      for (const p of unlisten) void p.then((off) => off())
    }
  }, [])

  const connect = useCallback(async () => {
    setPhase('connecting')
    setError(null)
    setShots([])
    setFrame(0)
    const result = await commands.cameraConnect()
    if (result.status === 'ok') {
      setName(result.data.name)
      setSettings(result.data.settings)
      setPhase('connected')
    } else {
      setError(errorText(result.error))
      setPhase('idle')
    }
  }, [])

  const disconnect = useCallback(async () => {
    await commands.cameraDisconnect()
    setPhase('idle')
    setPending(0)
    setFrame(0)
  }, [])

  const capture = useCallback(async () => {
    setPending((n) => n + 1)
    const result = await commands.cameraCapture()
    if (result.status === 'error') {
      setPending((n) => Math.max(0, n - 1))
      setError(errorText(result.error))
    }
  }, [])

  const step = useCallback(async (setting: ExposureSetting, delta: number) => {
    const result = await commands.cameraStep(setting, delta)
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  const setFormat = useCallback(async (format: CaptureFormat) => {
    const result = await commands.cameraSetFormat(format)
    if (result.status === 'error') setError(errorText(result.error))
  }, [])

  const saveSession = useCallback(async (folder: number) => {
    const result = await commands.saveCameraSession(folder)
    if (result.status === 'error') setError(errorText(result.error))
    else setError(null)
    return result.status === 'ok' && result.data !== null
  }, [])

  const discardSession = useCallback(async (folder: number) => {
    const result = await commands.discardCameraSession(folder)
    if (result.status === 'error') setError(errorText(result.error))
    else setError(null)
  }, [])

  return {
    phase,
    name,
    settings,
    frame,
    shots,
    pending,
    error,
    sessions,
    /** The tab of this session's shots, once there is one. */
    sessionFolder: shots[0]?.folder ?? null,
    connect,
    disconnect,
    capture,
    step,
    setFormat,
    saveSession,
    discardSession,
    clearError: () => setError(null),
  }
}

export type CaptureApi = ReturnType<typeof useCapture>
