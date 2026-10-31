import type { Brand } from '@/components/ui/brand-mark'

/**
 * Every agent that can work on a Bopixi canvas, in the order the connect
 * picker lists them. Bopixi itself is a streamable-HTTP MCP server, so any
 * MCP client qualifies; the catalogue exists so the picker reads like a
 * roster instead of a blank "bring your own client" box — the same agents
 * Recto ships. `native` marks the ones whose hand-off Bopixi can walk through
 * exactly (installer deep links, OAuth toggle, app-specific commands); the
 * rest share one honest, client-agnostic MCP recipe.
 *
 * `match` tokens name the spellings an outside agent reports itself under
 * (`agent_name` on an MCP call, the presence entry it creates). Presence and
 * task history turn those into the picker's live Online / last-worked marks.
 */

export type AgentCategory = 'terminal' | 'editor' | 'chat' | 'other'

/** the agents with a bespoke connect walkthrough */
export type NativeAgentId = 'claude-code' | 'claude' | 'chatgpt' | 'codex' | 'cursor' | 'mcp'

/** Every agent has a stable id; the built-ins are listed above, and a person
 *  can add their own CLI agents, whose ids are generated at add time. */
export type AgentId = string

export interface AgentEntry {
  id: AgentId
  name: string
  blurb: string
  category: AgentCategory
  /** the brand mark to draw, when Bopixi knows the glyph */
  brand?: Brand
  /** otherwise the two-letter mark inside a tinted square */
  monogram?: string
  /** the tint for that square, and nothing else */
  color: string
  /** spellings an outside agent uses for itself, lower-case */
  match: string[]
  native?: boolean
  /** the command that starts a CLI agent the person added */
  command?: string
  custom?: boolean
}

export const AGENT_CATALOG: AgentEntry[] = [
  {
    id: 'claude-code',
    name: 'Claude Code',
    blurb: 'Connect from your terminal',
    category: 'terminal',
    brand: 'claude',
    color: '#D97757',
    match: ['claude code', 'claude-code', 'claudecode'],
    native: true,
  },
  {
    id: 'claude',
    name: 'Claude',
    blurb: 'Add Bopixi to Claude',
    category: 'chat',
    brand: 'claude',
    color: '#D97757',
    match: ['claude'],
    native: true,
  },
  {
    id: 'chatgpt',
    name: 'ChatGPT',
    blurb: 'Connect in your browser',
    category: 'chat',
    brand: 'openai',
    color: '#10A37F',
    match: ['chatgpt', 'gpt'],
    native: true,
  },
  {
    id: 'codex',
    name: 'Codex',
    blurb: 'ChatGPT desktop and Codex CLI',
    category: 'terminal',
    brand: 'openai',
    color: '#10A37F',
    match: ['codex'],
    native: true,
  },
  {
    id: 'cursor',
    name: 'Cursor',
    blurb: 'Add Bopixi to Cursor',
    category: 'editor',
    brand: 'cursor',
    color: '#111827',
    match: ['cursor'],
    native: true,
  },
  {
    id: 'mcp',
    name: 'Other MCP client',
    blurb: 'Use a manual connection',
    category: 'other',
    monogram: '{}',
    color: '#475569',
    match: ['mcp'],
    native: true,
  },
  {
    id: 'gemini',
    name: 'Gemini CLI',
    blurb: "Google's terminal agent",
    category: 'terminal',
    brand: 'gemini',
    color: '#4285F4',
    match: ['gemini', 'gemini cli'],
  },
  {
    id: 'antigravity',
    name: 'Antigravity',
    blurb: "Google's agent-first editor",
    category: 'editor',
    monogram: 'An',
    color: '#4285F4',
    match: ['antigravity'],
  },
  {
    id: 'cline',
    name: 'Cline',
    blurb: 'Agent inside VS Code',
    category: 'editor',
    monogram: 'Cl',
    color: '#0F766E',
    match: ['cline'],
  },
  {
    id: 'kilo',
    name: 'Kilo Code',
    blurb: 'Open-source editor agent',
    category: 'editor',
    monogram: 'Kl',
    color: '#9333EA',
    match: ['kilo code', 'kilo'],
  },
  {
    id: 'copilot',
    name: 'GitHub Copilot',
    blurb: 'Agent mode in your editor',
    category: 'editor',
    brand: 'github',
    color: '#1F2328',
    match: ['github copilot', 'copilot'],
  },
  {
    id: 'opencode',
    name: 'OpenCode',
    blurb: 'Open-source terminal agent',
    category: 'terminal',
    monogram: 'OC',
    color: '#6366F1',
    match: ['opencode', 'open code'],
  },
  {
    id: 'aider',
    name: 'Aider',
    blurb: 'Pair-programming in your terminal',
    category: 'terminal',
    monogram: 'Ai',
    color: '#16A34A',
    match: ['aider'],
  },
  {
    id: 'goose',
    name: 'Goose',
    blurb: 'Block’s open agent',
    category: 'terminal',
    monogram: 'Go',
    color: '#0891B2',
    match: ['goose'],
  },
  {
    id: 'qwen',
    name: 'Qwen Code',
    blurb: "Alibaba's terminal agent",
    category: 'terminal',
    monogram: 'Qw',
    color: '#6D28D9',
    match: ['qwen', 'qwen code'],
  },
  {
    id: 'crush',
    name: 'Crush',
    blurb: "Charm's terminal agent",
    category: 'terminal',
    monogram: 'Cr',
    color: '#DB2777',
    match: ['crush'],
  },
  {
    id: 'droid',
    name: 'Droid',
    blurb: "Factory's terminal agent",
    category: 'terminal',
    monogram: 'Dr',
    color: '#0284C7',
    match: ['droid'],
  },
  {
    id: 'amp',
    name: 'Amp',
    blurb: "Sourcegraph's coding agent",
    category: 'terminal',
    monogram: 'Am',
    color: '#EA580C',
    match: ['amp'],
  },
  {
    id: 'kimi',
    name: 'Kimi CLI',
    blurb: "Moonshot's terminal agent",
    category: 'terminal',
    monogram: 'Ki',
    color: '#1E293B',
    match: ['kimi', 'kimi cli'],
  },
  {
    id: 'pi',
    name: 'Pi',
    blurb: 'A terminal agent',
    category: 'terminal',
    monogram: 'π',
    color: '#0D9488',
    match: ['pi', 'pi cli'],
  },
  {
    id: 'junie',
    name: 'Junie',
    blurb: "JetBrains' coding agent",
    category: 'terminal',
    monogram: 'Ju',
    color: '#2563EB',
    match: ['junie'],
  },
  {
    id: 'grok',
    name: 'Grok CLI',
    blurb: "xAI's terminal agent",
    category: 'terminal',
    monogram: 'Gr',
    color: '#111827',
    match: ['grok', 'grok cli'],
  },
]

