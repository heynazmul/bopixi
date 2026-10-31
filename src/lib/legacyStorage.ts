/* Keys written before the rename carried a `doop` prefix (`doop:active`,
   `doop-theme`, ...). Copy each one to its `bopixi` spelling on boot when
   the new key is still absent, so a returning browser keeps its ui state.
   Legacy keys stay in place: a rollback or an older tab in the same origin
   still reads them. */

const LEGACY_PREFIXES = ['doop:', 'doop-'] as const

function migrate(storage: Storage): void {
  const moves: [string, string][] = []
  for (let i = 0; i < storage.length; i += 1) {
    const key = storage.key(i)
    if (!key) continue
    const prefix = LEGACY_PREFIXES.find((p) => key.startsWith(p))
    if (!prefix) continue
    const next = `bopixi${key.slice('doop'.length)}`
    if (storage.getItem(next) === null) moves.push([key, next])
  }
  for (const [from, to] of moves) {
    const value = storage.getItem(from)
    if (value !== null) storage.setItem(to, value)
  }
}

export function migrateLegacyStorage(): void {
  try {
    migrate(localStorage)
  } catch {
    /* storage can be unavailable (blocked cookies, private mode) */
  }
  try {
    migrate(sessionStorage)
  } catch {
    /* as above */
  }
}

migrateLegacyStorage()
