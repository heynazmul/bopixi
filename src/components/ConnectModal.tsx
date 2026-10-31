import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { navigate } from '../App'
import { useStore } from '../lib/store'
import type { AgentTask } from '../../shared/types'
import { roleByAgentName } from '../../shared/agents'
import { posthog } from '../lib/posthog'
import { canUseChatGPTPublicEndpoint, CHATGPT_CONNECTION_GUIDE, CHATGPT_PLUGINS_URL } from '../lib/chatgptConnection'
import { ClaudeDesktopInstall } from './ClaudeDesktopInstall'
import { canAddToClaudeDesktop, claudeConnectorInstallUrl } from '../lib/claudeConnection'
import { CursorInstall } from './CursorInstall'
import { CodexInstall } from './CodexInstall'
import { Button } from './ui/button'
import { CodeBlock } from './ui/code-block'
import { Disclosure } from './ui/collapsible'
import { Dot } from './ui/dot'
import { Field } from './ui/field'
import { Input } from './ui/input'
import { CheckIcon, ChevronLeftIcon, PlusIcon, PuzzleIcon, TrashIcon } from './ui/icons'
import { Modal, ModalActions, ModalLede, ModalSpacer, ModalTitle } from './ui/modal'
import { SearchField } from './ui/search-field'
import { ToggleChipGroup, ToggleChipItem } from './ui/toggle-chip'
import { BrandMark } from './ui/brand-mark'
import { barPrimaryBtn } from './TopBar'
import { cn } from '@/lib/utils'
import { timeAgo } from '../lib/time'
import {
  AGENT_CATALOG,
  AGENT_CATEGORIES,
  addCustomAgent,
  agentIdOfName,
  findAgent,
  isNativeAgent,
  lastAgentId,
  loadCustomAgents,
  rememberAgent,
  removeCustomAgent,
  type AgentCategory,
  type AgentEntry,
  type AgentId,
  type NativeAgentId,
} from '../lib/agentCatalog'

/**
 * "Connect AI" is about agents that join the canvas from outside: Claude Code,
 * Codex, Cursor, the Claude or ChatGPT apps, or any MCP client. The picker is a
 * roster — searchable, grouped by where the agent lives, with live online and
 * last-worked marks for the ones that have been here — and each agent gets its
 * own walkthrough. Agents Bopixi knows intimately get their installer or deep
 * link; the rest share one honest streamable-HTTP recipe. Running the built-in
 * Bopixi Agent on your own ChatGPT or Claude plan is an account setting and lives
 * in Settings, next to the prompt bar's allowance meter — not here.
 */

type View = 'agents' | AgentId
type ClaudeConnection = 'desktop' | 'web'
type CategoryFilter = 'all' | AgentCategory

/** `canvasId` is optional: opened from the home dashboard there is no canvas to
 *  suggest a prompt for, and no presence connection to watch for an arrival. */
export function ConnectModal({
  canvasId,
  onClose,
  initialAgent,
  initialAdd = false,
  showAgentStatus = true,
}: {
  canvasId?: string
  onClose: () => void
  initialAgent?: AgentId | 'add'
  initialAdd?: boolean
  /** Standalone setup can suggest a canvas without subscribing to its presence. */
  showAgentStatus?: boolean
}) {
  const isAdd = initialAdd || initialAgent === 'add'
  const [view, setView] = useState<View>(isAdd || !initialAgent ? 'agents' : initialAgent)
  return (
    <Modal size="lg" className="flex flex-col overflow-hidden p-0 sm:p-0" onClose={onClose}>
      {view === 'agents' ? (
        <AgentPicker onPick={setView} initialAdd={isAdd} />
      ) : (
        <AgentSteps
          agent={view}
          canvasId={canvasId}
          showAgentStatus={showAgentStatus}
          onBack={() => setView('agents')}
          onClose={onClose}
        />
      )}
    </Modal>
  )
}

/* ---------- pick the agent, then its three steps ---------- */

