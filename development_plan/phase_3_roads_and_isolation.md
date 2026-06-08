# Phase 3: Roads and Isolation

GDD milestone: Roads and Isolation

## Goal

Make geography matter. Roads should bind settlements into a realm, change trade and loyalty, affect crisis response, and make the player's network visible as a deliberate structure.

## Key Implementation Tasks

- Define route runtime state:
  - site A, site B.
  - road level: none, path, road, stone road.
  - known status.
  - damaged or blocked state.
  - region ids crossed, if relevant.
- Implement road-building and road-upgrade actions:
  - Build Path.
  - Upgrade Path to Road.
  - Upgrade Road to Stone Road.
  - spend council actions and resources.
  - apply terrain or trait cost modifiers.
- Implement graph utilities:
  - connected to capital network.
  - shortest supply distance.
  - disconnected settlements.
  - road network reach overlay data.
- Apply road effects:
  - loyalty bonus for connected settlements.
  - isolation penalty for disconnected settlements.
  - trade/prosperity bonus for connected trade settlements.
  - event response modifiers for connected sites.
- Implement unmanaged strain:
  - calculate strain.
  - calculate unmanaged strain.
  - apply stability, crisis escalation, autonomy, and rival influence pressure.
- Add at least 3 road events or road warnings:
  - bridge washout.
  - caravan attacked.
  - winter blockage or toll dispute.
- Add at least 1 regional project that reduces future road, supply, or wilderness pressure.
- Update map UI:
  - draw roads by level.
  - show damaged or blocked route state.
  - show road network reach from capital.
  - show why a settlement is isolated.
- Add chronicle entries for first road, major road upgrade, isolation loss, and regional project completion.

## Riskiest Technical Unknowns

- Route graph implementation must stay simple but robust enough for supply, events, factions, and UI overlays.
- Visual road readability may be hard if 50 route links are present.
- Supply-distance rules can become opaque if the UI does not explain them well.
- Unmanaged strain may feel punitive if it triggers before players understand roads.
- Regional projects may overlap with actions, decrees, and future institutions unless responsibilities are clear.

## Dependencies

- Depends on Phase 1 route links, known sites, selection, and campaign clock.
- Depends on Phase 2 settlement ownership, resources, loyalty, stability, and focus.
- Phase 4 event weighting will depend on road and isolation state.
- Phase 5 faction targeting will depend on road access, weak settlement detection, and region pressure.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Start a campaign with at least two settlements. Build a path between them, advance seasons, then compare a connected settlement to an isolated one. Upgrade one route, trigger or observe a road warning, and use the map overlay to explain which settlements are connected to the capital."

The phase passes if the tester can:

- Build and upgrade a road between valid sites.
- See resources and council actions spent.
- Visually distinguish path, road, and upgraded road states.
- Identify which settlements are connected to the capital.
- Explain why an isolated settlement is suffering penalties.
- Observe road quality affecting loyalty, trade, or event response.
- Use a regional project to reduce future road, supply, or wilderness pressure.

