# Realmseed UI subtraction audit — 2026-09-23

Audit only. No gameplay, layout, styling, or interaction code changed. Classifications
refer to the player's phase, not to whether a system has value somewhere in the game.
Repeated instances of the same widget are grouped below; exceptions are explicit.
The implementation backlog is [TODO.md](../TODO.md).

## Evidence and limits

Reviewed current render paths in `src/ui.rs`, all screen/panel modules, map markers,
shared decoration helpers, notifications, relevant text/event data, and endgame summary
construction. Compared existing captures in `docs/verification/`: gameplay, minimum,
Realm summary, routes, issues, campaign, pressure, blocked review, minimum event,
season report, dense Chronicle, Help, title, and pause.

These captures are mixed-revision evidence, not fresh verification of HEAD. For example,
`ui_gameplay.png` has four footer utilities while current code has five including Help;
`ui_pause.png` still shows obsolete per-turn rates. Findings rely on current code when
images differ. Inspected images include 1280×720 and 800×600; README declares 800×560
as the minimum, which was not visually verified here. Unknown/independent selections,
lost/max-tier settlements, settings, exit recovery, and endgame were audited from code.
No live browser, touch, DPI, or fresh capture run was performed. Dense Chronicle and
route fixtures are not evidence of a complete late-game campaign.

This replaces the old TODO's mixed implementation history and broad redesign plan with
remaining subtraction tasks. Already implemented map-first composition, separate views,
truthful last-season rates, action review, history paging, and visible Help are not new
work. Earlier unverified interaction coverage remains unverified: camera picking/input
ownership, resize/DPI, first-use/load behavior, dense late play, and recovery. This audit
does not certify those behaviors or turn hypothetical defects into cleanup tasks.

## Exploration and seasonal decisions

Sources: `src/ui.rs`, `src/ui/map.rs`, `src/ui/map_sites.rs`, `src/ui/advisor.rs`.

| Visible element | Classification | Reason / disposition |
| --- | --- | --- |
| Terrain, roads, site artwork/markers, reachable question marks | Required now | Locate, compare, and select the frontier; artwork also identifies site kind. |
| Region names and selected/major-site labels | Required now | Spatial navigation; a map name and inspector title serve different roles. |
| Minor-site labels revealed by zoom/selection | Contextual | Existing disclosure is appropriate; keep it. |
| Selection ring, ownership/lost-site markers, damaged/blocked road markers | Required now | Identify the action target and immediate hazards; retain non-color cues. |
| Supply rings/route highlights and danger areas | Contextual | Already tied to the chosen overlay; retain. |
| Realm/Supply/Danger tabs, zoom minus/plus | Required now | Visible touch-accessible navigation and mode selection. |
| Overlay hover explanation | Contextual | Keep explanatory content in Help/tap-accessible guidance; never make hover the only home. |
| STRATEGIC VIEW caption | Unnecessary | Describes the obvious map, without conveying mode or state. |
| Permanent drag/zoom caption | Contextual | Teach once, dismiss, and reopen in Help; visible zoom controls remain. |
| Header crest and its dedicated divider | Unnecessary | Non-interactive ornament occupies persistent play space; title screen owns branding. |
| Season/year | Required now | Establishes timing for seasonal decisions and effects. |
| Realm stores label; food/timber/stone/wealth names, icons, values | Required now | Establish resource scope and capacity; labels make icons understandable. |
| Accurately labeled last-season resource/population deltas | Required now | Recent direction informs the next seasonal decision; do not mistake these for forecasts. |
| Realm population | Required now | Realm-wide growth/loss state; local population is a different fact. |
| Header Actions value | Redundant | Footer already shows the same count beside Advance Season; use that home. |
| Footer actions remaining and Advance Season | Required now | Budget and action that advances time, including intentional early advance. |
| Resolve-event warning | Required now | Explains why advancement is blocked. |
| Routine advisor recommendation title | Contextual | Repeated instructions such as revealing frontier need not be wallpaper; first-use guidance/Help owns them. |
| Pause, Realm, Reports, Factions, Help controls | Required now | Existing quiet, visible access to navigation and deferred views; no utility relocation needed. |
| Temporary action/season/save/error notifications | Contextual | Triggered feedback is appropriate; retain retrievable reports and persistent hazard state. |

## Selected site and settlement decisions

Sources: `src/ui/panel.rs`, `src/ui/panel/{site,settlement,readouts}.rs`, `src/ui/routes.rs`.

