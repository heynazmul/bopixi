import { useEffect, useState } from 'react'
import { isTypingTarget, MOD_KEY } from '../lib/keys'
import { Button } from './ui/button'
import { Kbd } from './ui/kbd'
import { Modal, ModalActions, ModalLede, ModalTitle } from './ui/modal'

/* The shortcut sheet: one page listing what the canvas and the dashboard
   answer to, opened by ? or from the Help menu and the command palette.
   A window event carries the "open" request so a menu row deep inside the
   canvas chrome can summon a modal that App mounts once for every route. */

export const SHORTCUTS_EVENT = 'bopixi:shortcuts'

export function openShortcuts() {
  window.dispatchEvent(new Event(SHORTCUTS_EVENT))
}

type Row = { keys: string[]; label: string }
type Group = { title: string; rows: Row[] }

/** Mac shows ⌘; elsewhere the same chords are Ctrl. */
const MOD = MOD_KEY === '⌘' ? '⌘' : 'Ctrl+'

const GROUPS: Group[] = [
  {
    title: 'Everywhere',
    rows: [
      { keys: [`${MOD}K`], label: 'Search canvases or run a command' },
      { keys: ['?'], label: 'Keyboard shortcuts' },
      { keys: [`${MOD}.`], label: 'Hide the interface' },
    ],
  },
  {
    title: 'Canvas tools',
    rows: [
      { keys: ['V'], label: 'Select' },
      { keys: ['H'], label: 'Hand' },
      { keys: ['F'], label: 'Frame' },
      { keys: ['R'], label: 'Rectangle' },
      { keys: ['O'], label: 'Circle' },
      { keys: ['P'], label: 'Vector path' },
      { keys: ['T'], label: 'Text' },
      { keys: ['C'], label: 'Comment' },
    ],
  },
  {
    title: 'Edit',
    rows: [
      { keys: [`${MOD}Z`], label: 'Undo' },
      { keys: [`${MOD}⇧Z`], label: 'Redo' },
      { keys: [`${MOD}C`], label: 'Copy' },
      { keys: [`${MOD}V`], label: 'Paste' },
      { keys: [`${MOD}D`], label: 'Duplicate' },
      { keys: [`${MOD}⇧H`], label: 'Hide or show the selection' },
      { keys: [`${MOD}⇧L`], label: 'Lock or unlock the selection' },
      { keys: ['⌫'], label: 'Delete the selection' },
      { keys: ['Esc'], label: 'Deselect, or return to Select' },
    ],
  },
  {
    title: 'View',
    rows: [
      { keys: ['Space'], label: 'Pan while held' },
      { keys: [`${MOD}+`], label: 'Zoom in' },
      { keys: [`${MOD}−`], label: 'Zoom out' },
      { keys: [`${MOD}0`], label: 'Actual size' },
      { keys: ['⇧1'], label: 'Fit everything' },
      { keys: ['⇧2'], label: 'Fit the selection' },
      { keys: ['⇧R'], label: 'Rulers' },
    ],
  },
  {
    title: 'Agents',
    rows: [
      { keys: [`${MOD}K`], label: 'Ask about what is selected' },
      { keys: ['Enter'], label: 'Send a message' },
      { keys: ['⇧Enter'], label: 'Start a new line' },
      { keys: [`${MOD}Enter`], label: 'Send a comment' },
    ],
  },
]

export function ShortcutsModal({ onClose }: { onClose: () => void }) {
  return (
    <Modal size="lg" onClose={onClose} aria-label="Keyboard shortcuts">
      <ModalTitle>Keyboard shortcuts</ModalTitle>
      <ModalLede>The canvas and the dashboard at a glance. Outside macOS, Ctrl stands in for ⌘.</ModalLede>
      <div className="mt-5 grid gap-x-8 gap-y-6 sm:grid-cols-2">
        {GROUPS.map((group) => (
          <section key={group.title}>
            <h3 className="text-micro font-medium uppercase leading-none tracking-[0.08em] text-ink-faint">
              {group.title}
            </h3>
            <dl className="mt-2.5 space-y-1.5">
              {group.rows.map((row) => (
                <div key={row.label} className="flex items-center justify-between gap-3">
                  <dt className="min-w-0 text-body text-ink-soft">{row.label}</dt>
                  <dd className="flex flex-none items-center gap-1">
                    {row.keys.map((key) => (
                      <Kbd key={key}>{key}</Kbd>
                    ))}
                  </dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
      </div>
      <ModalActions>
        <Button variant="ghost" onClick={onClose}>
          Close
        </Button>
      </ModalActions>
    </Modal>
  )
}

/** Mounted once by App: listens for ? (outside fields and open dialogs) and
 *  for the event the Help menu and palette dispatch. */
export function ShortcutsLauncher() {
  const [open, setOpen] = useState(false)
  useEffect(() => {
    function show() {
      setOpen(true)
    }
    function onKey(e: KeyboardEvent) {
      if (e.key !== '?' || e.metaKey || e.ctrlKey || e.altKey) return
      if (isTypingTarget(e.target)) return
      /* a modal, the palette or a sheet already owns the keyboard */
      if (document.querySelector('[role="dialog"]')) return
      e.preventDefault()
      setOpen(true)
    }
    window.addEventListener(SHORTCUTS_EVENT, show)
    window.addEventListener('keydown', onKey)
    return () => {
      window.removeEventListener(SHORTCUTS_EVENT, show)
      window.removeEventListener('keydown', onKey)
    }
  }, [])
  return open ? <ShortcutsModal onClose={() => setOpen(false)} /> : null
}
