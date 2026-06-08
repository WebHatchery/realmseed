# Phase 6: Complete Prototype Campaign

GDD milestone: Complete Prototype Campaign

## Goal

Turn the previous systems into a complete 20-year prototype campaign with guided onboarding, ambitions, projects, institutions, endings, and a generated chronicle that reflects both emergent pressure and proactive player plans.

## Key Implementation Tasks

- Implement full 20-year campaign flow:
  - 80 seasonal turns.
  - campaign start.
  - campaign end.
  - collapse, partial survival, and strong legacy outcomes.
- Implement guided opening:
  - scouting prompt.
  - founding prompt.
  - first road prompt.
  - first crisis prompt.
  - first independent request prompt.
  - first rival action prompt.
- Implement realm ambitions:
  - primary ambition selection.
  - ambition progress tracking.
  - ambition effects on objectives, event weighting, and ending text.
  - at least 3 usable ambition paths for the prototype.
- Implement proactive projects:
  - at least 1 settlement project.
  - at least 1 regional project.
  - at least 3 total projects.
  - project completion effects and chronicle entries.
- Implement institutions or offices:
  - at least 2 unlockable offices/institutions.
  - repeated pressure reduction.
  - UI display of active institution effects.
- Implement legacy scoring:
  - collapse requirements.
  - settlement, town, city, population, roads, rival, integration, rebellion, famine, and collapse score components.
  - ending bands.
  - identity scores.
- Implement endgame chronicle:
  - worst year.
  - golden year.
  - strongest identity tag.
  - largest settlement.
  - at least one defining event.
  - at least two grouped arcs.
  - at least one proactive player-built legacy when available.
- Implement end season summary:
  - production and consumption.
  - population changes.
  - road changes.
  - faction actions.
  - issue changes.
  - new warnings.
- Add tuning pass:
  - 30-turn validation.
  - full 80-turn campaign validation.
  - difficulty preset values.
  - event repetition checks.
- Add bugfix and usability pass for text fit, disabled reasons, warnings, and map readability.

## Riskiest Technical Unknowns

- End-to-end pacing may still feel too reactive, too punitive, or too quiet.
- Ambitions and projects may not meaningfully change the run unless their effects touch event weights, scoring, and chronicle.
- Chronicle generation may produce correct facts but weak narrative arcs.
- The 80-turn run may be too short to see towns, institutions, faction arcs, and meaningful endings without tuned migration and event pacing.
- UI density may grow beyond what Macroquad immediate-mode panels can comfortably display.
- Debugging full campaign state may be slow without internal inspection tools or deterministic seeds.

## Dependencies

- Depends on every prior milestone.
- Requires stable campaign state from Phases 1-5.
- Requires enough content from Phase 4 to support a complete 80-turn run.
- Requires faction, independent, and wilderness histories from Phase 5 for endings and chronicle arcs.
- Requires proactive systems from this phase to support agency, mastery, and replayability.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Play a complete 20-year campaign. Choose a primary realm ambition when offered. Build at least one project, unlock at least one office or institution, interact with an independent settlement, and respond to the rival at least once. At the end, read the chronicle summary and describe what kind of realm you created."

The phase passes if the tester can:

- Finish, collapse, or partially survive a 20-year campaign.
- Understand the guided opening without developer explanation.
- Declare an ambition and see progress toward it.
- Build at least one proactive project.
- Unlock at least one institution or office.
- See at least 5 chronicle entries reference different settlements or sites in a 30-turn slice.
- See at least 3 active issues create follow-up events in a 30-turn slice.
- See at least 2 event chains reach resolution in a 30-turn slice.
- See at least 1 faction action respond to player weakness in a 30-turn slice.
- Read an end summary naming worst year, golden year, strongest identity tag, largest settlement, and a defining event.
- Identify at least 2 grouped chronicle arcs.
- Point to at least one proactive player-built legacy in the final realm.

