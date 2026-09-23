# UI cleanup verification

## Gameplay hierarchy

Reviewed native gameplay captures at 1280×720 (`verification/ui_gameplay.png`)
and 800×560 (`verification/ui_minimum.png`). The site name leads the inspector,
metrics and stores are separated, and all existing actions remain reachable.
The map remains dominant; action budget and pending-event blocking remain in
the footer. Removed the redundant header count, crest, map heading, inspector
headings, inner panel frame, and metric borders.

Chrome preview: started a campaign and selected a rumored site using visible
click controls. The Scout control and its cost remained visible. This is
pointer verification, not physical touchscreen or pinch verification.

Validation: formatting, strict Clippy, 34 existing tests including the physical
source-size gate, and default Windows/WebGL preview publishing passed. Capture
processes exited normally. Tests were grouped across the batch; no new tests
were added for these presentation-only changes.

## Conditional controls and focused overlays

Reviewed all 12 scenes at 1280×720 and 800×560: gameplay, rumored site, lost
settlement, city settlement, single-page Chronicle, dense three-page Chronicle,
affordable action review, blocked upgrade review, Campaign Planning, faction
pressure, five-route frontier details, and active issues. Captures use the
existing `ui_*.png` names and corresponding `ui_minimum_*.png` names directly in
`verification/`; gameplay at minimum size remains `ui_minimum.png`.

Site status appears once, Scout keeps its cost, and lost/city settlements omit
Upgrade. The blocked review keeps one requirement message and disabled Confirm;
affordable review keeps costs. Chronicle navigation remains visible for three
pages and disappears for one. Frontier rows retain state, severity, cause, age,
and route actions. Campaign Planning now has a shorter panel with no duplicate
heading or Reports shortcut. The footer remains the Reports destination.

Chrome preview: inspected the unavailable Upgrade to Town through its visible
control, confirmed a single population requirement, cancelled back to unchanged
resources/action budget, and opened Reports from the footer. Native seeded
captures cover the other states; dense Chronicle page clicks, physical touch,
pinch, and minimum-size browser interactions were not exercised.

Final formatting, strict Clippy, five targeted data/source-size checks, and
default Windows/WebGL preview publishing passed. The complete 34-test suite
passed for the first batch; gameplay rules were not changed in the second.
All shared-harness capture processes exited normally.

## Focus and route disclosure (2026-09-23)

Moved focus comparisons behind Change Focus and route actions/Wardens into
Routes & Supply. Inspector retains current focus, supply state and a visible
count of damaged/blocked routes. Wardens remains reachable without known routes.

Reviewed gameplay, blocked-route inspector and focus comparison at 1280x720
and 800x560, plus the five-route overlay at both sizes. Evidence is stored
in verification/ui_gameplay.png, ui_minimum.png and the route_warning,
focus_choices and frontier_routes normal/minimum pairs. The compact inspector
now fits its supply text inside the panel. Browser pointer checks opened routes,
reviewed Wardens, opened Change Focus and selected Farming; the overlay closed
and the inspector updated without spending an action (the configured cost is 0).
Physical touch and pinch were not exercised.

Formatting, strict Clippy, the 34-test suite and source-size checks passed
across this batch. Default publishing completed Windows/WebGL builds and preview
deployment. Final focus-copy deduplication was included in WebGL publishing and
native verification captures; the next default publish will refresh the Windows
package with that copy. All shared capture processes exited normally.

## Guidance and records disclosure (2026-09-23)

Added dismissible routine guidance, site and event details, rival history,
completed project effects, and a deduplicated ending summary. Blockers and
current decision facts remain visible. Project completion checks are scoped to
the selected target and covered by a regression test.

The grouped 35-test suite passed before the final presentation refinements.
Final formatting, strict Clippy, source-size/data checks and default publishing
passed. Windows and WebGL preview packages now include the final changes,
including the focus-copy update noted above. The shared capture harness generated
11 scenes at each of 1280x720 and 800x560; all game processes exited.

Final visual inspection and browser/touch interaction review remain outstanding
in TODO.md because the user requested updating the list, committing and stopping.
Earlier iterations were inspected, but the final compact inspector refinements
and new browser disclosure interactions have not completed that review.
