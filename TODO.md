# TODO — Realmseed

## UI_STYLE review — 2026-09-20

Audit and planning only. Implement the tasks below in dependency order, keeping
each independently useful change reviewable. The previous file contained only
“No outstanding AI tasks.”; there were no existing checklists or completion
history to merge. Keep this tracked `TODO.md` filename (the project-root
`todo.md` on Windows), rather than creating a second file.

### Evidence and scope

- Read `AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`,
  `GAME_DEVELOPMENT_GUIDE.md`, `README.md`, and `gdd.md`. No
  `PROJECT_AGENTS.md` or additional nested `AGENTS.md` was found.
- Inspected `src/ui.rs`, all screen/panel modules, `src/game.rs` screen flow,
  `src/main.rs` capture entry point, map projection/picking, relevant simulation
  queries, text data, and toolkit pointer/viewport helpers.
- Visually inspected existing 1280×720 captures:
  `docs/verification/ui_gameplay.png`, `ui_sprite_showcase.png`, `ui_title.png`,
  and `ui_pause.png`. Gameplay/showcase captures date from August; title/pause
  from July. They are historical evidence, not verification of current HEAD:
  the pictured “SELECT SCOUT TARGET” footer differs from current quick actions.
  `sample_ui.png` is a visual reference, not proof of implemented gameplay.
- No live game, browser, minimum-size, touch, or dense campaign verification
  was performed. No new captures were written. Code-derived geometry and
  interaction findings below are explicitly distinguished from screenshot
  observations. Current capture scenes only seed title/menu, pause, fresh
  gameplay, and sprite showcase; showcase is not a late-game simulation.
- Retain the existing dedicated title/settings/pause flows, visible zoom
  controls, modal input blocking, state-owned action validation, save-failure
  feedback, and guarded exit recovery. No literal toolkit demo/debug labels
  were identified; the issue is dashboard-like composition, not proven
  template provenance. Do not copy `sample_ui.png` as an approved final layout.

### Verified findings — implementation backlog

- [ ] **UI-01 — Recompose normal play around the frontier and one contextual decision.**
  **Screen/files:** Main map, all selections; `src/ui.rs::draw_game_ui`,
  `left_panel_rect`, `map_panel_rect`, `side_panel_rect`, `footer_rect`;
  `src/ui/advisor.rs`; `src/ui/panel.rs`; `README.md`, `gdd.md` §20.
  **Evidence/problem:** Historical gameplay visibly divides attention among
  header, overview, map, inspector, and footer. Current code still draws all
  five persistently. At 1280×720 the allocated map is about 639×552 (50% of
  width, 38% of total area); terrain bleeding beyond that rectangle does not
  create additional selectable play area. Advice appears in both overview and
  footer, actions remaining in header and footer. Permanent crest, watermark,
  headings, and repeated “Active” status consume useful decision space.
  **Change:** First record the UI_STYLE §1 screen brief for exploration,
  settlement management, crisis choice, and campaign planning. Replace the
  permanent overview with a visible, dismissible Realm summary; retain only
  actionable alerts in normal play. Collapse the footer's prose/log regions
  into a retrievable season report (UI-07). Use one selected-site inspector
  with a compact summary and contextual action/details disclosure. Remove
  repeated status labels, decorative watermark/crest space, and redundant
  section headings before enlarging the map into the released space. Keep
  realm totals distinct from local stores when they are needed for a cost.
  **Acceptance:** Normal play has a dominant map, one supporting decision area,
  and quiet utilities: at most 2–3 strongly emphasized regions. Most screen
  area serves the map; the player can identify the selected site and next
  meaningful action immediately. Local danger, shortages, and action costs
  remain visible beside the decision they affect.
  **Verify:** Compare fresh, unknown-site, developed-settlement, and isolated
  settlement captures at 1280×720 and the minimum established in UI-02. Tap
  through selection, expanded details, Realm summary, and return to map.
  **Progress (2026-09-20):** Implemented the map-first composition, dismissible
  Realm summary, compact selected-site panel, and quiet footer utilities. The
  current fresh-campaign evidence is in `ui_gameplay.png`,
  `ui_minimum.png`, and `ui_realm_summary.png`. Unknown, developed, and
  isolated selections plus live tap verification remain pending because the
  native-app UI bridge is unavailable in this environment.

