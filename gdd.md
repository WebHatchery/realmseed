# Realmseed Game Design Document

Version: Production Seed v0.2  
Date: 2026-06-08  
Status: Initial production design draft  
Sources: `rough_notes.md`, `README.md`, `GAME_DEVELOPMENT_GUIDE.md`, `CODE_STANDARDS.md`, `MACROQUAD_TOOLKIT.md`

## 1. High Concept

Realmseed is a fantasy realm-building simulation where villages grow into towns, roads become lifelines, factions pressure the frontier, and every decision is recorded as the history of a young kingdom.

The player guides the Charter Council of a newly founded realm in Greenvale, a frontier land of river valleys, forests, hills, old roads, independent settlements, rival clans, and dangerous wilderness. The game is not about controlling individual villagers or placing buildings tile by tile. It is about shaping settlements, roads, regions, faction relationships, crises, and the long memory of a growing realm.

The seed version must prove one core question:

Can a small map, a limited action economy, escalating event chains, one rival faction, and a generated chronicle produce a campaign story worth retelling?

## 2. Product Scope

### Prototype Campaign

The first complete playable target is intentionally smaller than the eventual full seed campaign.

| Category | Prototype Target |
| --- | --- |
| Campaign length | 20 years |
| Seasonal turns | 80 turns |
| Terrain map | 60 x 40 hidden terrain tiles |
| Regions | 6 |
| Strategic sites | 30 |
| Route links | 50 |
| Player settlements | 4 to 8 |
| Independent settlements | 2 to 4 |
| Rival factions | 1 |
| Wilderness pressure systems | 1 |
| Event families | 12 |
| Session length | 30 to 90 minutes |

### Full Seed Campaign

The full seed campaign expands the same systems rather than replacing them.

| Category | Full Seed Target |
| --- | --- |
| Campaign length | 50 years |
| Seasonal turns | 200 turns |
| Strategic locations | 20 to 28 core visible sites, expandable by regions |
| Player settlements | 8 to 14 |
| Rival factions | 2 |
| Independent settlements | 4 to 8 |
| Event families | 24 |
| Crisis chains | Multiple simultaneous chains |

## 3. Design Pillars

### Settlements Are Characters

A settlement is not just a production node. It has needs, strengths, weaknesses, personality tags, grudges, ambitions, memories, and a place in the chronicle. A rich town can become an economic engine and a political problem. A rescued village can stay loyal for decades.

### Simple Systems, Deep Consequences

The seed version uses a small number of resources, stats, and actions, but they must interact often. Food shortage lowers stability. Low stability lowers loyalty. Low loyalty raises rebellion risk. Rebellion damages roads. Damaged roads worsen supply. Poor supply causes more shortages.

### The Map Remembers

The world records what happened. Settlements remember famine aid, neglect, road isolation, rebellion, protection, tax disputes, granted rights, and betrayal. The chronicle turns simulation changes into history.

### The Seed Must Stand Alone

The prototype must be enjoyable without magic, monsters, gods, dynasties, tactical battles, deep diplomacy, or complex city building. Future hooks may exist only if they have a clear current mechanical function.

### Every Future System Attaches to the Core

Future systems should deepen existing objects:

- Tiles define geography.
- Sites define decisions.
- Regions define politics.
- Roads define supply and consequence.
- Factions define pressure.
- Events define story.
- The chronicle defines memory.

## 4. Player Role

The player is the Charter Council, the long-lived governing authority behind the founding of a realm. The council can grant charters, found settlements, sponsor roads, settle disputes, send aid, negotiate, issue decrees, and decide how hard to hold the frontier together.

The player does not:

- Control individual villagers.
- Place buildings freely.
- Assign citizens to jobs tile by tile.
- Move tactical units around a grid.
- Paint borders manually.

The player should think in terms of places, promises, pressure, loyalty, and legacy.

## 5. Game Identity

Realmseed uses terrain tiles, but it must not become a tile empire game.

### Design Boundary

