# TODO — Realmseed

- [ ] Migrate all tests and test-only helpers from `src/data/` and `src/state/`
  into `tests/` before expanding coverage. Expose intentional public logic through
  `src/lib.rs`, make `main.rs` use it, and remove test declarations from `src/`.
  Preserve regression coverage and target at most five cases per major feature.
  Correct the source-gate comment in `tests/code_standards.rs`: all physical
  lines count, including tests (CODE_STANDARDS §§2.2, 11).
- [ ] Make gameplay fully touch-accessible: add persistent controls for pause,
  faction management, chronicle access, and advancing seasons; expose save/load
  through the reachable pause menu, provide touch map panning, and add visible
  restart/title actions to the endgame summary. Verify small-browser layouts;
  update advisor instructions, README controls, and `game_page.json` to name the
  actual visible controls and gestures (§7.5).
- [ ] Centralize modal input blocking in `src/game.rs` and `src/ui.rs` so faction,
  event, chronicle, and endgame overlays prevent underlying map/UI actions and
  gameplay shortcuts while retaining explicit close/recovery actions (§7).
- [ ] Extend existing startup validation in `src/data.rs`: reject duplicate IDs,
  invalid site regions/positions/owners, invalid faction controlled-site IDs,
  unsupported road levels/types, and inconsistent event family/stage references.
  Add malformed-data regression cases through the public API (§5.3).
- [ ] Move remaining gameplay balance into typed JSON configuration, including
  rival action/raid effects and integration thresholds in `src/state/faction.rs`
  and ending/identity thresholds in `src/state/campaign.rs`; validate invariants
  and retain toolkit loading (§5.3).
- [ ] Move hardcoded player-facing UI labels, advisor guidance, notifications,
  state messages, and legacy-summary copy into JSON under `assets/`, loaded
  through the toolkit and referenced by stable IDs (§5.3).
- [ ] Extract cohesive responsibilities from long functions such as
  `Game::apply_action`, `GameData::validate`, and advisor recommendation selection.
  Move recommendation rules out of rendering into testable state/query logic;
  split near-limit map rendering and road modules as part of that work, keeping
  functions within 100 lines and every Rust file within 800 (§§2.2, 4.1, 7.1).
- [ ] Make legacy summaries deterministic and tied to structured history:
  replace title-keyword sentiment checks in `chronicle_years`, define a stable
  year tie-break instead of relying on `HashMap` iteration, and add exact summary
  regressions for tied years, losses, and positive events (`src/state/campaign.rs`).
- [ ] Extend the existing 80-turn campaign test into deterministic action replay
  coverage that compares seasonal outcomes for event chains, rival pressure,
  settlement growth, roads, and endings; retain the existing campaign coverage
  rather than adding another endgame-exists smoke test (§11).
- [ ] Make `migrate_save_value` return a clear error for unreadable/unsupported
  saves instead of silently substituting a new campaign. Add migration and
  corrupt-save regressions and preserve the active session on load failure (§6).
- [ ] Remove unused custom-function parameters in `draw_realm_overview` and the
  unused `_loaded_assets` binding; handle asset-pack/texture loading outcomes
  explicitly in `Game::new`. Remove or narrowly justify the undocumented
  crate-wide `clippy::too_many_arguments` suppression (§§1.4, 6, 10.2).