export const AGENT_IDS = AGENT_CATALOG.map((a) => a.id)

const byId = new Map(AGENT_CATALOG.map((a) => [a.id, a]))

export function agentEntry(id: AgentId): AgentEntry {
  return byId.get(id)!
}

export function isNativeAgent(id: AgentId): id is NativeAgentId {
  return byId.get(id)?.native === true
}

export const AGENT_CATEGORIES: { id: 'all' | AgentCategory; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'terminal', label: 'Terminal' },
  { id: 'editor', label: 'Editor' },
  { id: 'chat', label: 'Chat' },
  { id: 'other', label: 'Other' },
]

/* Most specific first: "Claude Code" is not the Claude app, and "ChatGPT
   Codex" (if anyone ever names it that) is Codex, not ChatGPT. */
const NAME_RULES: [AgentId, RegExp][] = [
  ['claude-code', /\bclaude[ -]?code\b/i],
  ['codex', /codex/i],
  ['chatgpt', /\bchatgpt\b|\bgpt\b|\bopenai\b/i],
  ['claude', /claude/i],
  ['antigravity', /antigravity/i],
  ['kilo', /kilo/i],
  ['copilot', /copilot/i],
  ['opencode', /opencode|open code/i],
  ['cline', /\bcline\b/i],
  ['aider', /aider/i],
  ['goose', /\bgoose\b/i],
  ['qwen', /qwen/i],
  ['crush', /\bcrush\b/i],
  ['droid', /droid/i],
  ['amp', /\bamp\b/i],
  ['kimi', /kimi/i],
  ['junie', /junie/i],
  ['grok', /grok/i],
  ['gemini', /gemini/i],
  ['pi', /\bpi\b/i],
  ['mcp', /\bmcp\b/i],
]

/** The catalogue entry an outside agent's self-declared name reads as. Custom
 *  agents are checked after the built-ins: a person who names their agent
 *  "Aider" still gets Aider's own card, but an agent Bopixi cannot place is
 *  matched to the CLI agent that was added under that exact name. */