| Tile-Empire Direction | Realmseed Direction |
| --- | --- |
| Every tile is a decision | Every site is a decision |
| Cities work nearby tiles | Settlements inherit regional qualities |
| Workers improve terrain | Councils fund projects and charters |
| Units move across tiles | Expeditions and conflicts move along routes/fronts |
| Borders paint tile by tile | Influence spreads through regions and roads |
| Terrain gives direct yields | Terrain shapes settlement potential |
| Expansion is claiming land | Expansion is founding, integrating, and securing |
| War is unit positioning | War is pressure, sieges, raids, supply, and loyalty |

Core rule:

Tiles are geography. Sites are decisions. Regions are politics. Roads are consequences. Events are story.

Empty terrain can be inspected for information, but major actions happen through sites, settlements, roads, regions, event markers, and faction panels.

## 6. World Structure

Realmseed uses three map layers.

### Tile Layer: Geography

The tile layer creates a continuous fantasy world without making every tile a gameplay object.

Prototype tile target:

- 60 x 40 terrain tiles.
- Each tile stores terrain, fertility, timber, stone, danger, travel cost, and region id.
- Terrain examples: plains, forest, hills, mountain, river, coast, marsh.
- The player rarely interacts with individual tiles.

Tiles determine map shape, settlement potential, road cost, danger, and region identity.

### Site Layer: Decisions

Sites are the main strategic objects.

Prototype site target:

- 30 settlement sites.
- 8 independent settlements.
- 10 to 15 landmarks, passes, fords, ports, old roads, ruins, or resource sites.

A site calculates its qualities from nearby terrain. The player sees a named place such as Redford Crossing, not the individual terrain cells that make it valuable.

Example site:

```text
Redford Crossing
Region: Greenvale Basin
Traits: River Crossing, Fertile Basin
Food potential: High
Timber potential: Low
Stone potential: Low
Trade potential: High
Danger: Moderate
```

### Region Layer: Politics

Regions group sites into political and historical areas.

Prototype region examples:

- Greenvale Basin.
- Blackwood March.
- Stoneback Hills.
- Northwatch Pass.
- Ash Coast.
- Duskvale Lowlands.

Regions matter for faction control, unrest, trade routes, migration, disasters, border pressure, and future expansions.

## 7. Setting

The seed campaign takes place in Greenvale, a fertile but unstable frontier. The Crown grants the player a charter to settle and organize the region, but legitimacy is not control.

Greenvale contains:

- Scattered independent villages.
- At least one independent trade town.
- A rival frontier clan.
- Old roads and ruined fortifications.
- Dangerous wilderness.
- River crossings, passes, hills, forests, and coastlines.
- Useful mysterious sites with immediate mundane functions.

Magic is not active in the prototype. Ancient sites, sacred springs, and deep forests may exist only if they affect current systems such as stone, stability, migration, local identity, danger, or event weighting.

## 8. Campaign Structure

The campaign advances in seasons.

Each year has:

1. Spring.
2. Summer.
3. Autumn.
4. Winter.

### Seasonal Turn Order

1. World update.
2. Settlement production.
3. Food consumption.
4. Population adjustment.
5. Stability and loyalty drift.
6. Road and supply update.
7. Faction action selection.
8. Event and active issue update.
9. Player action phase.
10. Event resolution.
11. Chronicle entry generation.
12. End-of-season warnings and preview.

### Core Loop

Each season, the player:

1. Reviews realm condition.
2. Checks settlement warnings and active issues.
3. Resolves urgent events.
4. Spends limited council actions.
5. Expands, repairs, negotiates, fortifies, or stabilizes.
6. Ends the season.
7. Watches settlements, factions, roads, and events respond.

The player should never have enough capacity to fix everything.

## 9. Settlements

### Settlement Tiers

| Tier | Role |
| --- | --- |
| Camp | Fragile foothold, low output, can become a village with support |
| Village | Basic settlement, produces resources, can specialize |
| Town | Higher population, wealth, ambition, supply needs, and political weight |
| City | Major power center, high productivity, high instability if neglected |

### Settlement Stats

All main stats use 0 to 100 unless noted.

