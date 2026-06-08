# Phase 2: Settlement Economy

GDD milestone: Settlement Economy

## Goal

Make settlements function as stateful places. The player can found a camp, grow it into a village, assign focuses, produce and consume resources, and see neglect create real settlement trouble.

## Key Implementation Tasks

- Define settlement runtime state:
  - id, name, location id, owner faction.
  - tier, population, stored resources.
  - prosperity, stability, defence, loyalty, danger.
  - focus, traits, memory tags, active issue ids.
  - founded year and season.
- Add balance data for starting resources, production, consumption, tier modifiers, founding costs, and upgrade requirements.
- Implement Found Camp:
  - validate settlement-capable site.
  - spend council actions and resources.
  - transfer population from capital or migrant pool.
  - create settlement and chronicle entry.
- Implement tier upgrade validation:
  - Camp to Village.
  - Village to Town.
  - Town to City gate stub, even if city is unreachable in normal Phase 2 play.
- Implement seasonal settlement update:
  - resource production by focus.
  - food consumption by population and tier.
  - food shortage detection.
  - stability and loyalty drift.
  - population growth, migration, and loss stubs.
- Implement at least 3 settlement focuses with distinct outcomes:
  - Farming.
  - Logging.
  - Civic or Trade.
- Add selected settlement panel:
  - tier, population, food, timber, stone, wealth.
  - stability, loyalty, prosperity, defence, danger.
  - current focus and focus-change action.
  - founding and upgrade actions with disabled reasons.
- Add chronicle entries for founding, upgrade, starvation, and settlement loss.
- Implement a basic settlement loss condition through neglect, such as famine collapse or abandonment.

## Riskiest Technical Unknowns

- Initial balance may fail to create meaningful choices: either all settlements thrive or everything collapses too quickly.
- The UI may become dense before the player understands why resources changed.
- Population transfer can feel punitive if it weakens the source settlement too much.
- Upgrade gates may be unreachable in an 80-turn campaign without migration tuning.
- Storing resource values per settlement versus realm pool may complicate later road and supply systems.

## Dependencies

- Depends on Phase 1 site selection, campaign clock, known sites, and chronicle.
- Requires settlement-capable site metadata from Phase 1.
- Should keep resource and settlement update logic separate from UI rendering for later event and faction systems.
- Road connection checks can be stubbed in this phase but must not be hardcoded in a way that blocks Phase 3.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Start a new campaign. Found one camp, advance seasons until it can become a village, change its focus at least twice, then intentionally create or observe a food shortage. Try to upgrade a settlement before it meets the requirements and note whether the game explains why it is blocked."

The phase passes if the tester can:

- Found a camp on a valid site.
- See resources spent and population transferred.
- Upgrade a camp into a village when requirements are met.
- Understand why an invalid upgrade is blocked.
- Identify at least 3 settlement focuses and describe how their outputs differ.
- Observe food consumption and shortage effects.
- Produce or witness a settlement decline/loss path through neglect.