function AgentPicker({ onPick, initialAdd = false }: { onPick: (agent: AgentId) => void; initialAdd?: boolean }) {
  /* Start where the last connection did, like a tool switcher. The last pick
     can be filtered out by a search; then the first visible one stands in. */
  const [agent, setAgent] = useState<AgentId>(() => lastAgentId() ?? 'claude-code')
  const [query, setQuery] = useState('')
  const [category, setCategory] = useState<CategoryFilter>('all')
  const [custom, setCustom] = useState<AgentEntry[]>(() => loadCustomAgents())
  const [adding, setAdding] = useState(initialAdd)
  const { online, worked } = useAgentStatuses()

  const roster = useMemo(() => [...AGENT_CATALOG, ...custom], [custom])

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase()
    return roster.filter((entry) => {
      if (category !== 'all' && entry.category !== category) return false
      if (!needle) return true
      return [entry.name, entry.blurb, entry.id, entry.category, entry.command ?? ''].some((field) =>
        field.toLowerCase().includes(needle),
      )
    })
  }, [roster, query, category])

  const selected = shown.some((entry) => entry.id === agent) ? agent : shown[0]?.id

  const handleAdd = (name: string, command: string) => {
    const entry = addCustomAgent(name, command)
    if (!entry) return
    setCustom(loadCustomAgents())
    setAdding(false)
    setAgent(entry.id)
    rememberAgent(entry.id)
    posthog.capture('connect_ai_agent_added', { agent: entry.id })
    onPick(entry.id)
  }

  if (adding) return <AddAgentForm onCancel={() => setAdding(false)} onAdd={handleAdd} />

  return (
    <>
      <div className={body}>
        <div className="flex items-start justify-between gap-3">
          <div>
            <ModalTitle>Choose your agent</ModalTitle>
            <ModalLede>
              Let an agent you already use work on this canvas via MCP. Choose yours to see its connection instructions.
            </ModalLede>
          </div>
          <Button
            size="sm"
            variant="ghost"
            className="mt-1 shrink-0 gap-1.5 border border-line hover:border-ink-faint"
            onClick={() => setAdding(true)}
          >
            <PlusIcon width={14} height={14} />
            <span>Add CLI agent</span>
          </Button>
        </div>

        <SearchField
          className="mt-5 w-full"
          placeholder="Search agents"
          aria-label="Search agents"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />

        <ToggleChipGroup
          className="mt-3"
          value={category}
          onValueChange={(next) => setCategory(next as CategoryFilter)}
          aria-label="Agent type"
        >
          {AGENT_CATEGORIES.map(({ id, label }) => (
            <ToggleChipItem key={id} size="sm" value={id}>
              {label}
            </ToggleChipItem>
          ))}
        </ToggleChipGroup>

        <div className="mt-4 grid gap-3 sm:grid-cols-2">
          {shown.map((entry) => (
            <ChoiceCard
              key={entry.id}
              selected={selected === entry.id}
              onClick={() => setAgent(entry.id)}
              icon={<AgentMark entry={entry} size={20} />}
              title={entry.name}
              blurb={entry.blurb}
              status={statusOf(entry.id, online, worked)}
            />
          ))}
          <AddAgentCard onClick={() => setAdding(true)} />
        </div>

        {shown.length === 0 && (
          <p className="mt-6 text-copy leading-relaxed text-ink-soft">
            No agent matches “{query.trim()}”. Try another name or category.
          </p>
        )}
      </div>

      <ModalActions className={footer}>
        <span className="mr-auto self-center text-xs text-ink-faint">
          {shown.length} of {roster.length} agents
        </span>
        <Button
          variant="primary"
          className={barPrimaryBtn}
          disabled={!selected}
          onClick={() => {
            if (!selected) return
            rememberAgent(selected)
            posthog.capture('connect_ai_agent_chosen', { agent: selected })
            onPick(selected)
          }}
        >
          Connect
        </Button>
      </ModalActions>
    </>
  )
}

/** The "+" tile at the end of the roster: name any CLI agent that speaks MCP,
 *  give the command that starts it, and it joins the picker. */
function AddAgentCard({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex items-center gap-3 rounded-card border border-dashed border-line bg-surface/50 py-3.5 pl-3.5 pr-4 text-left text-ink transition-colors hover:border-ink-faint"
    >
      <span className="grid size-9 shrink-0 place-items-center rounded-[9px] border border-dashed border-line text-ink-faint">
        <PlusIcon width={16} height={16} />
      </span>
      <span className="block min-w-0">
        <span className="block text-copy font-semibold">Add a CLI agent</span>
        <span className="mt-0.5 block text-xs leading-[1.5] text-ink-soft">
          Name it and give its run command
        </span>
      </span>
    </button>
  )
}