| Stat | Meaning |
| --- | --- |
| Population | Settlement size and labor force |
| Food stored | Local food security, not 0 to 100 |
| Timber stored | Construction and road resource, not 0 to 100 |
| Stone stored | Fortification and advanced road resource, not 0 to 100 |
| Wealth stored | Flexible resource for aid, diplomacy, roads, and events, not 0 to 100 |
| Prosperity | Economic health and long-term growth |
| Stability | Internal order and resistance to crisis |
| Defence | Militia, walls, terrain advantage, and preparedness |
| Loyalty | Willingness to remain in the player's realm |
| Danger | Local wilderness, bandit, disaster, and raid pressure |

### Starting Settlement Baseline

Prototype starting values:

| Stat | Value |
| --- | --- |
| Population | 80 |
| Food stored | 100 |
| Timber stored | 40 |
| Stone stored | 10 |
| Wealth stored | 25 |
| Stability | 65 |
| Loyalty | 70 |
| Defence | 20 |

### Settlement Focuses

Each settlement has one active focus.

| Focus | Food | Timber | Stone | Wealth | Notes |
| --- | ---: | ---: | ---: | ---: | --- |
| Farming | +45 | +5 | +0 | +5 | Population growth, food security |
| Logging | +15 | +35 | +0 | +8 | Strong near forests, raises wilderness tension |
| Quarrying | +10 | +5 | +28 | +8 | Strong in hills, slower growth |
| Trade | +15 | +5 | +0 | +30 | Requires roads, raises local ambition |
| Fortification | +10 | +10 | +8 | +5 | Improves defence, slows economic growth |
| Civic | +20 | +8 | +3 | +10 | Improves stability and loyalty |

Base production is modified by settlement tier, traits, roads, region conditions, events, and decrees.

### Tier Modifiers

| Tier | Production Modifier | Food Consumption Modifier |
| --- | ---: | ---: |
| Camp | 0.5x | 0.8x |
| Village | 1.0x | 1.0x |
| Town | 1.8x | 1.2x |
| City | 3.0x | 1.5x |

### Population Rules

Food consumed per season:

```text
food_consumed = population * 0.25 * tier_consumption_modifier
```

Population growth:

- If food is secure and stability is above 50: +2 percent per year.
- If food is insecure: no growth.
- If famine is active: -3 percent to -12 percent per year, based on severity.

### Loyalty Drift

Per season:

| Condition | Loyalty Change |
| --- | ---: |
| Food secure and stability above 65 | +2 |
| Connected by good road | +1 |
| Food insecure | -2 |
| Disconnected from realm network | -2 |
| Active crisis ignored | -3 |
| Harsh event choice | -5 |
| Major aid during crisis | +5 to +15 |

### Personality Tags

Settlements can gain traits that affect event weights, faction behavior, and chronicle language.

Examples:

- Proud.
- Loyal.
- Restless.
- Wealthy.
- Militarized.
- Isolated.
- Hungry.
- Traditional.
- Opportunistic.
- Overcrowded.
- Frontier-Hardened.
- Trade-Minded.

## 10. Resources

The seed version uses four core resources.

| Resource | Produced By | Consumed By |
| --- | --- | --- |
| Food | Farming settlements, fertile sites, rivers | Population, crisis relief, festivals |
| Timber | Forest sites, logging focus, frontier camps | Camps, roads, basic upgrades, disaster recovery |
| Stone | Hills, uplands, quarrying focus | Fortifications, towns, cities, stone roads |
| Wealth | Trade focus, connected towns, taxes, events | Diplomacy, construction, aid, stabilization, special decisions |

Avoid adding new resources in the prototype unless a system cannot work without them.

## 11. Roads and Supply

Roads connect sites. The player funds a route between two sites; the game calculates cost from terrain crossed. The player does not build road segments tile by tile.

### Road Levels

| Level | Role |
| --- | --- |
| Path | Cheap, unreliable in winter, limited trade |
| Road | Reliable supply and trade route |
| Stone Road | Expensive, strong trade and military value, disaster resistant |

### Supply Distance

Settlements far from the capital or disconnected from good roads suffer:

- Lower loyalty.
- Slower crisis response.
- Reduced trade value.
- Higher danger.
- Higher autonomy pressure.

Roads should feel like strategic commitments. A road can make a settlement rich, expose it to rivals, or bind a distant village to the realm.

## 12. Council Capacity and Actions

The player spends council actions each season. Capacity scales more slowly than realm problems.

### Capacity Rules

| Condition | Actions |
| --- | ---: |
| Base council | 2 |
| Town Hall equivalent exists | +1 |
| Total settlements at least 5 | +1 |
| First city exists | +1 |
| Maximum normal actions | 5 |

Administrative strain is tracked separately:

```text
strain = controlled_settlements + active_crises + disconnected_settlements
```

The expected pressure curve:

| Realm Size | Normal Actions | Expected Problems |
| --- | ---: | ---: |
| 1 to 2 settlements | 2 | 1 to 2 |
| 3 to 4 settlements | 3 | 2 to 4 |
| 5 to 7 settlements | 4 | 4 to 7 |
| 8+ settlements | 5 | 6 to 10 |

### Action Costs

| Action | Cost |
| --- | ---: |
| Scout nearby location | 1 |
| Change settlement focus | 1 |
| Send emergency aid | 1 |
| Negotiate with settlement | 1 |
| Issue decree | 1 |
| Found camp | 2 |
| Upgrade settlement | 2 |
| Build road | 2 |
| Upgrade road | 2 |
| Fortify settlement | 2 |
| Major diplomatic pact | 2 |
| Suppress rebellion | 2 to 3 |

### Player Verbs

Use verbs that reinforce political and historical scale:

- Grant charter.
- Found settlement.
- Sponsor road.
- Secure region.
- Integrate village.
- Settle dispute.
- Fortify pass.
- Open trade route.
- Send expedition.
- Evacuate settlement.
- Negotiate autonomy.
- Establish watch.
- Fund recovery.

Avoid verbs that imply tile micromanagement, such as improve tile, move unit, work tile, buy tile, or build district here.

## 13. Decrees

Decrees are realm-wide temporary policies. Only one decree can be active at a time in the prototype.

| Decree | Benefit | Cost |
| --- | --- | --- |
| Grain Rationing | Reduces food consumption | Lowers stability |
| Road Levy | Reduces road construction cost | Lowers settlement loyalty |
| Border Watch | Improves defence against raids | Costs wealth each season |
| Tax Relief | Improves loyalty | Reduces wealth income |
| Settlement Festival | Improves stability and loyalty | Costs food and wealth |
| Work Charter | Improves construction speed | Raises unrest risk |

## 14. Events and Active Issues

Events are the primary story system. The game should not depend on hundreds of isolated one-off events. Instead, it uses event families with variants, escalation, memory, and chronicle templates.

### Event Family Structure

Each event family includes:

- Trigger conditions.
- Severity levels.
- Opening templates.
- Follow-up templates.
- Resolution templates.
- Variants.
- Memory tags.
- Chronicle templates.
- Local and global cooldowns.
- Escalation rules.
- Blocking tags.

### Prototype Event Families

The prototype needs 12 event families:

1. Food Shortage.
2. Settlement Unrest.
3. Road Trouble.
4. Bandit Pressure.
5. Border Tension.
6. Tax Dispute.
7. Migration.
8. Local Leadership.
9. Resource Boom.
10. Disaster.
11. Independent Settlement Request.
12. Rival Faction Move.

Each family should provide at least:

- 3 severity levels.
- 3 opening events.
- 3 follow-up events.
- 2 resolution events.
- 5 chronicle templates.

### Prototype Content Target

| Content Type | Target |
| --- | ---: |
| Event families | 12 |
| Individual event templates | 60 |
| Event variants | 180+ |
| Chronicle templates | 120+ |
| Follow-up links | 40+ |

### Full Seed Content Target

| Content Type | Target |
| --- | ---: |
| Event families | 24 |
| Individual event templates | 140 |
| Event variants | 400+ |
| Chronicle templates | 250+ |
| Follow-up links | 100+ |

### Active Issue Example