export function agentIdOfName(name: string | undefined): AgentId | undefined {
  if (!name) return undefined
  const builtIn = NAME_RULES.find(([, pattern]) => pattern.test(name))?.[0]
  if (builtIn) return builtIn
  const lower = name.trim().toLowerCase()
  return readCustomAgents().find((record) => record.name.toLowerCase() === lower)?.id
}

/* ---------- CLI agents the person adds themselves ---------- */

/* The command is only ever displayed and copied — Bopixi never runs a command a
   page supplies — so what is stored is the text the person will paste into
   their own terminal, kept short and whole. */
const CUSTOM_AGENTS_KEY = 'bopixi:customAgents'
const CUSTOM_COLORS = ['#6366F1', '#0891B2', '#DB2777', '#EA580C', '#16A34A', '#7C3AED', '#0284C7', '#B45309']

export interface CustomAgentRecord {
  id: string
  name: string
  command: string
}

function readCustomAgents(): CustomAgentRecord[] {
  try {
    const raw = localStorage.getItem(CUSTOM_AGENTS_KEY)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter(
        (row): row is CustomAgentRecord =>
          !!row &&
          typeof row === 'object' &&
          typeof (row as CustomAgentRecord).id === 'string' &&
          typeof (row as CustomAgentRecord).name === 'string' &&
          typeof (row as CustomAgentRecord).command === 'string',
      )
      .slice(0, 50)
  } catch {
    return []
  }
}

function writeCustomAgents(records: CustomAgentRecord[]) {
  try {
    localStorage.setItem(CUSTOM_AGENTS_KEY, JSON.stringify(records))
  } catch {
    /* blocked storage only means the agent lasts for this visit */
  }
}

function colorFor(name: string) {
  let hash = 0
  for (const char of name) hash = (hash + (char.codePointAt(0) ?? 0)) % CUSTOM_COLORS.length
  return CUSTOM_COLORS[hash] ?? '#6366F1'
}

function customEntry(record: CustomAgentRecord): AgentEntry {
  return {
    id: record.id,
    name: record.name,
    blurb: 'Your CLI agent',
    category: 'terminal',
    monogram: record.name.slice(0, 2),
    color: colorFor(record.name),
    match: [record.name.toLowerCase()],
    command: record.command,
    custom: true,
  }
}

/** The CLI agents this browser remembers, in the order they were added. */
export function loadCustomAgents(): AgentEntry[] {
  return readCustomAgents().map(customEntry)
}

/** Add a named CLI agent and its run command. Returns the new entry, or
 *  undefined when the name or command is blank. */
export function addCustomAgent(name: string, command: string): AgentEntry | undefined {
  const cleanName = name.trim().slice(0, 40)
  const cleanCommand = command.trim().slice(0, 300)
  if (!cleanName || !cleanCommand) return undefined
  const records = readCustomAgents()
  const slug =
    cleanName
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'agent'
  let id = `custom-${slug}`
  if (records.some((record) => record.id === id)) id = `${id}-${Date.now().toString(36)}`
  const record = { id, name: cleanName, command: cleanCommand }
  writeCustomAgents([...records, record])
  return customEntry(record)
}

export function removeCustomAgent(id: AgentId) {
  writeCustomAgents(readCustomAgents().filter((record) => record.id !== id))
}

/** A built-in or added agent by id. */
export function findAgent(id: AgentId): AgentEntry | undefined {
  const builtIn = byId.get(id)
  if (builtIn) return builtIn
  const record = readCustomAgents().find((row) => row.id === id)
  return record ? customEntry(record) : undefined
}

/* The picker starts where the last connection did, like Recto's agent
   switcher. Stored locally: which agent a person used is their business,
   not the canvas's. */
const LAST_AGENT_KEY = 'bopixi:connectAgent'

export function lastAgentId(): AgentId | undefined {
  try {
    const raw = localStorage.getItem(LAST_AGENT_KEY)
    if (!raw) return undefined
    if (AGENT_IDS.includes(raw)) return raw
    return readCustomAgents().some((record) => record.id === raw) ? raw : undefined
  } catch {
    return undefined
  }
}

export function rememberAgent(id: AgentId) {
  try {
    localStorage.setItem(LAST_AGENT_KEY, id)
  } catch {
    /* blocked storage only means the next visit starts from Claude Code */
  }
}