function AddAgentForm({ onCancel, onAdd }: { onCancel: () => void; onAdd: (name: string, command: string) => void }) {
  const [name, setName] = useState('')
  const [command, setCommand] = useState('')
  const ready = name.trim().length > 0 && command.trim().length > 0
  const submit = () => {
    if (ready) onAdd(name, command)
  }

  const handleNameChange = (val: string) => {
    setName(val)
    if (!command || command === 'claude' || command === 'agy' || command === 'codex' || command === 'aider') {
      const lower = val.trim().toLowerCase()
      if (lower === 'agy' || lower === 'antigravity') setCommand('agy')
      else if (lower === 'claude' || lower === 'claude code') setCommand('claude')
      else if (lower === 'codex') setCommand('codex')
      else if (lower === 'aider') setCommand('aider')
    }
  }

  const PRESETS = [
    { label: 'Claude', name: 'Claude', command: 'claude' },
    { label: 'Antigravity (agy)', name: 'Antigravity', command: 'agy' },
    { label: 'Codex', name: 'Codex', command: 'codex' },
    { label: 'Aider', name: 'Aider', command: 'aider' },
  ]

  return (
    <>
      <div className={body}>
        <ModalTitle>Add a CLI agent</ModalTitle>
        <ModalLede>
          Name the agent and the command that starts it (e.g. <code>claude</code>, <code>agy</code>). Bopixi connects over MCP, so it can design on your canvases.
        </ModalLede>

        <div className="mt-4 flex flex-wrap items-center gap-1.5">
          <span className="text-xs text-ink-faint">Presets:</span>
          {PRESETS.map((p) => (
            <button
              key={p.label}
              type="button"
              className="rounded-full border border-line bg-surface/60 px-2.5 py-0.5 text-xs text-ink-soft transition-colors hover:border-ink-faint hover:text-ink"
              onClick={() => {
                setName(p.name)
                setCommand(p.command)
              }}
            >
              {p.label}
            </button>
          ))}
        </div>

        <div className="mt-4 space-y-4">
          <Field label="Agent name" labelVariant="form" htmlFor="custom-agent-name">
            <Input
              id="custom-agent-name"
              autoFocus
              maxLength={40}
              placeholder="e.g. Claude, agy, Aider"
              value={name}
              onChange={(event) => handleNameChange(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') submit()
              }}
            />
          </Field>
          <Field
            label="Run command"
            labelVariant="form"
            htmlFor="custom-agent-command"
            hint="The command you run to start it in a terminal (e.g. claude, agy, aider)."
          >
            <Input
              id="custom-agent-command"
              className="font-mono"
              maxLength={300}
              placeholder="e.g. claude, agy, aider --model sonnet"
              value={command}
              onChange={(event) => setCommand(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') submit()
              }}
            />
          </Field>
        </div>
      </div>

      <ModalActions className={footer}>
        <ModalSpacer />
        <Button variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button variant="primary" className={barPrimaryBtn} disabled={!ready} onClick={submit}>
          Add agent
        </Button>
      </ModalActions>
    </>
  )
}

/** Live marks for the roster: who is connected to this canvas right now (an
 *  outside agent in presence), and who worked here last. Resident Bopixi roles are
 *  not "connections" — they are always around — so they stay out of both. */
function useAgentStatuses() {
  const presences = useStore((s) => s.presences)
  /* Test harnesses answer only `presences`; a missing task list is simply no
     history yet, not an error. */
  const tasks = useStore((s) => s.tasks) as AgentTask[] | undefined

  return useMemo(() => {
    const online = new Set<AgentId>()
    for (const presence of Object.values(presences)) {
      if (presence.kind !== 'agent' || roleByAgentName(presence.name)) continue
      const id = agentIdOfName(presence.name)
      if (id) online.add(id)
    }
    const worked = new Map<AgentId, number>()
    for (const task of tasks ?? []) {
      const id = agentIdOfName(task.agentName)
      if (id) worked.set(id, Math.max(worked.get(id) ?? 0, task.startedAt))
    }
    return { online, worked }
  }, [presences, tasks])
}

function statusOf(id: AgentId, online: Set<AgentId>, worked: Map<AgentId, number>): ReactNode {
  if (online.has(id)) {
    return (
      <span className="inline-flex items-center gap-1.5 text-micro font-medium text-success-ink">
        <Dot tone="done" size="sm" /> Online now
      </span>
    )
  }
  const at = worked.get(id)
  if (at) return <span className="text-micro text-ink-faint">Last worked {timeAgo(at)}</span>
  return null
}