```text
Issue: Food Crisis
Settlement: Blackwood
Severity: 2
Duration: 3 seasons
Tags: winter, poor_roads, low_food
Escalates if: food remains below 20
Improves if: aid sent, road repaired, farming focus active
Expires if: food above 60 for 2 seasons
Outcomes: recovery, grievance, migration, deaths, riot
```

The important behavior is continuity. The game should not only say that Blackwood is hungry. It should remember whether the player ignored the hunger, whether winter worsened it, whether people died, and whether the settlement blamed the council.

### Repetition Controls

Each event template needs:

- Local cooldown.
- Global cooldown.
- Maximum appearances per campaign.
- Required tags or thresholds.
- Blocking memory tags.
- Variant substitution.

Example:

```text
Event: Food Riot
Local cooldown: 12 seasons
Global cooldown: 4 seasons
Max per settlement: 2
Requires: food crisis severity >= 3, stability < 35
Blocked by: recent_riot
Follow-up: crackdown, concession, rebel_speaker
```

## 15. Factions

### Prototype Faction Types

| Type | Role |
| --- | --- |
| Player Realm | Growing charter realm |
| Rival Clan | Organized external pressure |
| Independent Settlements | Existing towns and villages with local agendas |
| Wilderness Pressure | Bandits, danger, harsh terrain, lawless zones, and frontier instability |

### Rival Faction State

Each rival faction tracks:

- Food need.
- Security need.
- Expansion need.
- Wealth need.
- Hostility to player.
- Fear of player.
- Confidence.
- Border pressure.
- Recent losses.
- Current goal.
- Personality.
- Memory tags.

### Goal Selection

Every 4 seasons, a rival faction chooses a main goal.

| Goal | Chosen When |
| --- | --- |
| Expand | High confidence and nearby open sites |
| Fortify | Low security or player nearby |
| Raid | Hostile, resource need, and weak target visible |
| Trade | Low hostility and resource need |
| Influence | Player settlement nearby has low loyalty |
| Recover | Recent loss or crisis |
| Confront | Border pressure high |
| Appease | Fear high and hostility low |

### Personality Modifiers

| Personality | Behavior |
| --- | --- |
| Expansionist | Prefers claiming sites |
| Vengeful | Remembers attacks longer |
| Mercantile | Prefers trade before raids |
| Proud | Reacts strongly to threats |
| Cautious | Fortifies before expanding |
| Opportunist | Targets weak or disloyal settlements |
| Traditionalist | Resists diplomacy but honors agreements |

Faction actions must be visible and explainable. The player should be able to understand why a rival supported separatists, claimed a pass, offered trade, or fortified a border.

## 16. Conflict, Loyalty, and Collapse

Conflict is abstract. There are no tactical battles in the prototype.

### Conflict Sources

- Rival raids.
- Settlement rebellion.
- Border disputes.
- Bandit attacks.
- Road ambushes.
- Independence movements.
- Fort sieges.

### Conflict Inputs

- Attacker strength.
- Defender defence.
- Road access.
- Local stability.
- Local loyalty.
- Faction support.
- Terrain traits.
- Event modifiers.

### Conflict Outcomes

- Defender holds.
- Attacker withdraws.
- Settlement damaged.
- Road damaged or destroyed.
- Settlement occupied.
- Settlement becomes independent.
- Loyalty shifts.
- Population loss.
- New grievance created.

### Rebellion Risk

A rebellion check occurs when:

- Loyalty is below 25.
- Stability is below 40.

Risk score:

```text
rebellion_risk =
    (25 - loyalty)
  + (40 - stability)
  + grievance_count * 8
  + rival_support * 15
  + wealthy_settlement_bonus
  - defence_loyalist_bonus
```

If risk exceeds 50, a rebellion event can trigger.

Rebellion outcomes include tax refusal, armed revolt, negotiated autonomy, breakaway city-state, rival defection, harsh suppression, or long-term resentment.

## 17. Progression

### Realm Stages

| Stage | Description |
| --- | --- |
| Charter Camp | Starting phase, few options, survival focus |
| Settled Valley | Several villages, basic economy, first faction pressure |
| Rising Realm | Towns appear, loyalty and trade matter, rivals react strongly |
| Crowned Power | Realm has scale, but internal instability becomes dangerous |