- [ ] **UI-02 — Establish a usable minimum viewport and reflow before shrinking.**
  **Depends on:** UI-01 composition brief; implement alongside its layout.
  **Screen/files:** Gameplay, title/settings/pause, all overlays;
  `src/ui.rs` layout helpers/buttons, `src/ui/menu.rs`,
  `src/ui/panel/settlement.rs::FocusButtonLayout`, `src/ui/map.rs`,
  `src/game.rs::draw`, `README.md`.
  **Evidence/problem (code):** README/GDD do not declare supported viewports.
  `draw` passes actual screen dimensions as virtual dimensions, so 1280×720
  constants do not provide a fixed-resolution fallback. The compact layout
  preserves two sidebars: at 800px width they reserve 470px and leave a 290px
  map. At 390px width the computed map width is negative. Header badges have
  78px minimum widths; main area has a 260px minimum height. Focus buttons are
  25–28px high, route rows 26px on a 24px step, and zoom buttons 30px with 4px
  gaps. Toolkit `touch_area` expansion alone cannot create sufficient space
  between densely packed controls. These are geometry findings, not live
  confirmation of every clipping/tap failure.
  **Change:** Declare 1280×720 as the normal baseline and establish a tested
  minimum, initially evaluating 800×600 and mobile 844×390/390×844 canvases.
  Use a map-first compact layout with a dismissible sheet/separate detail view,
  wrapping essential resources and anchored primary actions. Reflow menus and
  modals by available height; use deliberate scrolling for long content.
  Design non-overlapping targets around the toolkit's 44 CSS-pixel minimum,
  with readable text rather than relying on invisible hit-area enlargement.
  If portrait cannot be supported, provide a readable orientation/size path
  rather than drawing broken geometry, and document the actual minimum.
  **Acceptance:** At declared sizes no negative rectangles, overlapping
  controls, cropped labels, or unreachable actions; touch targets remain
  distinct after DPI scaling. Costs and warnings survive compact disclosure.
  **Verify:** Native and embedded browser canvas at 1280×720, 800×600,
  844×390, and 390×844; 100% and 200% display scaling. Record which sizes
  support play versus a size/orientation message. Tap New Game, inspect,
  change focus, zoom, Pause, Settings/Back, Save/Load, and exit-warning Cancel.
  **Progress (2026-09-20):** Declared 1280×720 as the baseline and
  800×560 as the minimum landscape play area. Captures cover 1280×720,
  800×600, 844×390, and 390×844; the two smaller canvases show the readable
  size/orientation message. The 800×600 pause and blocking-event captures are
  also clean. Browser/DPI/touch interaction checks remain pending.

- [ ] **UI-03 — Separate seasonal decisions from navigation and make primary emphasis contextual.**
  **Depends on:** UI-01/02.
  **Screen/files:** Normal play action area;
  `src/ui/advisor.rs::draw_quick_actions`, `draw_council_footer`;
  `src/ui/panel/site.rs`, `src/ui/panel/settlement.rs`, `src/game.rs`.
  **Evidence/problem (current code):** Pause, Chronicle, Factions, and Advance
  occupy the same four-button group. This directly mixes gameplay with menu
  navigation; the historical screenshot does not show this current group.
  Advance always receives primary tone, even when spending actions on a
  selected site is the current decision; selected overlay/focus controls also
  use primary tone as a state indicator.
  **Change:** Move Pause and report navigation into a quiet utility group
  separated spatially from seasonal/site decisions. Give Advance its own
  labeled season action area with the single actions-remaining readout and
  relevant pending-action warning. Emphasize the contextual site action while
  spending actions and Advance when ready; preserve intentional early advance
  and equally valid choices. Distinguish selected-state styling from action
  emphasis. Keep Save/Load/Settings in Pause, visibly reachable.
  **Acceptance:** A player cannot mistake Pause for a council action; the next
  gameplay step is obvious without following an advisor paragraph. Blocking
  events still prevent advancing; utility navigation remains discoverable.
  **Verify:** Normal/minimum sizes with 2, 1, and 0 actions, an unknown site,
  unaffordable action, and blocking event. Complete a season using taps only,
  then open/close Chronicle, Factions, and Pause without spending an action.
  **Progress (2026-09-20):** Separated Pause, Realm, Chronicle, and Factions
  from the contextual decision summary and gave `Advance Season` its own
  labeled action area with blocking-event feedback. Fresh and blocking-event
  captures are saved at normal and minimum sizes. The required 2/1/0-action
  and tap-only interaction matrix remains pending.

