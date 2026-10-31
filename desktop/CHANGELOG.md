# Changelog

## 0.7.0 (2026-10-26)


### Features

* **canvas:** add rulers and draggable snapping guides (#372) (fdbb07e)
* **desktop:** add macOS check for updates menu (#350) (f16644a)


### Bug Fixes

* **desktop:** crisp Windows window icon (#364) (37214b8)

## 0.6.0 (2026-10-21)


### Features

* **claude:** add direct connector setup and desktop extension (#280) (d058f12)
* **desktop:** add automatic Codex MCP setup (#289) (3b5b090)

## 0.5.0 (2026-10-13)


### Features

* **connect:** pick the agent first in Connect AI, nudge to connect a plan from the prompt bar (#226) (aa1442b)
* **desktop:** add a drag-to-Applications background to the macOS DMG (#225) (8f5ada5)

## 0.4.0 (2026-09-19)


### Features

* **agent:** add Claude CLI and API key providers (#155) (4bed23d)

## 0.3.0 (2026-09-16)


### Features

* **analytics:** segment desktop-app sessions in PostHog (#26) (7d0a318)
* **design:** move the app onto the Disc-D identity and editorial system (df5a134)
* desktop shell (Tauri) with signed DMG releases (#25) (3f26286)
* **desktop:** add Windows application support (#112) (2899cc3)
* **desktop:** ask where to save PNG/JPG exports with a native save panel (#181) (a42e8ee)
* **desktop:** figma-style canvas tabs, overlay title bar, external links open in the browser (#93) (3dcc1a5)
* **desktop:** google, microsoft and sso sign-in through the system browser with a bopixi:// handoff (#177) (5b51765)


### Bug Fixes

* **desktop:** cmd+w closes the active tab instead of the app, logout lands on /auth (#155) (eaeebd2)
* **desktop:** let the tab strip drag the window on macOS (#106) (4309e7e), closes #95
* **desktop:** mailto and tel links open the system handler instead of being dropped (#172) (15385ee)
* **desktop:** PNG/JPG exports download to ~/Downloads instead of opening in the window (#178) (115989f)
* **desktop:** regenerate app-icon.png on the Disc-D mark (#103) (5e10b49)
* **mcp:** bill agent comment replies to the connecting user, not the canvas owner (5198236)

## 0.2.1 (2026-09-10)


### Bug Fixes

* **desktop:** cmd+w closes the active tab instead of the app, logout lands on /auth (#155) (eaeebd2)

## 0.2.0 (2026-09-09)


### Features

* **desktop:** add Windows application support (#112) (2899cc3)


### Bug Fixes

* **desktop:** let the tab strip drag the window on macOS (#106) (4309e7e), closes #95

## 0.1.4 (2026-09-03)


### Bug Fixes

* **desktop:** regenerate app-icon.png on the Disc-D mark (#103) (5e10b49)