### Unlock Examples

| Trigger | Unlocks |
| --- | --- |
| 2 settlements | Build roads, assign settlement focus |
| 4 settlements | Issue seasonal decree, open trade with independents |
| First town | Appoint local reeve, fortify, hold realm council |
| First rebellion | Negotiate autonomy, suppress revolt, offer rights |
| First rival border | Send envoy, fortify border, demand withdrawal |

## 18. Legacy, Endings, and Victory

Realmseed should avoid a flat pass/fail victory state. The campaign ends with survival status, Realm Legacy Score, and identity tags.

### Required to Avoid Collapse

The player must end with:

- At least 3 controlled settlements.
- No active realm-wide collapse.
- Population above 250.
- Capital still controlled.

### Legacy Score

| Category | Points |
| --- | ---: |
| Settlements controlled | +10 each |
| Towns | +25 each |
| Cities | +60 each |
| Population | +1 per 25 people |
| Average loyalty above 60 | +50 |
| Average stability above 60 | +50 |
| Roads connected to capital | +10 each |
| Rival faction contained | +75 |
| Independent settlements peacefully integrated | +40 each |
| Rebellions survived | +15 each |
| Rebellions prevented through negotiation | +35 each |
| Settlement lost | -50 each |
| Famine deaths | Negative scaling |
| Realm collapse event | -150 |

### Ending Bands

| Score | Ending |
| ---: | --- |
| Less than 0 | Fallen Charter |
| 0 to 149 | Scarred Survival |
| 150 to 299 | Fragile Realm |
| 300 to 499 | Enduring Realm |
| 500+ | Founding Legend |

### Realm Identity Tags

Assign 1 to 3 identity tags at the end.

| Tag | Condition |
| --- | --- |
| Breadbasket Realm | High food surplus |
| Roadbound Realm | Many upgraded roads |
| Iron Realm | High defence, low loyalty |
| Fractured Realm | Many breakaways |
| Merchant Realm | High wealth |
| Hungry Realm | Repeated famines |
| Border Realm | Many raids or conflicts |
| Civic Realm | High stability and loyalty |

## 19. Chronicle System

The chronicle is a generated campaign history, not a static log.

### Chronicle Entry Data

Every significant event emits a `ChronicleEntry`:

```text
year
season
type
importance
settlement_id
faction_id
tags
template_id
values
```

### Importance Levels

| Level | Use |
| --- | --- |
| Minor | Local flavor, not always shown |
| Notable | Yearly chronicle |
| Major | Campaign history |
| Defining | Endgame summary |

### Template Example

```text
Template ID: famine_major
Condition: food_crisis severity >= 3
Text: In {season} of Year {year}, hunger took hold in {settlement}. {deaths} people died before relief arrived.
```

### Endgame Summary Inputs

The final chronicle calculates:

- First settlement founded.
- Largest settlement.
- Richest settlement.
- Most loyal settlement.
- Least loyal surviving settlement.
- Number of rebellions.
- Number of famines.
- Number of roads built.
- Settlements lost.
- Rival conflicts.
- Worst year.
- Golden year.
- Final realm identity.

## 20. User Interface

The interface should feel like a strategic royal map table: readable, calm, and information-dense.

### Main Screens

| Screen | Purpose |
| --- | --- |
| World Map | Primary play screen with terrain, regions, sites, roads, warnings, and selection |
| Settlement Panel | Tier, population, resources, stats, focus, traits, memories, active issues, actions |
| Realm Overview | Total population, resources, stability, settlements, decree, warnings, trends |
| Event Screen | Narrative, affected location, choices, visible consequences, uncertainty |
| Faction Screen | Known factions, attitude, controlled sites, goals, recent actions, border tension |
| Chronicle | Yearly history, major events, settlement founding dates, conflicts, famines, endings |

### Main Layout Direction