- [ ] **UI-04 — Show truthful resource flow, population change, and site status.**
  **Depends on:** UI-01 decides the readouts' permanent homes; do before polish.
  **Screen/files:** Header, settlement inspector;
  `src/ui.rs::realm_totals`, `realm_flows`, `population_flow`, `draw_header`;
  `src/ui/panel/readouts.rs`, `src/ui/panel/site.rs::draw_selected_site`;
  `src/state/settlement.rs` economy/report queries.
  **Evidence/problem (code, with historical rate labels visible):** Header
  “per turn” production uses `tier_modifier / 10.0`, unlike the simulation's
  full modifier and terrain adjustments, and excludes food consumption.
  Population flow is a fixed 3% estimate, unlike conditional natural growth,
  migration, and famine. The inspector shows base focus output without its
  scope being explicit. Its “Active” badge checks existence of a settlement,
  whereas the subtitle reads actual Active/Lost status. “Clear Skies” is
  unconditional copy rather than a weather-state query.
  **Change:** Remove invented rates; obtain accurately labeled last-season
  deltas or deterministic forecasts from state-owned queries, with production,
  consumption, and uncertain effects distinguished. Clarify total realm vs
  local stock and which source funds each action. Show real settlement status
  once; remove unconditional weather copy unless backed by gameplay state.
  Label base focus yields as base yields or replace them with actual output.
  **Acceptance:** Readouts cannot promise food/population growth while the
  represented calculation is negative. Totals/deltas reconcile with their
  labeled scope; lost settlements never display Active. UI rendering owns no
  separate approximation of simulation rules.
  **Verify:** Normal/minimum captures with capital only, multiple settlements,
  non-default focus/tier, famine, and lost settlement. Compare displayed
  results to the state report after advancing. Add focused integration
  regression coverage in `tests/` for shared forecast/report queries.
  **Progress (2026-09-20):** Header deltas now come from the state-owned
  `LastSeasonFlow` report and are labeled `last season`; initial play shows no
  invented rate. Focus output is explicitly labeled as base output, lost
  settlements no longer show `Active`, and the season-report capture shows a
  negative food delta truthfully. Added a regression case in
  `tests/settlement.rs`. Multiple-settlement, famine, lost-settlement, and
  interactive comparison evidence remain pending.

- [ ] **UI-05 — Make action costs, effects, and blocking reasons inspectable by tap.**
  **Depends on:** UI-01/02 contextual inspector and action layout.
  **Screen/files:** Settlement/site actions, routes, events, campaign controls;
  `src/ui/panel/{site,settlement}.rs`, `src/ui/routes.rs`, `src/ui/event.rs`,
  `src/ui/faction.rs`, `src/ui/style.rs::hover_tooltip`, `src/ui.rs::UiAction`,
  `src/game.rs::apply_gameplay_action`, relevant state action-status queries.
  **Evidence/problem (code):** Upgrade and Wardens details require hover;
  route rows spend resources on release but show destination/level/condition,
  not the build/upgrade verb or cost. Toolkit pointer hover is false for touch.
  Focus/trade/integration/project/institution controls use `status.enabled`
  without exposing their reasons. Event choices color the consequence red
  when disabled but omit `status.reason` and explicit resource requirements.
  Commands dispatch immediately, including the costly actions for which GDD
  §20 specifies a review/confirmation step. Scout/founding already show some
  inline reasons; retain those useful explanations.
  **Change:** Use tap-to-inspect action details with exact council/resource
  costs, funding settlement, prerequisites, expected effects, and risks beside
  an explicit action verb. Disabled options must remain inspectable without
  executing. Add review/confirm/cancel for 2+ action or risky commands per GDD;
  preserve direct execution for clearly explained low-risk commands. Display
  event shortages next to the affected choice. Keep hover as supplementary.
  **Acceptance:** A touch player can explain why an action is blocked and what
  it will spend before committing. Inspecting a route or disabled action
  never consumes resources. Cancel preserves state; confirmation spends once.
  **Verify:** At normal/minimum sizes tap inspect/confirm/cancel for founding,
  upgrade, road, Wardens, trade/integration, project, and event choices. Cover
  no actions, resource shortage, unmet unlock, and enabled states without
  keyboard or hover. Preserve existing action-validation regression tests.
  **Progress (2026-09-20):** Added a shared tap-accessible action review for
  founding, upgrades, roads, Wardens, independent trade/integration, campaign
  projects/institutions, and event choices. Reviews reuse state-owned status
  text, keep blocked options inspectable, and require an explicit confirmation
  before consequential actions execute; Escape and Cancel preserve state.
  Enabled and blocked normal/minimum captures are saved as
  `ui_action_review.png`, `ui_blocked_action_review.png`,
  `ui_minimum_action_review.png`, and `ui_minimum_blocked_action_review.png`.
  Live touch-only confirmation/cancellation across every action remains
  pending because the native-app UI bridge is unavailable in this environment.