| Visible element | Classification | Reason / disposition |
| --- | --- | --- |
| SELECTED SITE, SCOUTING, SITE DECISIONS, ACTIONS headings | Unnecessary | Named target and explicit action buttons already describe these areas. |
| Site title, category icon, tier/status subtitle | Required now | Identify the target and available kind of decision; retain one status. |
| Active/Lost badge beside tier/status subtitle | Redundant | Exact status already appears in the subtitle; do not remove Lost itself. |
| Known/Rumor badge for non-settlement sites | Required now | Communicates knowledge limits; keep one knowledge indicator. |
| Rumor detail's Known status and scout reason | Redundant | Knowledge badge and reason beside Scout already convey these facts. |
| No-site prompt | Contextual | Required orientation only when nothing is selected. |
| Region, type, owner, actionable traits | Required now | These can affect expansion and political decisions. |
| Known site's descriptive flavour paragraph | Deferred | Tap/click details can preserve it without crowding decisions. |
| Scout / Found Camp and their costs or blocking reasons | Required now | Primary choices and constraints for these selections. |
| Independent trust, autonomy, rival pressure, integration state, local need | Required now | Directly informs trade and integration while that site is selected. |
| Trade/Integrate controls and inspectable prerequisites | Required now | Relevant choices; disabled affordability does not make them disposable. |
| Selected settlement population, loyalty, stability, danger | Required now | Local health, risk, and upgrade constraints. |
| Local stores label and four resource readouts | Required now | Local funds differ from realm totals even when a one-settlement start makes values equal. |
| Issue-count button | Contextual | Already appears only with active local issues; retain the warning and detail route. |
| Current Focus label and one current-focus name | Required now | Explains the settlement's ongoing production choice. |
| Focus name repeated inside base-output sentence | Redundant | Keep the name once in the summary. |
| Six focus choices and base-yield/notes comparison | Contextual | Expand through a visible Change Focus control when choosing production; keep complete comparison available. |
| Upgrade with a next tier but unmet prerequisites | Required now | Keep inspectable to explain how to progress. |
| Upgrade for lost settlements / maximum tier | Unnecessary | No applicable next-tier action; status and tier already explain the state. |
| Routes & Supply entry and local supply summary | Required now | Preserve route access and isolation/network constraints. |
| Two route preview rows, More Routes sentence, local Wardens shortcut | Deferred | Existing Routes view contains the full route choices and Wardens; retain urgent problems beside the site. |

## Inspection, pressure, and planning

Sources: `src/ui/advisor.rs`, `src/ui/frontier_details.rs`, `src/ui/faction.rs`.

| Visible element | Classification | Reason / disposition |
| --- | --- | --- |
| Realm summary title, Close, charter/ambition identity and crest | Required now | This deliberate realm-identity view is an appropriate home for identity and overview. |
| Realm stability/loyalty/supply/danger bars and values | Required now | Aggregate health is the purpose of this view; bar and number provide magnitude and precision. |
| Known-site, active-settlement and issue counts | Required now | Summarize scale and problems during realm inspection, including zero issues. |
| Realm Advisor's Counsel title/detail | Redundant | Guidance already has Help; remove this extra home when routine footer teaching is contextualized. |
| Frontier title, selected site, Routes/Issues tabs, Close | Required now | Establish scope and allow switching/return. |
| Routes/Issues introductory explanatory sentences | Unnecessary | Describe the labeled content rather than helping the current decision. |
| Route destination, level, condition, labeled next action, Wardens | Required now | Compare connections and act in this dedicated route view. |
| No-known-routes / no-active-issues messages | Contextual | Explain an explicitly opened empty view; do not replace with a blank panel. |
| Repeated ACTIVE ISSUE row heading | Unnecessary | Issues tab and each row's specific state already establish this. |
| Issue cause, state, severity, age, pending-event indicator | Required now | Explain what is happening and which problem requires a response. |
| FRONTIER PRESSURE overline and second CAMPAIGN section heading | Redundant | Repeat the view context; the pressure overline also mislabels Campaign Planning. |
| One current-view title, Pressure/Campaign tabs, Close | Required now | Distinguish inspecting pressure from long-term planning. |
| Rival/Independents/Wilderness group labels and column separation | Required now | Different subjects genuinely need grouping. |
| Rival name, personality/current goal, confidence/fear/hostility/border values | Required now | The explicit pressure view exists to interpret behavior; personality is decision context here. |
| Rival Recent Actions heading, full log entries | Contextual | Expand when investigating behavior; default to current goal and pressure. |
| No rival actions logged paragraph | Unnecessary | Empty historical prose offers no decision value; omit with collapsed history. |
| Independent names and trust/autonomy/rival/integration values | Required now | Compare political positions in this dedicated view. |
| Wilderness region, severity band, exact pressure, delta | Required now | Band interprets risk while value/delta show proximity and direction; they are complementary. |
| Ambition choices / selected ambition, progress, objective | Required now | Campaign decision and progress belong here; never subtract objectives needed before commitment. |
| Season Report shortcut inside Campaign Planning | Redundant | Reports footer already owns this destination; removes an unrelated interruption. |
| Uncompleted projects / inactive institutions and inspectable unlock criteria | Required now | Planning includes future requirements; keep locked choices discoverable. |
| Completed project / activated institution action buttons | Deferred | Move achieved effects to expandable completed details instead of offering spent choices. |