- Map occupies the majority of the screen.
- Resource and season summary sits at the top.
- Realm overview can be narrow or collapsible.
- Selected settlement details can slide in or occupy a right panel.
- Bottom or side actions show only currently valid verbs.
- Warning markers explain cause and urgency.
- Overlays show fertility, danger, trade reach, faction influence, supply, unrest, and migration.

### Visual Style

- Illustrated strategic map rather than literal terrain simulation.
- Continuous terrain with site anchors.
- Inked roads, settlement crests, region labels, faction banners, event markers, and seasonal tinting.
- Muted earth colors with strong contrast for warnings and selected objects.
- No excessive decorative panels. The map should remain the star.

## 21. Art and Audio Direction

### Visual Tone

The world should feel young, earthy, unsettled, and political.

Key moods:

- Frontier hope.
- Seasonal hardship.
- Old mystery with current mechanical value.
- Growing civilization.
- Quiet danger.
- Political tension.

### Settlement Icon Progression

| Tier | Visual |
| --- | --- |
| Camp | Tents and smoke |
| Village | Cottages and fields |
| Town | Market, roads, walls |
| City | Towers, banners, dense buildings |
| Fort | Palisade or stone keep |
| Ruin | Broken stones and overgrowth |

### Audio Target

The seed version can use restrained audio:

- Soft map ambience.
- Seasonal wind and weather.
- Parchment and quill interface sounds.
- Low drums during crisis.
- Settlement sound on selection.
- Bell or horn for major events.

Music should support long strategic play without becoming intrusive.

## 22. Data and Technical Direction

Realmseed is a Rust game using Macroquad and the shared `macroquad-toolkit`.

### Project Constraints

- Use Macroquad for runtime, input, rendering, assets, audio, and timing.
- Use `macroquad-toolkit` for reusable UI, asset loading, camera, grid, notifications, event bus, data loading, and persistence where appropriate.
- Keep game logic in owned state and stateless services rather than embedding simulation behavior inside UI drawing code.
- Keep static data in JSON under `assets/data/` where practical.
- Keep Rust source files below 800 lines.
- Use named Rust module files such as `foo.rs` and `foo/bar.rs`; do not add new `mod.rs` files.

### Expected Data Files

Prototype data should move toward:

```text
assets/data/game_config.json
assets/data/terrain.json
assets/data/regions.json
assets/data/sites.json
assets/data/roads.json
assets/data/settlement_focuses.json
assets/data/event_families.json
assets/data/event_templates.json
assets/data/chronicle_templates.json
assets/data/factions.json
assets/data/decrees.json
assets/data/balance.json
```

### Core State Shapes

Location data should include:

- id.
- name.
- position.
- region id.
- traits.
- current state.
- owner faction.
- connected locations.
- road levels.
- known status.
- danger level.
- settlement id, if occupied.

Settlement data should include:

- id.
- name.
- location id.
- owner faction.
- tier.
- population.
- stored resources.
- prosperity.
- stability.
- defence.
- loyalty.
- danger.
- focus.
- traits.
- memory tags.
- active issues.
- founded year and season.

Faction data should include:

- id.
- name.
- type.
- attitude toward player.
- controlled locations.
- personality.
- needs.
- confidence.
- fear.
- hostility.
- current goal.
- memory tags.

Event data should include:

- id.
- family id.
- title.
- type.
- trigger conditions.
- weight.
- affected scope.
- narrative text.
- choices.
- visible consequences.
- hidden consequences.
- memory tags applied.
- chronicle template id.
- cooldown rules.
- follow-up links.

## 23. Prototype Milestones

### Milestone 1: Map and Turns

Acceptance criteria:

- Player can start a new campaign.
- Map has at least 12 playable sites.
- Player can select sites.
- Player can scout unknown adjacent sites.
- Seasonal turn advances correctly.
- Chronicle records campaign start and scouting discoveries.

### Milestone 2: Settlement Economy

Acceptance criteria:

- Player can found a camp.
- Camps can grow into villages.
- Settlements produce and consume resources each season.
- Food shortages affect stability.
- At least 3 settlement focuses produce different outcomes.
- Player can lose a settlement through neglect.

### Milestone 3: Roads and Isolation