- [ ] **UI-06 — Make every known route and active local problem reachable.**
  **Depends on:** UI-01/02/05.
  **Screen/files:** Selected site/settlement; `src/ui/routes.rs::draw_route_section`,
  `src/ui/panel/settlement.rs::active_issue_count`, `draw_existing_settlement`,
  `src/ui/map_sites.rs`, `src/state/road.rs`, issue state/query modules.
  **Evidence/problem (code):** Route rendering uses `.take(2)` without More,
  scrolling, or another complete route-action list; map lines are not picked
  as routes. This hides later links at a junction. The settlement inspector
  reduces active problems to a count badge, so it cannot explain the listed
  problems, their causes, urgency, or available responses.
  **Change:** Keep a concise supply warning in the inspector and add visible
  Routes and Issues disclosures containing all relevant items. Prioritize
  blocked supply/urgent crises, label both route endpoints, and provide
  explicit inspect/action paths from UI-05. For issues show current severity,
  cause, escalation pressure, and applicable responses; distinguish resolved
  history from unresolved state. Never silently drop items to fit a panel.
  **Acceptance:** Every known eligible route at a site is actionable; players
  can find why a settlement is isolated or threatened and what to do next.
  Quiet sites do not gain a permanent empty dashboard.
  **Verify:** Normal/minimum sizes with 3+ known links, damaged/blocked route,
  disconnected settlement, multiple issues, and no issues. Tap each route,
  scroll the list, inspect the last issue, return, and confirm one valid fix.
  **Progress (2026-09-20):** Added a visible Routes & Supply disclosure and a
  Frontier Details view with Routes/Issues tabs. Compact play keeps the inline
  supply summary readable; the full view lists all five known Charter Hall
  links in the capture matrix and exposes Wardens through the same action
  review. Active issues show state, severity, age, trigger cause, and pending
  event status. Current normal/minimum evidence is in
  `ui_frontier_routes.png`, `ui_frontier_issues.png`,
  `ui_minimum_frontier_routes.png`, and `ui_minimum_frontier_issues.png`.
  Damaged/blocked-route, multi-issue, scroll, and live touch verification
  remain pending because those states and the native-app UI bridge were not
  available for interaction testing.

- [ ] **UI-07 — Replace repeated season prose with prioritized feedback and retrievable history.**
  **Depends on:** UI-01/03; coordinate issue links with UI-06.
  **Screen/files:** Season advance, footer, Chronicle;
  `src/game.rs::advance_season`, `notify_season_counts`, `draw_bottom_message`;
  `src/ui/advisor.rs`, `src/ui/panel/chronicle.rs::draw_chronicle_overlay`,
  `src/ui/faction.rs::draw_campaign_controls`, session season-report data.
  **Evidence/problem:** Historical images show repeated counsel and a
  permanent empty season-log heading. Current code repeats last-season detail
  in footer and campaign controls. Notifications use capacity 1, while a
  season enqueues multiple notices in fixed order, allowing a later minor
  notice to replace settlement-loss feedback. Chronicle renders only eight
  entries without paging; its 70px row step exceeds its fixed 556px modal
  body for eight entries (code geometry, not a captured overflow).
  **Change:** Present a short prioritized season result with a visible report
  link; retain detailed results in a scrollable/paged report and Chronicle.
  Rank urgent losses/crises ahead of routine faction changes, aggregate
  simultaneous notices, and link affected places to the map/inspector. Remove
  permanent event prose and duplicated log snippets from normal play. Keep
  unresolved warnings in state UI after notifications expire. Provide access
  to the full retained Chronicle with bounded content and fixed dismissal.
  **Acceptance:** The player sees the important outcome, then returns to calm
  play; every significant result remains recoverable after the toast ends.
  Older entries and all report rows are reachable, readable, and selectable.
  **Verify:** Normal/minimum sizes after a season with loss, shortage, and
  routine changes together; wait for feedback to expire and recover the
  result by tap. Browse 9+ Chronicle entries and an 80-turn history, tap a
  report location, and confirm the overlay closes or reveals its destination.

