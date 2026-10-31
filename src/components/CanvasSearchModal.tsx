import { useEffect, useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from 'react'
import type { CanvasMeta } from '../../shared/types'
import { cn } from '@/lib/utils'
import { timeAgo } from '../lib/time'
import { FrameThumb } from './FramePreview'
import { Modal } from './ui/modal'
import { Pressable } from './ui/pressable'
import { SearchField } from './ui/search-field'

/**
 * The rail's palette: one field over the page, with runnable commands above
 * the matching canvases. Arrow keys move the highlight, Enter runs or opens,
 * Escape closes — the keyboard path ⌘K people expect. Commands are the
 * dashboard's own verbs, handed in by the rail so this stays a list of
 * entries rather than a second router.
 */

const MAX_RESULTS = 8
const MAX_COMMANDS = 6

export interface PaletteCommand {
  id: string
  label: string
  /** extra words the query can match, beyond the label */
  keywords?: string
  /** a short right-aligned note — a shortcut, or where it goes */
  hint?: string
  icon?: ReactNode
  run: () => void | Promise<void>
}

type Entry =
  | { kind: 'command'; command: PaletteCommand }
  | { kind: 'canvas'; canvas: CanvasMeta }

function matches(label: string, keywords: string | undefined, q: string) {
  return label.toLowerCase().includes(q) || (keywords ?? '').toLowerCase().includes(q)
}

export function CanvasSearchModal({
  canvases,
  commands = [],
  onOpen,
  onClose,
}: {
  canvases: CanvasMeta[]
  commands?: PaletteCommand[]
  onOpen: (canvas: CanvasMeta) => void
  onClose: () => void
}) {
  const [query, setQuery] = useState('')
  const [cursor, setCursor] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const entries = useMemo<Entry[]>(() => {
    const q = query.trim().toLowerCase()
    const found = (q ? commands.filter((c) => matches(c.label, c.keywords, q)) : commands).slice(0, MAX_COMMANDS)
    const canvasesFound = (q ? canvases.filter((c) => c.name.toLowerCase().includes(q)) : canvases).slice(0, MAX_RESULTS)
    return [
      ...found.map((command): Entry => ({ kind: 'command', command })),
      ...canvasesFound.map((canvas): Entry => ({ kind: 'canvas', canvas })),
    ]
  }, [canvases, commands, query])

  /* the highlight follows the list: a shorter result set cannot leave it past the end */
  const active = Math.min(cursor, Math.max(entries.length - 1, 0))

  /* a keyboard walk past the fold brings the row into view */
  useEffect(() => {
    listRef.current?.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView({ block: 'nearest' })
  }, [active, entries.length])

  function choose(entry: Entry) {
    if (entry.kind === 'canvas') return onOpen(entry.canvas)
    onClose()
    try {
      const outcome = entry.command.run()
      if (outcome instanceof Promise) outcome.catch(console.error)
    } catch (err) {
      console.error(err)
    }
  }

  function onKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setCursor((i) => Math.min(i + 1, entries.length - 1))
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setCursor((i) => Math.max(i - 1, 0))
    } else if (e.key === 'Enter') {
      const hit = entries[active]
      if (hit) choose(hit)
    }
  }

  const label = 'Search canvases or run a command'

  return (
    <Modal size="lg" onClose={onClose} className="p-0 sm:p-0" aria-label={label}>
      <SearchField
        ref={inputRef}
        variant="bar"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value)
          setCursor(0)
        }}
        onKeyDown={onKeyDown}
        placeholder={label}
        aria-label={label}
        inputClassName="text-copy md:text-copy"
      />
      <div ref={listRef} className="max-h-[60vh] overflow-y-auto p-2" role="listbox" aria-label="Results">
        {entries.length === 0 ? (
          <p className="px-3 py-6 text-center text-body text-ink-faint">No canvas or command matches “{query.trim()}”.</p>
        ) : (
          entries.map((entry, i) => {
            const sectionStart = i === 0 || entries[i - 1]!.kind !== entry.kind
            return (
              <div key={entry.kind === 'command' ? `cmd:${entry.command.id}` : `canvas:${entry.canvas.id}`}>
                {sectionStart && (
                  <p className="px-2.5 pb-1 pt-2 text-micro font-medium uppercase tracking-[0.08em] text-ink-faint">
                    {entry.kind === 'command' ? 'Commands' : 'Canvases'}
                  </p>
                )}
                {entry.kind === 'command' ? (
                  <Pressable
                    role="option"
                    aria-selected={i === active}
                    focus="none"
                    data-active={i === active || undefined}
                    onMouseEnter={() => setCursor(i)}
                    onClick={() => choose(entry)}
                    className={cn(
                      'flex items-center gap-3 rounded-pill px-2.5 py-2 text-left transition-colors',
                      i === active ? 'bg-paper-deep' : 'hover:bg-paper',
                    )}
                  >
                    <span className="grid size-5 flex-none place-items-center text-ink-soft">{entry.command.icon}</span>
                    <span className="min-w-0 flex-1 truncate text-body font-medium text-ink">{entry.command.label}</span>
                    {entry.command.hint && (
                      <span className="flex-none text-caption text-ink-faint">{entry.command.hint}</span>
                    )}
                  </Pressable>
                ) : (
                  <Pressable
                    role="option"
                    aria-selected={i === active}
                    focus="none"
                    data-active={i === active || undefined}
                    onMouseEnter={() => setCursor(i)}
                    onClick={() => choose(entry)}
                    className={cn(
                      'flex items-center gap-3 rounded-pill px-2.5 py-2 transition-colors',
                      i === active ? 'bg-paper-deep' : 'hover:bg-paper',
                    )}
                  >
                    <FrameThumb frameId={entry.canvas.previewFrameId} />
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-body font-medium text-ink">{entry.canvas.name}</span>
                      <span className="mt-px block text-caption text-ink-faint">
                        {entry.canvas.frameCount} frame{entry.canvas.frameCount === 1 ? '' : 's'} · edited{' '}
                        {timeAgo(entry.canvas.updatedAt)}
                        {entry.canvas.shared ? ' · shared with you' : ''}
                      </span>
                    </span>
                  </Pressable>
                )}
              </div>
            )
          })
        )}
      </div>
    </Modal>
  )
}
