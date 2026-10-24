// SPDX-License-Identifier: GPL-3.0-only
import { useCallback, useEffect, useRef, useState } from 'react'
import { commands, events, type Action, type DocumentState } from '@/ipc/bindings'
import { errorText } from '@/lib/errors'
import { isTyping } from '@/lib/keyboard'

export type UnsavedChoice = 'save' | 'discard' | 'cancel'

/** macOS has the native File menu; elsewhere the shortcuts are handled here. */
const isMac = navigator.userAgent.includes('Mac')

type Result<T> =
  | { status: 'ok'; data: T }
  | { status: 'error'; error: { code: string; message: string } }

/**
 * The `.fotollection` document (FR-023): name, unsaved changes, missing folders, and the
 * New / Open / Save / Save As / Relink flows, each asking about unsaved changes first.
 */
export function useProject() {
  const [document, setDocument] = useState<DocumentState>({
    name: null,
    edited: false,
    generation: 0,
    missing: [],
  })
  const [error, setError] = useState<string | null>(null)
  const [question, setQuestion] = useState<((choice: UnsavedChoice) => void) | null>(null)
  const edited = useRef(false)

  useEffect(() => {
    let live = true
    void commands.getDocument().then((d) => {
      if (live) {
        setDocument(d)
        edited.current = d.edited
      }
    })
    const unlisten = events.documentChanged.listen(({ payload }) => {
      setDocument(payload.document)
      edited.current = payload.document.edited
    })
    return () => {
      live = false
      void unlisten.then((fn) => fn())
    }
  }, [])

  /** Runs a command; shows its error, if any. Returns true when it did something. */
  const run = useCallback(async (call: Promise<Result<DocumentState | null>>) => {
    const result = await call
    if (result.status === 'error') {
      setError(errorText(result.error))
      return false
    }
    setError(null)
    return result.data !== null
  }, [])

  const save = useCallback(() => run(commands.saveDocument()), [run])
  const saveAs = useCallback(() => run(commands.saveDocumentAs()), [run])

  /** Before replacing the library: ask, and save if asked to. False means stop. */
  const settleUnsaved = useCallback(async () => {
    if (!edited.current) return true
    const choice = await new Promise<UnsavedChoice>((resolve) => setQuestion(() => resolve))
    setQuestion(null)
    if (choice === 'cancel') return false
    if (choice === 'save') return save()
    return true
  }, [save])

  const newProject = useCallback(async () => {
    if (await settleUnsaved()) await run(commands.newDocument())
  }, [run, settleUnsaved])
  const open = useCallback(async () => {
    if (await settleUnsaved()) await run(commands.openDocument())
  }, [run, settleUnsaved])
  const openRequested = useCallback(async () => {
    if (await settleUnsaved()) await run(commands.openRequestedDocument())
  }, [run, settleUnsaved])
  const relink = useCallback((folderId: number) => run(commands.relinkFolder(folderId)), [run])
  const rename = useCallback((name: string) => run(commands.renameDocument(name)), [run])

  // Native menu (macOS) and files handed over by the OS.
  useEffect(() => {
    const actions: Record<Action, () => unknown> = {
      new: newProject,
      open,
      save,
      saveAs,
      openRequested,
    }
    const unlisten = events.menuAction.listen(({ payload }) => void actions[payload.action]())
    return () => void unlisten.then((fn) => fn())
  }, [newProject, open, save, saveAs, openRequested])

  // Ctrl+N / Ctrl+O / Ctrl+S / Ctrl+Shift+S where there is no native menu.
  useEffect(() => {
    if (isMac) return
    const onKey = (e: KeyboardEvent) => {
      if (!e.ctrlKey || e.altKey || isTyping(e.target)) return
      const key = e.key.toLowerCase()
      const action =
        key === 'n' ? newProject : key === 'o' ? open : key === 's' ? (e.shiftKey ? saveAs : save) : null
      if (action) {
        e.preventDefault()
        void action()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [newProject, open, save, saveAs])

  return {
    document,
    error,
    dismissError: () => setError(null),
    question,
    newProject,
    open,
    save,
    saveAs,
    relink,
    rename,
  }
}

export type ProjectApi = ReturnType<typeof useProject>