- [ ] **UI-08 — Give campaign planning its own focus and disclose systems when relevant.**
  **Depends on:** UI-02/05/07.
  **Screen/files:** Factions/campaign overlay; `src/ui/faction.rs`,
  `src/state/campaign.rs`, `src/game.rs` overlay state; `assets/data/` campaign
  definitions/text; `README.md`/`gdd.md` screen brief.
  **Evidence/problem (code):** Factions combines rival stats/recent actions,
  all independents, all wilderness pressures, ambitions, projects,
  institutions, and season rows in one fixed two-column modal. Campaign
  controls receive about 160px of height at 1280×720 but draw beyond that
  allocation. The compact path hides the ambition objective. Independent
  and pressure lists have no knowledge filtering; all projects/institutions
  render regardless of relevance. Campaign strategy is buried under a
  “Factions” entry point and the player receives unrelated systems at once.
  **Change:** Separate frontier inspection from campaign planning using
  clearly named views/disclosures within the existing overlay flow. Move
  season history to UI-07. Make ambition choice a readable comparison with
  objectives before commitment; keep chosen progress/objective accessible.
  Reveal actionable projects/offices at relevant milestones with inspectable
  unlock criteria, and label unknown frontier facts rather than exposing
  undiscovered site details. Preserve an obvious route to future-system help.
  **Acceptance:** Each view has one decision focus, bounded scrolling, and
  readable choices. New players see relevant possibilities; experienced
  players can locate projects and institutions without searching faction
  statistics. Costs/unlocks remain available through UI-05.
  **Verify:** At normal/minimum sizes inspect opening campaign, first unlock,
  ambition chosen, and late campaign with all systems active. Tap view
  switches, compare ambitions, inspect locked content, activate an eligible
  institution, and close. Do not infer unlock rules from GDD examples alone;
  reconcile with implemented balance/state rules.

- [ ] **UI-09 — Teach the opening once and provide visible, reopenable help.**
  **Depends on:** UI-01/03/05 control names and locations.
  **Screen/files:** First-use play, map overlays, help route;
  `src/ui/advisor.rs::recommendation_for`, `src/ui/map.rs::draw_map_caption`,
  `src/state/campaign.rs::guidance_text`, `src/state/advisor.rs`,
  `src/game.rs`, `assets/data/text.json`, `README.md` controls.
  **Evidence/problem:** Historical screenshots show permanent repeated advice
  and drag prose. Current advice is recalculated continuously with no visible
  Help action or completed-instruction dismissal state. Map overlay meanings
  are supplied only through hover. Guidance such as selecting question marks
  does not consistently teach the exact visible control for the next action.
  **Change:** Introduce brief dismissible first-use prompts for selecting a
  rumor, Scout Selected Site, founding, roads, first crisis, independent
  request, and rival activity, driven by actual supported milestones. Retain
  a visible Help control to replay explanations and tap-accessible overlay
  legends. Remove completed instructions from the permanent layout. Name the
  actual drag gesture and zoom buttons, with keyboard shortcuts secondary.
  **Acceptance:** A new touch player can complete the opening without README
  or hover; experienced play is free of permanent tutorial paragraphs. Help
  remains reachable and dismissible without changing campaign state.
  **Verify:** Fresh opening at normal/minimum sizes by taps only; complete and
  dismiss each supported prompt, reopen Help, load a campaign, and verify
  completed prompts do not repeatedly cover gameplay or hide urgent warnings.

