# Phase 5: Rival Faction

GDD milestone: Rival Faction

## Goal

Make the world move without the player. The rival faction, independent settlements, and wilderness pressure should all create visible, explainable pressure that responds to map state and player weakness.

## Key Implementation Tasks

- Implement rival faction runtime state:
  - personality.
  - needs, confidence, fear, hostility, border pressure, recent losses.
  - current goal and goal age.
  - action cooldowns.
  - controlled locations and known targets.
  - memory tags.
- Implement rival goal scoring:
  - Expand.
  - Fortify.
  - Raid.
  - Trade.
  - Influence.
  - Recover.
  - Confront.
  - Appease.
- Implement rival action execution loop:
  - update faction state.
  - rescore goals every 4 seasons or when invalid.
  - generate legal candidate actions.
  - score targets by value, weakness, region control, road access, risk, and distance.
  - execute the best legal action.
  - log the reason and result.
- Implement concrete rival actions:
  - claim site.
  - raid road or weak settlement.
  - offer trade.
  - support separatists.
  - fortify border.
  - demand withdrawal or close pass.
- Implement abstract conflict resolver if not already added for road/bandit events:
  - raids.
  - road attacks.
  - border disputes.
  - rebellion support.
- Implement independent settlement behavior:
  - trust, autonomy, integration progress, rival pressure, local need.
  - request aid.
  - offer trade.
  - begin integration.
  - resist integration.
  - drift toward rival under pressure.
- Implement wilderness pressure:
  - regional pressure value.
  - pressure bands.
  - seasonal pressure delta.
  - bandit/road/danger event weighting.
  - player reductions through roads, patrols, and fortification.
- Add faction/wilderness UI:
  - recent faction action log.
  - visible personality.
  - action reason text.
  - independent settlement trust/autonomy.
  - wilderness pressure overlay or warning.
- Add chronicle entries for first rival move, first independent request, first integration, major raid, and wilderness escalation or recovery.

## Riskiest Technical Unknowns

- Rival scoring may be too predictable or too chaotic.
- Explaining AI actions clearly without exposing every formula can be difficult.
- Rival actions must reuse existing event/conflict systems instead of creating parallel one-off logic.
- Independent settlement integration can become too passive unless trust/autonomy changes are visible.
- Wilderness pressure can feel like random punishment if regional cause and player counters are unclear.
- Multiple pressure systems may overwhelm the player if event pacing is not controlled.

## Dependencies

- Depends on Phase 1 map, regions, sites, route links, and chronicle.
- Depends on Phase 2 settlement stats, loyalty, stability, defence, and ownership.
- Depends on Phase 3 roads, supply, isolation, region control, and unmanaged strain.
- Depends on Phase 4 event, active issue, memory tag, and chronicle systems.
- Phase 6 endings and identity scoring depend on faction, independent, and wilderness histories.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Play 30 turns. Watch the rival faction log, interact with at least one independent settlement, and try to reduce wilderness pressure in one region. Leave one border or road weakness exposed and observe whether the rival responds."

The phase passes if the tester can:

- Identify the rival faction personality.
- See the rival choose and execute concrete actions.
- Understand why at least one rival action happened.
- Observe the rival claim, raid, trade, or influence.
- See rival behavior change after a player response.
- Receive an independent settlement request.
- Begin trade or integration with an independent settlement.
- Observe wilderness pressure escalate or reduce based on roads, patrols, or fortification.
- Confirm at least one faction action responds to low loyalty, poor road supply, or exposed border.

