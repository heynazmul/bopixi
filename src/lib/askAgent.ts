/* Asking the canvas agent about one frame or element. A context-menu row
   dispatches this window event and CanvasPage owns the response — select the
   frame, pick the element, then open the Agents composer with ⌘K's focus.
   An event rather than a callback prop because the menus render from portal
   subtrees the page never passes props into. */

export const ASK_AGENT_EVENT = 'bopixi:ask-agent'

export interface AskAgentDetail {
  frameId: string
  /** an element inside the frame, as a selector path from the live document */
  selector?: string
}

export function askAgentAbout(detail: AskAgentDetail) {
  window.dispatchEvent(new CustomEvent<AskAgentDetail>(ASK_AGENT_EVENT, { detail }))
}