### Further inspection — reproduce before declaring additional defects

- [ ] **UI-10 — Verify camera framing, picking, and input ownership in the recomposed map.**
  **Depends on:** UI-01/02; complete before visual sign-off.
  **Screen/files:** Main map at all zoom levels;
  `src/ui/map.rs::MapView::new`, `draw_map_panel`, `draw_overlay_tabs`,
  `src/ui/map_sites.rs::picked_site_id`, map terrain/sprite modules,
  `src/game.rs::new`, `update`, `zoom_map`, `apply_action`.
  **Evidence/uncertainty:** Historical gameplay shows tightly clustered
  frontier markers beside an oversized capital illustration; showcase shows
  overlapping illustrations/labels and edge-clipped sites. Exact current
  selection usability is untested. Projection clamps tile size to 11–25px,
  while camera zoom ranges 0.8–2.6. Mouse camera updates and touch gestures
  are not scoped to the map rectangle in `Game::update`; map picking occurs
  after overlay/zoom controls without an explicit consumed-input guard.
  Picking uses reverse data order, unlike the depth-sorted drawing. New Game
  resets session but not camera target/zoom. These code paths merit targeted
  interaction checks, not an assertion that every gesture is broken.
  **Change:** Reproduce at supported sizes after reflow. Frame the capital
  and reachable frontier initially, reset/focus the camera on new campaigns,
  add a visible recenter control if players can lose the realm, and align
  zoom limits with useful rendered scale. Where reproduced, fix viewport
  clipping, control input consumption, map-only gesture ownership, and
  picking order/marker-label collision. Reuse toolkit camera/pointer helpers.
  **Acceptance:** Players can distinguish and select nearby sites, pan to all
  discovered regions, zoom predictably, and recover their place. Toolbar taps
  never select underlying sites; dragging panels never pans the world.
  **Verify:** Normal/minimum native and browser canvases, 100%/200% DPI;
  capital cluster, map edges, discovered dense area, min/max zoom, resize,
  restart after panning. Exercise mouse drag, one-finger drag, pinch, +/−,
  release over a marker/control, and recenter. Record actual reproductions.

- [ ] **UI-11 — Complete current-build dense-state visual and touch verification.**
  **Depends on:** UI-01–10; capture a baseline before implementation where useful.
  **Screen/files:** All supported screens, especially crisis, long Chronicle,
  factions, endgame, settings and recovery; `src/main.rs` capture harness,
  `src/game.rs::begin_capture_scene`, `src/ui/{event,endgame,menu}.rs`,
  `docs/verification/`, `README.md`.
  **Evidence/uncertainty:** Available captures lack current footer controls,
  small viewports, dense events, faction planning, and endgame. Fixed offsets
  in event choices/defer and endgame prose/footer need visual inspection for
  longest actual content; no visual defect is claimed for those unseen states.
  The existing title image and modal dimming work as clear focal treatments
  in historical 1280×720 captures and should not be redesigned without need.
  **Change:** Exercise supported states through play or extend deterministic
  capture scenes without changing gameplay rules. Verify UI_STYLE §9 after
  composition changes; fix reproduced overflow by reflow/scrolling and stable
  action footers, not by shrinking text. Apply restrained border/type cleanup
  only after layout: retain selection/warning meaning and remove nested
  frames or redundant labels that still compete with the decision.
  **Acceptance:** Current screenshots and interaction notes substantiate the
  attention budget, readable content, reachable controls, and recovery at
  normal/minimum sizes. Distinguish captured visuals, exercised interactions,
  and untested device behavior. No screenshot-only claim of touch compliance.
  **Verify:** Use UI-02's size/DPI matrix. Cover first use, selected/expanded,
  long labels/large values, dense late play, urgent/unaffordable events,
  permitted Defer, endgame Restart/Title, pause/settings, save/load failure,
  and exit-warning Save First/Cancel. Store captures directly in
  `docs/verification/`, replacing equivalent states. Run `./publish.ps1`
  without parameters after meaningful game changes and report its result;
  this documentation-only audit does not require publishing. Keep `.rs`
  files within 800 physical lines and preserve useful tests in `tests/`.
