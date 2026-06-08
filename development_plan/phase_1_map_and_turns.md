# Phase 1: Map and Turns

GDD milestone: Map and Turns

## Goal

Create the first playable Realmseed shell: a new campaign starts, the player can inspect and scout a small strategic map, seasons advance, and the chronicle records the beginning of the realm.

## Key Implementation Tasks

- Replace template identity strings with Realmseed names in `Cargo.toml`, `assets/data/game_config.json`, `index.html`, and runtime UI copy.
- Define core data schemas for terrain tiles, regions, sites, route links, seasons, campaign clock, and chronicle entries.
- Add initial JSON data files for `terrain.json`, `regions.json`, `sites.json`, `roads.json`, and basic `chronicle_templates.json`.
- Implement campaign state with:
  - current year and season.
  - known and unknown site states.
  - selected site id.
  - chronicle entry list.
  - deterministic campaign seed if practical.
- Build a basic map renderer:
  - terrain backdrop from tile layer.
  - 30 total map sites.
  - at least 8 visible sites on turn 1.
  - visible route links between known sites.
  - selected site highlight.
  - unknown adjacent site markers.
- Implement site selection with mouse input.
- Implement scout action for adjacent unknown sites.
- Implement seasonal turn advancement through Spring, Summer, Autumn, Winter.
- Add minimal selected site panel with name, region, site type, known status, traits, and scout action.
- Add basic chronicle entries for campaign start and scouting discoveries.
- Add save/load compatibility only if it is cheap through existing toolkit persistence; otherwise keep state resettable for this phase.

## Riskiest Technical Unknowns

- Whether the current template UI should be refactored immediately or incrementally adapted.
- How much of `macroquad-toolkit` grid/camera can be reused for a continuous terrain map plus site anchors.
- The right state ownership boundary between map data, runtime campaign state, UI selection, and future simulation services.
- Whether deterministic map/site setup should be fully data-authored now or seeded procedurally later.
- Visual readability of 30 sites and 50 future route links at common browser sizes.

## Dependencies

- No prior gameplay phase dependency.
- Requires current GDD site-count contract:
  - 60 x 40 terrain tiles.
  - 6 regions.
  - 30 total map sites.
  - 8 starting visible sites.
  - 18 settlement-capable sites.
  - 4 independent settlements.
  - 8 landmark/resource/pass sites.
- Should preserve room for Phase 2 settlement state and Phase 3 road levels.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Start a new campaign. Click every visible map marker, identify the capital or charter starting site, scout at least 5 unknown adjacent sites, advance through 6 seasons, and open the chronicle after each scouting action."

The phase passes if the tester can:

- Start a campaign without developer help.
- Select sites and understand which site is selected.
- Tell visible sites from unknown sites.
- Scout unknown adjacent sites.
- See the season/year advance correctly.
- Find chronicle entries for campaign start and scouting discoveries.
- Explain, in their own words, what the map objects are after 10 minutes.