Acceptance criteria:

- Player can build roads between connected sites.
- Road quality affects loyalty, trade, and event response.
- Disconnected settlements suffer measurable penalties.
- At least 3 road events can trigger.
- Player can clearly see why a settlement is isolated.

### Milestone 4: Event Chains

Acceptance criteria:

- At least 6 event families exist.
- Events can create active issues.
- Active issues can escalate or resolve.
- Follow-up events can occur.
- Event choices apply memory tags.
- Chronicle records major outcomes.

### Milestone 5: Rival Faction

Acceptance criteria:

- Rival faction has visible personality.
- Rival chooses goals based on state.
- Rival can claim sites, raid, trade, or influence settlements.
- Player can understand rival action reasons.
- Rival behavior changes after player actions.

### Milestone 6: Complete Prototype Campaign

Acceptance criteria:

- Campaign lasts 20 years.
- Player can collapse, partially survive, or earn a strong legacy.
- End summary is generated from actual campaign data.
- At least 12 settlements or locations can appear in the chronicle.
- Player can name three memorable events after one run.
- No event repeats in a way that feels broken within the first 30 turns.

## 24. Out of Scope for Prototype

The prototype should not include:

- Tactical combat.
- Freeform building placement.
- Tile-by-tile improvements.
- Unit movement on a grid.
- Individual citizen simulation.
- Dynasties, bloodline mechanics, or ruler inheritance.
- Active magic systems.
- Complex religion systems.
- Dozens of resources.
- Procedural world maps with hundreds of active locations.
- Multiplayer.
- Large illustration requirements for every event.

## 25. Competitive Positioning

Realmseed overlaps with grand strategy, settlement simulation, and survival management games, but its center is different.

| Reference | Realmseed Difference |
| --- | --- |
| Crusader Kings | No dynasty focus; settlements and chronicle are the stars |
| Dwarf Fortress | Not a deep colony sim; operates at realm and settlement scale |
| Frostpunk | Similar pressure and moral tradeoffs, but broader map and longer history |
| Northgard | No unit-led RTS economy; expansion is political, logistical, and historical |
| Against the Storm | Shared frontier pressure, but Realmseed emphasizes persistent towns and long-term memory |
| Civilization | Terrain exists, but sites, roads, regions, loyalty, and history are the decision layer |

The pitch should emphasize realm history, settlement memory, road dependency, faction pressure, and endings generated from actual play.

## 26. Design Risks and Mitigations

### Risk: Too Abstract

Mitigation:

- Use strong settlement names.
- Show traits and memories clearly.
- Give events specific local causes.
- Make the chronicle visible early.
- Let map icons change as places grow, starve, rebel, recover, or prosper.

### Risk: Too Much Scope

Mitigation:

- Keep prototype to 80 turns.
- Use 12 event families, not hundreds of bespoke events.
- Keep resources to four.
- Avoid tactical combat and tile micromanagement.
- Require every future hook to have a current mechanical function.

### Risk: Events Feel Random

Mitigation:

- Show event causes.
- Use state-based triggers and weights.
- Use active issues for escalation.
- Apply memory tags from choices.
- Show warnings before major collapse where possible.

### Risk: Factions Feel Fake

Mitigation:

- Give the rival a visible personality.
- Choose goals from needs, confidence, fear, hostility, and opportunity.
- Explain rival actions in the UI.
- Let past player actions affect faction memory.

### Risk: Chronicle Becomes Flavor Only

Mitigation:

- Generate entries from real event data.
- Assign importance levels.
- Use chronicle templates tied to event families.
- Build the endgame summary from tracked campaign facts.

## 27. Success Criteria

The prototype succeeds if players can tell stories like:

- My richest town became my biggest problem.
- I saved a starving village and it stayed loyal for the rest of the campaign.
- I ignored the roads and the border settlements drifted away.
- The rival clan did not destroy me. My own towns nearly did.
- The map at the end looked like the result of my choices.
- I want to replay and found the realm differently.

Final seed test:

After one campaign, does the player remember at least three settlements by name?

If yes, Realmseed has found its core.