/** The agent's glyph: a known brand where Bopixi has one, a puzzle piece for a
 *  generic MCP client, otherwise the catalogue's own monogram in its tint. */
function AgentMark({ entry, size }: { entry: AgentEntry; size: number }) {
  if (entry.id === 'mcp') return <PuzzleIcon width={size} height={size} />
  if (entry.brand) return <BrandMark brand={entry.brand} size={size} color={entry.color} />
  return (
    <span
      aria-hidden
      className="grid size-full place-items-center rounded-[6px] font-semibold leading-none"
      style={{
        backgroundColor: `${entry.color}1a`,
        color: entry.color,
        fontSize: Math.max(9, Math.round(size * 0.44)),
      }}
    >
      {entry.monogram ?? entry.name.slice(0, 2)}
    </span>
  )
}

interface Step {
  title: string
  body: ReactNode
  code?: string
  action?: ReactNode
}

function canvasStep(prompt: string): Step {
  return {
    title: 'Point it at this canvas',
    body: prompt
      ? 'Paste this as your first message. Everything else the agent needs comes from the server.'
      : 'Open a canvas and tell the agent to work on it. Everything else it needs comes from the server.',
    code: prompt || undefined,
  }
}

/** Every agent Bopixi has no bespoke hand-off for reads the same recipe: add the
 *  streamable-HTTP server, approve the OAuth tab, name the canvas. Honest beats
 *  an invented installer for a tool we cannot drive. */
function manualSteps(entry: AgentEntry, mcpUrl: string, prompt: string): { lede: string; steps: Step[] } {
  const jsonConfig = JSON.stringify({ mcpServers: { bopixi: { type: 'http', url: mcpUrl } } }, null, 2)
  return {
    lede: `${entry.name} connects over MCP. Add Bopixi once and it designs on your canvases as you.`,
    steps: [
      {
        title: `Add Bopixi to ${entry.name}`,
        body: (
          <>
            Open the MCP settings in {entry.name}, add a remote server named <code>bopixi</code>, and paste the address
            below. Bopixi is a streamable-HTTP server, so there is nothing to install.
          </>
        ),
        code: mcpUrl,
        action: (
          <Disclosure label="Config-file form" className="mt-3 text-label leading-relaxed text-ink-soft">
            <p>If it reads a JSON file instead, add this entry.</p>
            <CodeBlock text={jsonConfig} className="mt-2" />
          </Disclosure>
        ),
      },
      {
        title: 'Approve the sign-in',
        body: 'The first time it connects, a browser tab opens to authorize Bopixi. The agent then works as you, and its edits are attributed to you.',
      },
      canvasStep(prompt),
    ],
  }
}

/** A CLI agent someone added themselves. Bopixi cannot install it or know its
 *  flags, so the walkthrough is: attach the MCP server, start it with the
 *  command that was given, sign in, then hand it the canvas. */
function customSteps(entry: AgentEntry, mcpUrl: string, prompt: string): { lede: string; steps: Step[] } {
  const jsonConfig = JSON.stringify({ mcpServers: { bopixi: { type: 'http', url: mcpUrl } } }, null, 2)
  const cmd = entry.command?.trim().toLowerCase() ?? ''
  const name = entry.name.toLowerCase()
  const isClaude = cmd.startsWith('claude') || name.includes('claude')
  const isAgy = cmd.startsWith('agy') || name.includes('agy') || name.includes('antigravity')

  let setupTitle = `Add Bopixi to ${entry.name}`
  let setupBody: ReactNode = (
    <>
      Add a remote MCP server named <code>bopixi</code> with this address. Bopixi ships its own instructions and
      design tools over that connection, so the agent knows how to work on a canvas.
    </>
  )
  let setupCode = mcpUrl

  if (isClaude) {
    setupTitle = 'Add Bopixi as an MCP server in Claude'
    setupBody = (
      <>
        Run this command once in your terminal. It registers the <code>bopixi</code> MCP server:
      </>
    )
    setupCode = `claude mcp add --transport http bopixi "${mcpUrl}"`
  } else if (isAgy) {
    setupTitle = 'Add Bopixi to Antigravity (agy) MCP config'
    setupBody = (
      <>
        Add Bopixi to <code>~/.gemini/config/mcp_config.json</code> or your project&rsquo;s{' '}
        <code>.gemini/mcp_config.json</code>:
      </>
    )
    setupCode = JSON.stringify({ mcpServers: { bopixi: { url: mcpUrl } } }, null, 2)
  }

  return {
    lede: `${entry.name} runs in your terminal and connects back over MCP. Once it does, it designs here as you.`,
    steps: [
      {
        title: setupTitle,
        body: setupBody,
        code: setupCode,
        action:
          !isClaude && !isAgy ? (
            <Disclosure label="Config-file form" className="mt-3 text-label leading-relaxed text-ink-soft">
              <p>If it reads a JSON file instead, add this entry.</p>
              <CodeBlock text={jsonConfig} className="mt-2" />
            </Disclosure>
          ) : undefined,
      },
      {
        title: `Start ${entry.name}`,
        body: 'Run this in a terminal and leave it open. The agent stays connected to Bopixi while it runs.',
        code: entry.command,
      },
      {
        title: 'Approve the sign-in',
        body: 'The first time it connects, a browser tab opens to authorize Bopixi. The agent then works as you, and its edits are attributed to you.',
      },
      canvasStep(prompt),
    ],
  }
}