## Events, confirmation, history, and supporting screens

Sources: `src/ui/{event,action_review,season_report,endgame,help,menu}.rs`,
`src/ui/panel/chronicle.rs`, `assets/data/{text,event_templates}.json`,
`src/state/campaign.rs`.

| Visible element | Classification | Reason / disposition |
| --- | --- | --- |
| Council Event identifier, event title/icon, severity, affected-site narrative, cause | Required now | Explain the interruption, target, urgency and problem; narrative is context in this phase. |
| Generic Visible/Uncertain paragraphs repeated across event templates | Deferred | System explanations belong in expandable details/Help; preserve any event-specific risk at the choice. |
| Choice labels, individual consequences, inspectable costs/blockers, permitted Defer | Required now | Essential informed choice and navigation; do not hide uncertainty or unavailable alternatives blindly. |
| Action-review identifier, chosen action, one cost/effect/requirement message | Required now | Establish commitment and its consequences. |
| Blocked reason rendered both in status and body | Redundant | Render once; the code currently interpolates the same reason twice. |
| Ready text and recurring confirmation instruction | Unnecessary | Confirm/Cancel and the actual requirements explain the operation; Help can teach it once. |
| Confirm and Cancel, including disabled Confirm when blocked | Required now | Clear commitment and recovery; preserve blocked explanation. |
| Season Report title/date, categorized rows and detail, Open site, Chronicle, Close | Required now | Reviewing the season is this screen's purpose; zero counts can confirm that no harm occurred. |
| No-season-summary message | Contextual | Explains why an explicitly opened report has no results yet. |
| Chronicle title, dated/tagged entries and bodies, Open site, Close | Required now | Historical facts belong here; tags distinguish kinds, site affordances expose touch navigation. |
| Chronicle paging controls and page count | Contextual | Only needed for multiple pages; hide the one-page/empty pager. |
| Endgame title, score/band, unique legacy facts, Restart and Title | Required now | Reflection and next-session navigation are the phase's purpose. |
| Endgame facts repeated in heading, summary and details | Redundant | Current summary text repeats all six detail fields plus score/band; merge without losing ambition or duration. |
| Help title, subtitle, grouped instructions, current guidance, Escape/visible-controls explanation, Close | Required now | Teaching is explicitly requested here; keep reusable instructions out of normal play. |
| Title art/name, New Game, Continue, Settings, Exit | Required now | Choose a session; branding has an appropriate home. Disabled Continue communicates absent save availability. |
| Paused title, Resume, Save, Load, Settings, Title, Exit | Required now | Utility/recovery phase; prominence here does not clutter active gameplay. |
| Settings title, fullscreen state/toggle, Back | Required now | Minimal existing settings flow. |
| Exit-warning title/reason, Save First, leave-anyway, Cancel | Required now | Prevent accidental loss and allow recovery. |
| Unsupported-viewport title/message/minimum size | Contextual | Essential recovery outside supported dimensions; retain. |

## Shared visual scaffolding

Source: `src/ui/style.rs` and individual readout renderers.

| Visible element | Classification | Reason / disposition |
| --- | --- | --- |
| Modal dimming, panel backgrounds, outer boundaries and meaningful separators | Required now | Isolate the current decision and group different content. |
| Decorative inner panel outline | Unnecessary | Repeats the same boundary without adding grouping. |
| Individual settlement metric-cell borders | Unnecessary | Existing aligned labeled row already groups the metrics. |
| Button outlines, press/hover state, selected tabs/focus, warning cues | Required now | Interaction and selection signals, not disposable decoration. |
| Resource/category icons paired with words; map shading/vignette | Required now | Identification/readability support; no blanket art or icon purge. |
| Debug displays / workflow breadcrumbs | Unnecessary if present | None identified in the reviewed ordinary-play render paths; no invented removal task. |

## Intended boundary for implementation

Nineteen concrete tasks remain in TODO. The largest savings come from focus disclosure,
removing duplicate route controls, dismissing routine teaching, and giving repeated facts
one home. No new dashboard, restyling, camera redesign, or speculative system is proposed.
Disclosure must have visible tap/click access. After implementation, verify affected normal,
800×560 minimum, dense, warning, lost/max-tier and expanded states, including touch-only
navigation and recovery. Do not claim that screenshot inspection proves interaction safety.
