# Changelog

## 0.10.0 (2026-10-26)


### Features

* **automations:** run workflows when designs are approved (#358) (8df0e89)
* **canvas:** add design panel geometry, layout, and appearance controls (#340) (d98df37)
* **canvas:** add element context menus and restrict frame menus to titles (#351) (0c13614)
* **canvas:** add image fill editor with replacement and sizing (#361) (803a750)
* **canvas:** add ready-for-dev screen approvals (#353) (ded535d)
* **canvas:** add rulers and draggable snapping guides (#372) (fdbb07e)
* **canvas:** clickable prototypes with interactions and a player (#373) (420e177)
* **canvas:** copy a frame as PNG from its menu and the inspector (#219) (06169b4)
* **canvas:** copy links to specific frame elements (#338) (81fb23a)
* **canvas:** drag HTML and SVG layers directly on the canvas (#347) (989721c)
* **canvas:** keep zoom sharp up to 800% and steady peer cursors (#216) (aa04e95)
* **canvas:** open element properties while editing text (#226) (dcb3a84)
* **canvas:** resize elements inside a frame with drag handles (#343) (f897395)
* **canvas:** resize the Layers and Agents panels (#225) (3b3bd59)
* **canvas:** select multiple elements in a frame (#362) (98b646a)
* **canvas:** snap dragged elements into DOM layout (#359) (f7e3a19)
* **canvas:** use icons in the element toolbar (#227) (7e8a7c6)
* **design:** link width and height with an aspect ratio toggle (#342) (8ef7396)
* **design:** support dragging numeric field units (#339) (1a3373f)
* **export:** vector PDF export for frames, selections and pages (#370) (0bd4647)
* **ios:** add selection action bar and image sharing (#357) (609ce87)
* **ios:** refine composer and add agent and model pickers (#238) (1cef0cf)
* **models:** discover API-key provider models from their catalogs (#236) (754d6b8)
* **sharing:** view-only and comment-only share links (#369) (c65183b)
* **ui:** animate rail icons on hover (#374) (f8c256e)


### Bug Fixes

* **analytics:** count Google, Microsoft, Apple and SSO signups (#367) (7c2e75e)
* **canvas:** add font selector and text alignment controls (#337) (4e3e8b6)
* **canvas:** bring the top bar and panel chrome in line with the system (#231) (64d62c4)
* **canvas:** drop the "Click any text to edit" note from the edit pill (2ee9ec4)
* **canvas:** give the Connect AI button the shadow ring (#232) (bd37598)
* **canvas:** hide resize handles for unsupported transforms (91e8bad)
* **canvas:** keep the follow pill clear of the presence avatars (#224) (4512dae)
* **canvas:** keep the hover tag above the selection outline (#218) (0531b70)
* **canvas:** match the welcome frame's Connect AI chip to the app (#228) (116d6c1)
* **canvas:** polish the remote-edit flash and selection cues (#230) (70ac16b)
* **canvas:** preserve fast element resize gestures (cb176aa)
* **canvas:** resize flex Fill children with handles (#211) (0ab9cf0)
* **canvas:** tidy the getting started card (#229) (a4b7ab8)
* **comments:** make resolved replies readable in the panel (#332) (d1f9752)
* **drawing:** reject conflicting transfer saves and deletes (#335) (2caad52)
* **inspector:** align coordinate and dimension field labels (#346) (3d94cb4)
* **integrations:** finish OAuth connections started in the desktop app (#349) (c87949b)
* **ios:** clip outer box-shadow and outline with clip-path (#368) (bf18156)
* **ios:** isolate canvas pages and add page switching (#348) (832427d)
* **ios:** refine canvas interactions and native controls (#344) (bf3ebac)
* **layers:** preserve visibility during inline text edits (1df6c0b)
* **mcp:** mark save_decision idempotent (#223) (2f2e56b)
* **pages:** persist hidden and locked frames in page copies (7f509bd)
* **pages:** restore the selected-frame destination picker (#334) (3edf6e8)
* **ui:** readable native select options in Windows dark mode (#363) (524d5bc)
* **ui:** replace window.confirm with ConfirmDialog for destructive settings actions (#205) (2f88fa9)


### Documentation

* refresh the README visuals and add agents and integrations (#207) (74e7eb4)

## 0.9.0 (2026-10-21)


### Features

* **agents:** dock the "Design it" composer in the agents tab instead of a floating bar (#277) (361ada6)
* **auth:** land social and SSO signups on their first canvas (#245) (c32df65)
* **auth:** sign in with Apple on the web and natively on iOS (#263) (8397b1e)
* **canvas:** ⌘. hides all panels and toolbars, Figma-style (#274) (8fb92e5)
* **canvas:** add drawing tools to the bottom toolbar (#292) (9a771f5)
* **canvas:** bottom toolbar with select, hand and comment tools (#275) (210525a)
* **canvas:** draw Bézier curves and move elements between frames (#293) (b611204)
* **canvas:** replace chat with a comments panel and drop the chat data model (#272) (abe2fa5)
* **canvas:** snap and edit vector path anchors (#295) (7e24f96)
* **canvas:** zoom controls move to the bottom-right with a zoom menu on the percentage (#271) (5058e61)
* **claude:** add direct connector setup and desktop extension (#280) (d058f12)
* **comments:** comment cursor and comments anywhere on the canvas (#290) (a880ff8)
* **connect:** add Cursor plugin and ChatGPT setup (#281) (1881b5e)
* **desktop:** add automatic Codex MCP setup (#289) (3b5b090)
* **editor:** getting-started panel sits bottom left of the canvas (#306) (83f2f95)
* **editor:** help button in the side rail with Docs and Contact support (c77801f)
* **editor:** the tool rail's active fill slides between tools (#303) (9abe6b3)
* **editor:** white design system refresh for the canvas editor (#300) (8dc2269)
* **frames:** add frame version history with restore (#258) (a30bb35)
* **home:** new home with prompt composer, shared rail and canvas tiles (#307) (ba97429)
* **ios:** lighter dark canvas and selection-scoped prompts (c2db870)
* **ios:** sign in with Google, Microsoft and SSO (#259) (2958a9c)
* **ios:** snap dragged frames to alignment guides (#257) (4c28b38)
* **layers:** a Layers header like the right-hand panel (#291) (b31f7e0)
* **layers:** eye toggle to hide frames and layers from the layers panel (#276) (e829182)
* **layers:** lock frames and layers (#278) (cd0b827)
* **layers:** pages above frames, search at the bottom, no hints or counter (#288) (de4390c)
* **mcp:** make Bopixi a ChatGPT app with an inline frame preview card (#264) (35579dd)
* **pages:** duplicate page from the context menu (#273) (ad62ebd)
* **panel:** icon tabs in the side panel, the active one unfolds its label (#284) (8251a12)
* **slack:** add the Bopixi Slack agent (#256) (8a54d79)
* **ui:** component layer for every dashboard screen (#325) (07af1be)
* **ui:** component layer for the dashboard chrome (03b2599)
* **ui:** modals open over 250ms from 96% and close over 150ms (#304) (9bcf1ec)
* **ui:** named type scale and corner shapes in the design system (#308) (0a5a78e)
* **ui:** quieter modal chrome (#305) (408050d)
* **ui:** storybook catalogue of the design system (cb6d3d6)


### Bug Fixes

* **agents:** refuse and stop work for a banned installer (0d087c6)
* **agents:** time a run out from dispatch, not from the request's arrival (faf4528)
* **auth:** disconnect the Slack and Linear agents when an account is deleted (63139ed)
* **build:** resolve Claude extension dependencies under Bun (#283) (a4c54d5)
* **canvas:** carry body attributes into the frame runtime (#326) (e56ac62)
* **canvas:** crisp-render zoom keeps a frame's viewport units intact (#329) (23e4e06)
* **canvas:** keep contact support chat open (#328) (f097917)
* **canvas:** slim path editing guides and handles (#296) (6e155c9)
* **canvas:** the Select tool keeps the arrow cursor over frames (#287) (2c35f3c)
* **chatgpt:** no blank card on create_frame, placeholder for empty frames (#267) (83b96a0)
* **comments:** the comment UI keeps its screen size at every zoom (#286) (fc3005f)
* **editor:** no names on layer rows, even gap in the online pill (43cb94a)
* **editor:** no tooltips on the layer row eye and lock (5310393)
* **editor:** support chat bubble stays off the canvas editor (a13eb55)
* **ios:** hold the pinch anchor when a finger lifts and snap borders at every zoom (#262) (2758f4b)
* **ios:** stop false connection interruption loops (#268) (89fd9b1)
* **mcp:** prime the screenshot cache from a frame snapshot (9209c89)
* **pages:** page rename input text was invisible in light mode (#270) (ed23d23)
* **slack:** join an early follow-up to the thread's first request (1e85e62)
* **tests:** keep the test harness off the live PostHog project (#327) (0afa052)
* **ui:** phone dashboard opens the rail as a drawer (#313) (38e45fe)

## 0.8.0 (2026-10-13)


### Features

* **canvas:** add named pages within canvases (#235) (0e3484c)
* **comments:** email collaborators about new comments and replies (e402606)
* **connect:** pick the agent first in Connect AI, nudge to connect a plan from the prompt bar (#226) (aa1442b)
* **ios:** add native app with agent status and Live Activities (#173) (94afcd7)
* **ios:** end Live Activities as soon as the app sees the task is over (#243) (6f1aabf)
* **ios:** redesign Live Activity with bopixi logo and step log (#241) (fa2a02a)
* **server:** make per-tool rate limits configurable via env vars (#176) (a7322df)
* **ui:** dark mode with theme tokens, zero-flash boot and contrast fixes (#164) (1a3c3a6)
* **webhooks:** add per-user outbound webhooks for canvas events (#230) (74df768)


### Bug Fixes

* **agent:** recover interrupted model streams (#223) (67ee169)
* **identity:** keep one client id when localStorage writes throw (a939c69)
* **integrations:** stop the Linear tile shrinking in Safari (#227) (e1f3861)
* **ios:** only reconcile activities open before the fetch (1e9136d)
* **ios:** pick the APNs environment from the provisioning profile (#237) (32f62a2)
* **live-activities:** log every push outcome (#240) (9036777)
* **live-activities:** log registration requests ahead of auth (#238) (ce76cb7)
* **live-activities:** register push-to-start tokens on their own route (#236) (5be9651)
* **pages:** rehome frames whose page is gone on boot (178dff9)
* **ui:** use theme colors in element action popover (1c20765)

## 0.7.0 (2026-10-01)


### Features

* **agent:** add an opt-in Gemini cloud worker prototype (#220) (f154e92)
* **agents:** support OpenRouter and Gemini model accounts and an image-model registry (#211) (3253e88)
* **canvas:** export progress toast and desktop save panel for exports (#221) (34b65ca)
* **canvas:** move frame export options into a modal (#219) (aa6cb82)
* **canvas:** per-canvas chat with [@mentions](https://github.com/mentions) that assign agents (#197) (2c4f077)
* **canvas:** redesign the import modal as a source picker with one screen per source (#208) (e5eaafb)
* **linear:** run Bopixi designs from delegated linear tickets (#214) (44c32ac)
* **mcp:** agent keys for headless MCP clients (bearer auth) (#213) (33379d4)


### Bug Fixes

* **agents:** drop the hover-only 'Not available' label on task rows (#200) (fbfdd9a)
* **canvas:** drop icons from import screen primary actions (#215) (510a2da)
* **canvas:** keep a followed avatar's colour under the follow halo (#162) (77cb306)
* land a new frame in the middle of the current view (#165) (d540e7e)
* **linear:** simplify integration card and use official logo (#222) (f08782c)
* **server:** bump undici to 8.x to stop an uncatchable parser crash (#198) (894b1a2)

## 0.6.0 (2026-09-19)


### Features

* **agent:** add Claude CLI and API key providers (#155) (4bed23d)
* **agent:** add GPT-6 Astra to the OpenAI model menu (#190) (31123df)
* **canvas:** locate and follow collaborators from task rows and avatars (#191) (876abcf)


### Bug Fixes

* **canvas:** keep frame title labels from stealing clicks at low zoom (#193) (3a5df79)
* **canvas:** snap duplicate-drags to guides and let ⌫ delete the picked element (#187) (08c9fe7)
* close the socket with a clear code when a canvas does not exist (#154) (39f037e), closes #73
* **db:** renumber the local-agent migration to 0017 behind task_frames (a4cc4bb)
* one-click copy button for the frame's HTML code (#148) (2230a8b)


### Documentation

* **agents:** teach agents to generate hero backgrounds (90853e8)

## 0.5.0 (2026-09-16)


### Features

* **agents:** generate_image tool draws with AI on the user's own account (#165) (ad55317)
* **automations:** scheduled pulls and agent tasks, with a Meta integration (#168) (ce34ba6)
* **canvas:** element properties panel opens from a click on the canvas (#167) (510fdfe)
* **canvas:** prompt bar scopes a request to the selected frame or element (#180) (ba36d77)
* **canvas:** reorder elements in the Layers panel (#164) (fbc831e)
* **desktop:** google, microsoft and sso sign-in through the system browser with a bopixi:// handoff (#177) (5b51765)
* **mcp:** reply to and resolve element comments (#150) (775fd2e)
* **workspaces:** shared workspaces with a per-seat Team plan on Stripe (#170) (f64c9ab)


### Bug Fixes

* **canvas:** clicking a second element in a frame moves the selection (#171) (bd23ae6)
* **canvas:** guard Layers drops against void targets and stale selectors (#169) (8cfcd1a)
* **layers:** delete key on a layer row no longer deletes the whole frame (#179) (377ae4c)
* **mcp:** bill agent comment replies to the connecting user, not the canvas owner (5198236)
* **mcp:** protected-resource metadata names the /mcp endpoint so strict OAuth clients can connect (#174) (fcf2947)
* **replay:** capture sandboxed frame content in PostHog (#176) (4a50323)
* **workspaces:** order billing events across subscriptions, durable revocations, owner-only moves in (33aff5c)

## 0.4.0 (2026-09-10)


### Features

* **auth:** sign in with Google, and enforce SIGNUP_EMAIL_DOMAINS on every sign-up path (#159) (1a78a84)
* **auth:** sign in with Microsoft (#160) (b736528)
* **backgrounds:** list_backgrounds — browse the library by eye instead of keyword search (#154) (941074f)
* **canvas:** collapsible side rail replaces the top-bar Activity button (#158) (619734e)
* **canvas:** element properties panel opens from a Layers row (#161) (80c63ad)
* **canvas:** layers panel on the left, frame inspector moves right (#157) (86a2c9c)
* **canvas:** present mode plays the selected frame full-screen (#163) (ccab4eb)
* **canvas:** restyle the top bar after the Bopixi interface reference (#156) (2a45f34)
* **inspiration:** drop the recipe menu; the brief starts with a live search (#134) (c3aaa0c)
* **mcp:** search_backgrounds tool over a curated background library (#152) (1c4852f)


### Bug Fixes

* **desktop:** cmd+w closes the active tab instead of the app, logout lands on /auth (#155) (eaeebd2)

## 0.3.0 (2026-09-09)


### Features

* **agents:** demand real logos via search_logos instead of placeholders (#149) (1051c5e)
* **canvas:** marquee selection, shift-click multi-select and space-to-pan (#141) (1b1a5f6)
* **comments:** reply inside an element comment thread (#140) (cd7e6ee)
* **community:** gallery of published designs to preview and copy (#147) (83a2773)
* **desktop:** add Windows application support (#112) (2899cc3)
* **github:** queue repo imports as board cards instead of outline frames (#138) (e495537)
* **importer:** drop [@font-face](https://github.com/font-face) blocks the page never uses (#126) (9dc4230)
* **importer:** prune CSS the captured page does not use (#125) (1f03123)
* **mcp:** add read-only access to canvas comments (#117) (bbb174e)
* **ui:** bopixi mark for agents and coloured crew badges on the board (#144) (1fd46f4)


### Bug Fixes

* **agent:** stream Anthropic turns so long recon runs are not refused (#130) (bd48e16)
* **canvas:** copy, paste and duplicate the whole selection (#146) (238dd70)
* **canvas:** satisfy strict TypeScript and lint in the multi-frame clipboard port (9061782)
* **cards:** stop truncating a queued card's prompt at 200 chars (#88) (dbc9f31)
* **community:** size modal preview tiles to the clipped thumbnail, not the frame (#148) (97ffab9)
* correct contrast on tag chip and rules new button (#111) (86a42f1)
* **dash:** point Help & docs at /docs instead of /blog (#142) (503ac68)
* **dash:** point Help & docs at the hosted docs site (396ccba)
* **frame:** confirm the code block copy (#93) (fc1f3fa)
* **frame:** make the code view scrollable (#94) (dcacb21)
* **importer:** allow bigger stylesheets (#114) (009e7be)
* **importer:** keep [@font-face](https://github.com/font-face) blocks when the font shorthand uses var() (#129) (76e9343)
* make extractHtml accept html without doctype and component fragments (#119) (22a8a8a)
* **stage:** pinch to zoom on touch screens (#137) (c91d60c)
* **ui:** default the codex mark to black instead of the presence colour (cf030d9)

## 0.2.0 (2026-09-03)


### Features

* add canvas actions and duplication (#97) (f671e76)
* add Context.dev-backed website imports (100e162)
* add generic OIDC/SSO support (#49) (88b1fe3)
* **admin:** instance admin surface with read-only view-as (10156ef)
* **admin:** restyle /admin to match home and settings (dae3de5)
* **agent:** design-brief inspiration retrieval — style recipes + live refero search (1c949b8)
* **agent:** remove the free Bopixi Agent allowance (e0f640d)
* **agent:** run the Bopixi Agent's server tier on Azure OpenAI (e367ca5)
* **analytics:** capture custom agent connect and use in PostHog (2e70352)
* **analytics:** segment desktop-app sessions in PostHog (#26) (7d0a318)
* **auth:** make email verification optional via REQUIRE_EMAIL_VERIFICATION (7b82e57)
* **auth:** restrict signups by email domain (#44) (10dae3d)
* ban and unban accounts from the admin dashboard (5de4803)
* **canvas:** ⌥⇧-drag duplicates a frame; ⌘0/⌘+/⌘− zoom shortcuts (97bf266)
* **canvas:** render append_frame_html chunks instantly instead of pacing them (dffc2d7)
* **canvas:** sync icon on frames pushed by the design-sync snippet (7e5903d)
* design-sync refinements — import-once, pristine first captures, agent flow context (f98ce7a)
* design-sync snippet — an app's live screens land on a canvas (534934f)
* **design:** move the app onto the Disc-D identity and editorial system (df5a134)
* desktop shell (Tauri) with signed DMG releases (#25) (3f26286)
* **desktop:** figma-style canvas tabs, overlay title bar, external links open in the browser (#93) (3dcc1a5)
* flow map — connectors show which button leads to which screen (e1a4453)
* **github-import:** agent reconstruction of code-only screens (0981875)
* **github-import:** components and design system become the headline (6127e04)
* **github-import:** one-click GitHub App install flow (331cd88)
* **github-import:** review flow leads with the design system (09301ee)
* **github-recon:** render-critique round — the sketch reviews itself (b0ade2e)
* **github-recon:** the model investigates the repo before designing (3bce7c1)
* **github-recon:** the sketch pass shows up as Bopixi working (ff59d6d)
* **github-recon:** transplant real repo assets, full-page output (77483e9)
* **guide:** taste curriculum — mood method, brief ritual, anchor and hero-review rules (87f9246)
* import complete websites with page review (5567f17)
* **import:** connect a GitHub repo as an import source (7392e11)
* **inspo:** refero.design style extractor (inspiration pipeline v0) (9634d6f)
* meter feedback and retries — every resident trigger spends a free task (4949a5f)
* right-click menu on desktop canvas tabs; signed-out visitors always see sign-in (#104) (d79a0c6)
* run the Bopixi Agent on your own ChatGPT subscription (c99ed93)
* **server:** surface Bopixi Agent state at boot (1edec74)
* **ui:** a component system for Bopixi, and the end of plain CSS (26891dc)
* workspace-rail dashboard for home and settings (73f31e0)


### Bug Fixes

* a stalled agent queue no longer hides another agent's runnable work (66e72af)
* **admin:** back button during "view as" returns to admin instead of 404 (43ceb95)
* **agent:** only a connected model account lifts the resident meter (2701602)
* **agent:** read Codex-backend output from streamed items, not the completed event (258ee8f)
* bill one requester per agent run, and stop two account races (7556094)
* **canvas:** move live-app sync keys from the share dialog to Import (b16ba55)
* **canvas:** open the frame details panel only from a frame-name click (b844709)
* **canvas:** raise the element toolbar above the pin-to-memory brain button (8266df5)
* **canvas:** show the duplicate cursor while ⌥⇧ is held, not at drag-start (94ca0fc)
* **ci:** give doc-lint node globals and repo formatting (2b73aef)
* compile the PostHog support widget into the no-external bundle (62e3992)
* **dashboard:** reliable canvas previews — persisted thumbnails, stale-while-revalidate (3aaad88)
* **dashboard:** rounded corners on active view-mode segment (30e1907)
* disable vertical tab bar scrolling (#85) (45250cc)
* **github-app:** handle already-installed accounts; surface install failures on the canvas (ae1e155)
* **github-import:** actionable failed-sketch card; fail fast on dead model accounts (6b572b4)
* **github-import:** default to design-system-only; name the CTA honestly (8d0aa10)
* **github-import:** deploy URL editable after connect; resync upgrades placeholders (ce4a262)
* **github-import:** outline prompts deep-link to their exact frame (e9e26d3)
* hide demo frames from agents so they don't steer new designs (de5e3a5)
* **importer:** skip the scroll walk for Context.dev captures so imports stop hanging (63b46ec)
* **mcp:** state one-query-per-call in search_icons and search_logos descriptions (a7a014f)
* preserve X-Forwarded-For chain in PostHog relay so recordings geolocate the real client (7d0fab4)
* **server:** decode HTML-escaped frame writes (3c64285)
* snippet finds its config when injected by a tag manager (b5d892c)
* sync key rides in the snippet URL — the form tag managers can't strip (6739094)
* **tests:** move admin suite to ports 4985-4988, clear of parallel test files (13f00ea)
* **ui:** hand the limit wall's MCP path off to the connect modal (baadd86)


### Documentation

* add discord badge to readme (d76cf05)
* add link to bopixi website in readme (#96) (f51ba0c)
* document Bopixi Agent setup and standardise its name (b92b65b)
* rebuild the README graphics on the Disc-D identity (#87) (5d5a732)
