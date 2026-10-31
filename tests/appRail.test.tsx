// @vitest-environment happy-dom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  navigate: vi.fn(),
  listCanvases: vi.fn().mockResolvedValue([]),
  listWorkspaces: vi.fn().mockResolvedValue({ workspaces: [] }),
}))

vi.mock('../src/lib/api', () => ({
  api: {
    listCanvases: mocks.listCanvases,
    listWorkspaces: mocks.listWorkspaces,
  },
  errorMessage: (e: unknown, fallback: string) => fallback,
}))
vi.mock('../src/lib/auth', () => ({
  authClient: {
    useSession: () => ({ data: { user: { name: 'Test User' } } }),
  },
}))
vi.mock('../src/lib/navigate', () => ({ navigate: mocks.navigate }))
vi.mock('../src/lib/railDrawer', () => ({
  useRailDrawer: (select: (s: { open: boolean; setOpen: (v: boolean) => void; setMounted: (v: boolean) => void }) => unknown) =>
    select({ open: false, setOpen: vi.fn(), setMounted: vi.fn() }),
}))

import { AppRail } from '../src/components/AppRail'
;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let root: Root
let container: HTMLDivElement

beforeEach(() => {
  vi.clearAllMocks()
  localStorage.clear()
  vi.stubGlobal('location', new URL('http://localhost:4301'))
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
  vi.unstubAllGlobals()
})

const mount = async () => {
  container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  await act(async () => root.render(<AppRail active="all" />))
}

it('renders the Agents section with a plus button and lists custom agents', async () => {
  localStorage.setItem(
    'bopixi:customAgents',
    JSON.stringify([{ id: 'custom-agy', name: 'agy', command: 'agy' }]),
  )

  await mount()

  const text = document.body.textContent ?? ''
  expect(text).toContain('Agents')
  expect(text).toContain('agy')

  // Plus button with aria-label "Add a CLI agent"
  const addBtn = document.body.querySelector('button[aria-label="Add a CLI agent"]')
  expect(addBtn).not.toBeNull()

  // Click the plus button to open AddAgentForm
  await act(async () => {
    addBtn?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
  })

  expect(document.body.textContent).toContain('Add a CLI agent')
  expect(document.querySelector('#custom-agent-name')).not.toBeNull()
  expect(document.querySelector('#custom-agent-command')).not.toBeNull()
})