function agentSteps(
  agent: NativeAgentId,
  mcpUrl: string,
  prompt: string,
  claudeConnection: ClaudeConnection,
  canvasId?: string,
): { lede: string; steps: Step[] } {
  const jsonConfig = JSON.stringify({ mcpServers: { bopixi: { type: 'http', url: mcpUrl } } }, null, 2)
  const pointAt = canvasStep(prompt)
  switch (agent) {
    case 'claude-code':
      return {
        lede: 'Three steps, about a minute. Claude Code then designs on this canvas as you.',
        steps: [
          {
            title: 'Add Bopixi as an MCP server',
            body: (
              <>
                Run this once in any terminal. It registers the <code>bopixi</code> server for every project.
              </>
            ),
            code: `claude mcp add --transport http bopixi "${mcpUrl}"`,
          },
          {
            title: 'Sign in from Claude Code',
            body: (
              <>
                Inside Claude Code type <code>/mcp</code>, pick <strong>bopixi</strong>, then{' '}
                <strong>Authenticate</strong>. A browser tab opens so you can approve the connection.
              </>
            ),
          },
          pointAt,
        ],
      }
    case 'claude':
      if (claudeConnection === 'desktop') {
        return {
          lede: 'Add Bopixi from the desktop app, confirm in Claude, and start designing together.',
          steps: [
            {
              title: 'Add Bopixi to Claude Desktop',
              body: 'Click Add to Claude in the Bopixi desktop app, then confirm the installation in Claude. Your Bopixi server address is filled in for you.',
              action: <ClaudeDesktopInstall mcpUrl={mcpUrl} />,
            },
            {
              title: 'Connect your Bopixi account',
              body: 'Enable the extension in Claude. Sign in to Bopixi in the browser that opens and approve the connection.',
            },
            {
              ...pointAt,
              body: prompt
                ? 'Start a new Claude chat, enable Bopixi in the tools menu, and send this as your first message.'
                : 'Start a new Claude chat, enable Bopixi in the tools menu, and tell it which canvas to work on.',
            },
          ],
        }
      }
      if (!claudeConnectorInstallUrl(mcpUrl)) {
        return {
          lede: 'Claude’s online connector needs a publicly reachable HTTPS server.',
          steps: [
            {
              title: 'Use the desktop connection for this server',
              body: 'This Bopixi address is local or uses HTTP. Choose Desktop app above to connect from your computer, or open Bopixi at its public HTTPS address.',
            },
          ],
        }
      }
      return {
        lede: 'Add Bopixi to your Claude account, approve the connection, and start designing. No download needed.',
        steps: [
          {
            title: 'Add Bopixi to Claude',
            body: 'Open Claude with Bopixi’s name and server address already filled in. Review the details and add the connector.',
            action: (
              <Button asChild variant="solid" className="mt-2.5">
                <a
                  href={claudeConnectorInstallUrl(mcpUrl)!}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={() => posthog.capture('claude_connector_opened')}
                >
                  Add to Claude
                </a>
              </Button>
            ),
          },
          {
            title: 'Approve the sign-in',
            body: 'Click Connect on the new connector. A browser tab opens so you can approve the connection.',
          },
          {
            ...pointAt,
            body: prompt
              ? 'In a new chat enable Bopixi under the tools menu and send this as your first message.'
              : 'In a new chat enable Bopixi under the tools menu and tell it which canvas to work on.',
          },
        ],
      }
    case 'chatgpt':
      if (!canUseChatGPTPublicEndpoint(mcpUrl)) {
        return {
          lede: 'ChatGPT needs a public HTTPS address or a secure tunnel to reach this Bopixi server.',
          steps: [
            {
              title: 'Make your Bopixi server reachable',
              body: 'ChatGPT cannot reach this address directly. Open your public HTTPS Bopixi address and start this connection there, or configure an OpenAI Secure MCP Tunnel for this server.',
              code: mcpUrl,
              action: (
                <Button asChild variant="solid" className="mt-2.5">
                  <a href={CHATGPT_CONNECTION_GUIDE} target="_blank" rel="noopener noreferrer">
                    View ChatGPT setup guide
                  </a>
                </Button>
              ),
            },
          ],
        }
      }
      return {
        lede: 'No download. Open ChatGPT, paste the server address below, and approve the Bopixi connection.',
        steps: [
          {
            title: 'Open ChatGPT plugins',
            body: (
              <>
                Click the <strong>+</strong> button on the Plugins page, then <strong>Add custom MCP server</strong>.
                Your account or workspace must allow custom servers.
              </>
            ),
            action: (
              <Button asChild variant="solid" className="mt-2.5">
                <a
                  href={CHATGPT_PLUGINS_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={() => posthog.capture('chatgpt_connection_opened')}
                >
                  Open ChatGPT setup
                </a>
              </Button>
            ),
          },
          {
            title: 'Add Bopixi and sign in',
            body: (
              <>
                Name it <strong>Bopixi</strong>, describe it as a design canvas, and paste this as the MCP server URL.
                Choose <strong>OAuth</strong>, review the permissions, then select <strong>Create as a plugin</strong>.
                Sign in to Bopixi when prompted and install the resulting plugin.
              </>
            ),
            code: mcpUrl,
          },
          {
            ...pointAt,
            body: prompt
              ? 'Start a new chat, type @ and select Bopixi, then send this as your first message.'
              : 'Start a new chat, type @ and select Bopixi, then tell it which canvas to work on.',
          },
        ],
      }
    case 'codex':
      return {
        lede: 'Add Bopixi from the desktop app, sign in, and start designing with Codex.',
        steps: [
          {
            title: 'Add Bopixi to Codex',
            body: 'Bopixi Desktop configures Codex for you. This works with Codex CLI and the Codex tab in the ChatGPT desktop app.',
            action: <CodexInstall mcpUrl={mcpUrl} canvasId={canvasId} />,
          },
          {
            title: 'Sign in from Codex',
            body: 'Open a new Codex chat. If Bopixi needs authentication, sign in from its MCP settings. For Codex CLI, run the command below and approve the connection in your browser.',
            code: 'codex mcp login bopixi',
          },
          pointAt,
        ],
      }
    case 'cursor':
      return {
        lede: 'Add Bopixi to Cursor, sign in, and start designing together.',
        steps: [
          {
            title: 'Add Bopixi to Cursor',
            body: 'Open Cursor and review the server connection. No file download needed.',
            action: <CursorInstall mcpUrl={mcpUrl} />,
          },
          {
            title: 'Approve the sign-in',
            body: 'Enable the bopixi server in Cursor and complete its browser sign-in. Your agent can then work on the canvases you can access.',
          },
          {
            ...pointAt,
            body: prompt
              ? 'Open the Agent chat and send this as your first message.'
              : 'Open the Agent chat and tell it which canvas to work on.',
          },
        ],
      }
    case 'mcp':
      return {
        lede: 'Bopixi is a streamable-HTTP MCP server with OAuth. Any client that supports that can design here.',
        steps: [
          {
            title: "Add this server to your client's MCP config",
            body: (
              <>
                Windsurf, Zed, Cline and VS Code all read a JSON file like this. Paste the <code>bopixi</code> entry into
                it.
              </>
            ),
            code: jsonConfig,
          },
          {
            title: 'Approve the sign-in',
            body: 'The first time your client connects, a browser tab opens to authorize it. The agent then works as you, and its edits are attributed to you.',
          },
          pointAt,
        ],
      }
    case 'antigravity':
      return {
        lede: 'Two steps. Antigravity (agy) connects over MCP to design on your canvases.',
        steps: [
          {
            title: 'Add Bopixi to Antigravity MCP config',
            body: (
              <>
                Add Bopixi to <code>~/.gemini/config/mcp_config.json</code> (global) or{' '}
                <code>.gemini/mcp_config.json</code> in your workspace:
              </>
            ),
            code: JSON.stringify({ mcpServers: { bopixi: { url: mcpUrl } } }, null, 2),
          },
          {
            title: 'Start Antigravity',
            body: 'Run the agy CLI in your terminal and leave it open. It connects to Bopixi automatically:',
            code: 'agy',
          },
          pointAt,
        ],
      }
  }
}

function AgentSteps({
  agent,
  canvasId,
  showAgentStatus,
  onBack,
  onClose,
}: {
  agent: AgentId
  canvasId?: string
  showAgentStatus: boolean
  onBack: () => void
  onClose: () => void
}) {
  const entry = findAgent(agent) ?? findAgent('mcp')!
  const mcpUrl = `${location.origin}/mcp`
  const [claudeConnection, setClaudeConnection] = useState<ClaudeConnection>(() =>
    canAddToClaudeDesktop() || !claudeConnectorInstallUrl(mcpUrl) ? 'desktop' : 'web',
  )
  /* Deliberately thin: the MCP server ships its own INSTRUCTIONS on connect and the
     rest lives behind get_guide. The only thing this prompt knows that they don't is
     which canvas the human is looking at. */
  const prompt = canvasId
    ? `Work on Bopixi canvas ${canvasId}. Start with get_guide({ topic: "bopixi-instructions" }) and follow it.`
    : ''
  const { lede, steps } = entry.custom
    ? customSteps(entry, mcpUrl, prompt)
    : isNativeAgent(agent)
      ? agentSteps(agent, mcpUrl, prompt, claudeConnection, canvasId)
      : manualSteps(entry, mcpUrl, prompt)
  return (
    <>
      <div className={body}>
        <div className="flex items-center gap-3.5">
          <span className="grid size-11 shrink-0 place-items-center rounded-[11px] border border-line-soft bg-paper">
            <AgentMark entry={entry} size={22} />
          </span>
          <div>
            <ModalTitle>Connect {entry.name}</ModalTitle>
            <ModalLede className="mt-1">{lede}</ModalLede>
          </div>
        </div>

        {agent === 'claude' && (
          <div className="mt-5 flex gap-2" role="group" aria-label="Claude connection method">
            {(['web', 'desktop'] as const).map((connection) => (
              <Button
                key={connection}
                size="sm"
                variant={claudeConnection === connection ? 'solid' : 'ghost'}
                aria-pressed={claudeConnection === connection}
                onClick={() => setClaudeConnection(connection)}
              >
                {connection === 'desktop' ? 'Desktop app' : 'Connect online'}
              </Button>
            ))}
          </div>
        )}

        <ol className="mt-6">
          {steps.map((step, i) => (
            <li
              key={step.title}
              className={cn('grid grid-cols-[28px_1fr] gap-3.5', i > 0 && 'mt-3.5 border-t border-line-soft pt-3.5')}
            >
              <span className="mt-px grid size-[26px] place-items-center rounded-full bg-ink text-xs font-bold text-white">
                {i + 1}
              </span>
              <div>
                <h4 className="text-copy font-semibold text-ink">{step.title}</h4>
                <p className="mt-0.5 text-body leading-[1.5] text-ink-soft [&_code]:rounded-[4px] [&_code]:bg-paper-deep [&_code]:px-1 [&_code]:font-mono [&_code]:text-caption [&_code]:text-ink">
                  {step.body}
                </p>
                {step.code && <CodeBlock text={step.code} className="mt-2.5" />}
                {step.action}
              </div>
            </li>
          ))}
        </ol>

        {agent === 'claude' && claudeConnection === 'web' && claudeConnectorInstallUrl(mcpUrl) && (
          <Disclosure label="Connection help" className="mt-4 text-label leading-relaxed text-ink-soft">
            <p>
              If Claude asks you to sign in first, return here and click Add to Claude again. You can also use{' '}
              <strong>Customize → Connectors → Add custom connector</strong> in Claude with this address:
            </p>
            <CodeBlock text={mcpUrl} className="mt-2" />
          </Disclosure>
        )}

        {agent === 'mcp' && (
          <p className="mt-4 text-xs leading-[1.5] text-ink-faint">
            No browser to sign in with (n8n, Mastra, CI)? Mint an agent key in{' '}
            <button
              type="button"
              className="font-medium text-accent-ink underline underline-offset-2"
              onClick={() =>
                navigate(`/settings?pane=keys${canvasId ? `&from=${encodeURIComponent(`/c/${canvasId}`)}` : ''}`)
              }
            >
              Settings → Agent keys
            </button>{' '}
            and send it as <code>Authorization: Bearer bpk_…</code>. The key acts as you, on the canvases you can reach.
          </p>
        )}

        {entry.custom && (
          <button
            type="button"
            className="mt-5 inline-flex items-center gap-1.5 text-xs text-ink-faint transition-colors hover:text-accent-ink"
            onClick={() => {
              removeCustomAgent(entry.id)
              onBack()
            }}
          >
            <TrashIcon width={13} height={13} />
            Remove {entry.name} from this list
          </button>
        )}
      </div>

      <ModalActions className={footer}>
        {canvasId && showAgentStatus ? <AgentArrival id={agent} name={entry.name} /> : <ModalSpacer />}
        <button
          type="button"
          className="inline-flex items-center gap-1 rounded-sm px-2 py-2 text-xs font-medium text-ink-soft hover:text-ink"
          onClick={onBack}
        >
          <ChevronLeftIcon width={14} height={14} />
          Back
        </button>
        <Button onClick={onClose}>Done</Button>
      </ModalActions>
    </>
  )
}

/* ---------- shared pieces ---------- */

/* The modal is a column: the screen's body scrolls in the middle and the
   action row sits on the bottom edge, so the back button and the main action
   stay in reach on the long agent screens (the JSON config plus the headless
   note runs past a laptop viewport). */
const body = 'min-h-0 flex-1 overflow-y-auto p-5'
const footer = 'mt-0 shrink-0 items-center border-t border-line-soft bg-surface px-5 py-4'

function ChoiceCard({
  selected,
  onClick,
  icon,
  title,
  blurb,
  status,
}: {
  selected: boolean
  onClick: () => void
  icon: ReactNode
  title: string
  blurb: string
  status?: ReactNode
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className={cn(
        'relative flex items-start gap-3 rounded-card border bg-surface py-3.5 pl-3.5 pr-9 text-left text-ink transition-colors',
        selected ? 'border-brand' : 'border-line hover:border-ink-faint',
      )}
    >
      <span className="grid size-9 shrink-0 place-items-center rounded-[9px] border border-line-soft bg-paper">
        {icon}
      </span>
      <span className="block min-w-0">
        <span className="block text-copy font-semibold">{title}</span>
        <span className="mt-0.5 block text-xs leading-[1.5] text-ink-soft">{blurb}</span>
        {status && <span className="mt-1 block">{status}</span>}
      </span>
      <span
        aria-hidden
        className={cn(
          'absolute right-4 top-1/2 size-4 -translate-y-1/2 rounded-full border',
          selected ? 'border-[5px] border-brand' : 'border-line',
        )}
      />
    </button>
  )
}

/** Live connection status: flips the moment an outside (non-resident) agent
 *  joins this canvas's presence, so nobody is left wondering whether the
 *  OAuth dance actually worked. With a roster, the arrival prefers the agent
 *  whose screen is open. */
export function AgentArrival({ name, id }: { name?: string; id?: AgentId }) {
  const presences = useStore((s) => s.presences)
  const arrived = useMemo(() => {
    const outside = Object.values(presences).filter((p) => p.kind === 'agent' && !roleByAgentName(p.name))
    return (id && outside.find((p) => agentIdOfName(p.name) === id)) || outside[0]
  }, [presences, id])
  return arrived ? (
    <span className="mr-auto inline-flex items-center gap-[7px] text-body text-success-ink">
      <CheckIcon size={14} />
      {arrived.name} is here — it worked
    </span>
  ) : (
    <span className="mr-auto inline-flex items-center gap-[7px] text-body text-ink-faint">
      <Dot className="animate-[arrival-pulse_1.6s_ease-in-out_infinite] bg-brand" /> listening for{' '}
      {name ?? 'your agent'}…
    </span>
  )
}
