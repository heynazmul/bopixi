# RELEASE - Bopixi

## Versioning

This repository has no CI or release automation: the upstream `.github/` workflows,
release-please configuration and CLA were removed when the project was renamed. Versions in
`package.json`, `desktop/package.json`, `desktop/src-tauri/tauri.conf.json` and
`desktop/src-tauri/Cargo.toml` are bumped by hand, and releases are cut locally:

- Web app: bump `package.json`, add a `CHANGELOG.md` entry, tag `vX.Y.Z`.
- Desktop: bump the three desktop files together, keeping `Cargo.lock` in step for
  `cargo test --locked`, then tag `desktop-vX.Y.Z`.

Both packages are 0.x, so breaking changes bump the minor, not the major.

## Self-hosted server (Docker)

No container release pipeline for the server app. Self-hosting is
`docker compose up` (`docker-compose.yml` + `Dockerfile`) - builds the app image locally, runs it
on port 4400 alongside a `postgres:16-alpine` container, and applies DB migrations at boot
(`server/db/index.ts`). Validate changes locally with `bun run typecheck`, `bun run lint`,
`bun run format:check`, `bun run build` and `bun run test`.
The build uses `bun run --bun build` to match the `oven/bun` Docker build stage,
where `node` commands run with Bun. Extension tests also build with both runtimes
and run the Bun-built archive under Node, including its nested runtime dependencies.

## Desktop app (Tauri: macOS DMG + Windows NSIS installer)

Build installers locally with `bunx tauri build` from `desktop/`:

- macOS: `bunx tauri build --target universal-apple-darwin --bundles dmg`.
- Windows: `bunx tauri build --bundles nsis` (`bopixi_<version>_x64-setup.exe`).
- macOS signing/notarization: automatic once the `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
  `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` variables are set in the
  build environment (see `desktop/README.md`). Without them the DMG is unsigned - it still works,
  but downloaders must approve it via System Settings -> Privacy & Security.
- The Windows installer is unsigned (no Authenticode certificate configured): SmartScreen warns on
  first run and users click "More info" -> "Run anyway".
- Connecting an OAuth integration (Meta, Linear, Slack) from the app opens the
  provider in the system browser and returns over a `bopixi://integration/<id>` link; the app
  finishes it under its own session (`shared/desktopConnect.ts`). This is web-only: shells from
  0.2.1 already deliver any `bopixi://` link, and older shells open Integrations in the browser.
- The macOS View menu (`desktop/src-tauri/src/view_menu.rs`) holds the canvas toggles (Rulers,
  Show/Hide UI, Layers and Agents panels) and zoom. Items reach the page as `view-command`
  events; the page mirrors its state back through `view_menu_state` (`src/lib/viewMenu.ts`,
  gated on `window.__BOPIXI_VIEW_MENU__`), so checkmarks follow it and canvas items are disabled
  off a canvas. Only ⌘ shortcuts are bound in the menu; ⇧R / ⇧1 / ⇧2 stay with the page.
- The shell tells the page its platform via `window.__BOPIXI_DESKTOP_PLATFORM__` (read in
  `src/lib/shell.ts`); only macOS gets the overlay title bar / traffic-light inset.
- Desktop app version is tracked separately from the root `package.json`; bump it by hand and tag
  as `desktop-vX.Y.Z` when you publish installers.

## Claude Desktop extension

`bun run build:claude-extension` packages `claude-extension/manifest.json`, the Bopixi logo,
license and pinned `mcp-remote` runtime into `public/claude-extension/bopixi.mcpb`. Both `dev`
and `build` run it automatically; Vite copies the download into the production `dist/`.
Connect AI → Claude offers a prefilled **Add to Claude** link for public HTTPS servers.
The shared endpoint check filters common local addresses while allowing public IPv6
and numeric-prefixed DNS names; the connecting client checks actual reachability.
In updated desktop shells it prepares the extension, prefills the current `/mcp` URL,
and invokes `claude_add_extension` to open Claude's installer. Native installation
requires a desktop release; the online link ships with the web app. Manual downloads
remain a collapsed fallback for older shells and local browser sessions.
OAuth requests for consent open `/oauth/consent`. The approval screen resolves the client
from a code bound to the signed-in account; `/api/mcp-consent/:code` handles approval and
denial through better-auth. This also supports other MCP clients that request consent.

`tests/claudeExtension.test.ts` extracts the actual archive outside the repository and checks
tools, resources, OAuth and reconnecting against a fresh Bopixi server. Before releasing,
also install it in Claude Desktop on macOS and Windows. Installation, build and release
notes live in [`claude-extension/README.md`](../claude-extension/README.md). No directory
submission or publication is performed by the build.

## Codex desktop setup

Connect AI → Codex can register Bopixi directly in the shared Codex config through
`codex_add_mcp`. Browser sessions hand off to the installed app with a `bopixi://connect/codex`
link; the app opens a review screen and writes only after the user clicks Add to Codex.
The native capability marker gates older shells. Release the updated desktop binary
and frontend together; no downloadable integration bundle is built. Codex still owns
the separate OAuth approval. Config preservation, conflict handling and handoff checks
are covered by Rust/frontend tests; desktop CI now runs the Rust tests on both platforms.
See [`desktop/README.md`](../desktop/README.md#add-to-codex) for release smoke checks.

## Cursor marketplace plugin

`cursor-plugin/` is a standalone Cursor plugin directory; `.cursor-plugin/marketplace.json` at
repo root indexes it. It contains the hosted MCP connection, the scoped `bopixi-design` skill,
metadata, logo, and license. It needs no build artifact or release download. The app's
**Connect AI → Cursor → Add to Cursor** uses Cursor's MCP install link for the current Bopixi
instance and is available independently of marketplace publication.

To publish the plugin, submit this repository at `https://cursor.com/marketplace/publish`. Confirm
the root marketplace manifest and `cursor-plugin/` are present at the same paths. There is no
export or cherry-pick step; this repository is the listing source. Cursor must approve the plugin
before it appears in the public marketplace. Nothing submits or publishes it automatically. Bump
the version in `cursor-plugin/.cursor-plugin/plugin.json` for plugin updates; full setup and
reviewer steps live in `cursor-plugin/README.md`.
