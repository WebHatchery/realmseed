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
