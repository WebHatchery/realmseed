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
| Total map sites | 30 |
| Starting visible sites | 8 |
| Settlement-capable sites | 18 |
| Independent settlements | 4 |
| Landmark, resource, or pass sites | 8 |
| Route links | 50 |
| Player settlements | 4 to 8 |
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
| Total map sites | 48 to 60 |
| Settlement-capable sites | 30 to 36 |
| Player settlements | 8 to 14 |
| Rival factions | 2 |
| Independent settlements | 4 to 8 |
| Landmark, resource, or pass sites | 14 to 18 |
| Route links | 80 to 110 |
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

- 30 total map sites.
- 8 starting visible sites.
- 18 settlement-capable sites that can be founded, claimed, or occupied.
- 4 independent settlements, counted as occupied map sites.
- 8 landmark, resource, pass, ford, port, old road, ruin, or hazard sites.
- 50 route links between sites.

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

### Region Control

Each faction has a region control score per region. The highest score defines visual control if it is at least 25 points ahead of the next faction. If no faction clears that margin, the region is contested.

```text
region_control =
    controlled_major_sites * 25
  + controlled_minor_sites * 10
  + road_network_presence * 15
  + local_population_share * 0.2
  + regional_capital_bonus
  + active_patrol_or_watch_bonus
  - active_rebellion_penalty
```

Baseline values:

| Input | Prototype Value |
| --- | ---: |
| Major site controlled | +25 |
| Minor site controlled | +10 |
| Connected road presence | +15 |
| Regional capital controlled | +30 |
| Active patrol or watch decree | +10 |
| Active rebellion in region | -35 |

Region control affects faction action weights, border tension, migration direction, trade safety, and the color or banner treatment shown on the map.

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

### Upgrade Requirements

Tier upgrades require population, settlement health, road access, and resource investment. These numbers are prototype defaults and should be tuned after simulated 20-year runs.

| Upgrade | Requirements | Resource Cost |
| --- | --- | --- |
| Camp to Village | Population 80+, stability 45+ | 80 timber, 30 wealth |
| Village to Town | Population 250+, prosperity 50+, road connection to capital network | 120 timber, 80 stone, 150 wealth |
| Town to City | Population 800+, prosperity 70+, stability 55+, stone road or port connection | 300 stone, 400 wealth |

A settlement cannot upgrade while under active famine, rebellion, occupation, or unresolved severity 3 crisis.

### Population Rules

Food consumed per season:

```text
food_consumed = population * 0.25 * tier_consumption_modifier
```

Population growth:

- If food is secure and stability is above 50: +2 percent per year.
- If food is insecure: no growth.
- If famine is active: -3 percent to -12 percent per year, based on severity.

Natural growth is intentionally slow. The prototype must rely on migration and founding waves to reach midgame tiers inside 80 turns.

| Growth Source | Role | Prototype Rule |
| --- | --- | --- |
| Natural growth | Slow baseline | +0.5 percent per season when food secure and stability > 50 |
| Founding wave | Early boost | Founding a camp moves 30 to 60 population from the capital or migrant pool |
| Prosperity attraction | Medium ongoing growth | +2 to +8 migrants per season when prosperity > 65 and food is secure |
| Safety attraction | Stability-driven migration | +1 to +5 migrants per season when stability > 70 and danger < 35 |
| Refugee event | Burst growth | +20 to +120 population through event chains, with stability or food pressure |
| Crisis flight | Negative migration | -10 to -80 population from famine, raids, rebellion, or forced relocation |

Population transfer must be visible. If founding draws people from an existing settlement, that source settlement loses population and may gain pride, grievance, or frontier-duty memory tags.

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

Background drift should be slow enough to avoid random whiplash. Active issues provide the faster crisis movement.

| Active Issue State | Stability Change Per Season | Loyalty Change Per Season |
| --- | ---: | ---: |
| Warning severity 1 | -1 | 0 |
| Active severity 1 | -1 | -1 |
| Active severity 2 | -3 | -2 |
| Escalating severity 2 | -4 | -3 |
| Escalating severity 3 | -5 | -4 |

Prototype crisis pace target:

- A stable settlement should not collapse from one bad season.
- A neglected severity 2 crisis should become dangerous within 3 to 5 seasons.
- A settlement with poor food, poor road supply, and ignored events should be able to move from safe to rebellion risk within 6 to 10 seasons.
- The UI must show the top causes so the player can understand the arc before collapse.

Example crisis arc:

| Season | State |
| --- | --- |
| 1 | Food drops below threshold; Hungry Winter warning appears |
| 2 | Player ignores warning; issue becomes active severity 1 |
| 3 | Poor road blocks relief; issue becomes active severity 2 |
| 4 | Stability and loyalty penalties stack; migration begins |
| 5 | Player still ignores issue; escalation score crosses threshold |
| 6 | Famine Riot or Winter Graves event triggers |
| 7 to 8 | Resolution creates recovery, grievance, migration, or rebellion follow-up |

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

### First-Pass Resource Costs

These values are rough balance targets, not final tuning. Missing costs are worse than imperfect costs because the economy cannot be tested without resource pressure.

| Action | Food | Timber | Stone | Wealth | Notes |
| --- | ---: | ---: | ---: | ---: | --- |
| Scout nearby site | 0 | 0 | 0 | 10 | Waived for first tutorial scout |
| Found camp | 40 | 60 | 0 | 20 | Also transfers 30 to 60 population |
| Upgrade camp to village | 0 | 80 | 0 | 30 | Requires population and stability gate |
| Upgrade village to town | 0 | 120 | 80 | 150 | Requires road connection |
| Upgrade town to city | 0 | 0 | 300 | 400 | Requires stone road or port |
| Build path | 0 | 40 | 0 | 20 | Terrain can add 0 to 60 percent cost |
| Upgrade path to road | 0 | 80 | 40 | 60 | Bridge or pass traits add cost |
| Upgrade road to stone road | 0 | 60 | 120 | 120 | Strong disaster resistance |
| Fortify village | 0 | 30 | 80 | 50 | Raises defence, may raise local pride |
| Fortify town | 0 | 60 | 160 | 120 | Stronger border deterrent |
| Emergency food aid | 60 | 0 | 0 | 0 | Alternative to wealth aid |
| Emergency wealth aid | 0 | 0 | 0 | 80 | Converts to local relief |
| Negotiate autonomy | 0 | 0 | 0 | 60 | Adds concession memory |
| Suppress rebellion | 0 | 0 | 0 | 120 | Also uses conflict resolver |

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
unmanaged_strain = max(0, strain - council_actions)
```

Unmanaged strain applies at the end of each season:

| Unmanaged Strain Effect | Per Point |
| --- | ---: |
| Crisis escalation chance | +3 percent |
| Realm stability | -1 |
| Autonomy pressure in disconnected or distant settlements | +1 |
| Rival influence action weight against weak settlements | +2 |

If unmanaged strain is 5 or higher for 3 consecutive seasons, generate a realm administration event such as court backlog, corrupt reeves, unpaid road crews, or ignored petitions.

The expected pressure curve:

| Realm Size | Normal Actions | Expected Problems |
| --- | ---: | ---: |
| 1 to 2 settlements | 2 | 1 to 2 |
| 3 to 4 settlements | 3 | 2 to 4 |
| 5 to 7 settlements | 4 | 4 to 7 |
| 8+ settlements | 5 | 6 to 10 |

### Pacing Guardrails

Council capacity is tight, but the first campaign must teach pressure before punishing it.

Opening rules:

- Spring Year 1 grants 1 bonus charter action usable only for scouting, founding the first camp, or building the first path.
- No random severity 3 event can appear before Year 2 unless caused directly by a player choice.
- The first 4 seasons favor warnings and active severity 1 issues over collapse outcomes.
- Tutorial prompts should explain why a problem appeared and which actions can respond to it.

Late-game overwhelm rules:

- A maximum of 2 new non-faction active issues can appear in one season.
- Existing active issues can still escalate, so neglect remains dangerous.
- First Town unlocks a basic delegation action: spend 1 council action and 60 wealth to reduce unmanaged strain by 2 for one season.
- Problems should have different urgency classes: immediate, seasonal, and long-term. Not every warning should demand instant action.

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

Event families mature in tiers. Milestones should use the smallest tier that proves the mechanic.

| Family Tier | Use | Per-Family Content |
| --- | --- | --- |
| Skeleton family | Milestone 4 mechanical proof | 1 opening, 1 follow-up, 1 resolution, 2 chronicle templates |
| Prototype-ready family | Complete 80-turn prototype | 2 openings, 2 follow-ups, 1 resolution, 5 chronicle templates |
| Content-complete family | Polished prototype content pass | 3 openings, 3 follow-ups, 2 resolutions, 10 chronicle templates |

### Event Content Ramp

Milestone 4 does not require the full content-complete target. It only needs enough content to prove triggering, escalation, follow-up, cooldowns, memories, and chronicle output.

| Content Type | Milestone 4 Mechanical Slice |
| --- | ---: |
| Event families | 6 |
| Core event templates | 18 minimum |
| Event variants | 36+ |
| Chronicle templates | 12+ |
| Follow-up links | 12+ |

The complete 80-turn prototype should use all 12 families, but can ship with prototype-ready family depth before the larger content pass.

| Content Type | Complete 80-Turn Prototype |
| --- | ---: |
| Event families | 12 |
| Core event templates | 60 minimum |
| Event variants | 120+ |
| Chronicle templates | 60+ |
| Follow-up links | 36+ |

The content-complete prototype target is a later writing and balance pass, not a blocker for proving the prototype loop.

| Content Type | Content-Complete Prototype |
| --- | ---: |
| Event families | 12 |
| Core event templates | 96 minimum |
| Event variants | 240+ |
| Chronicle templates | 120+ |
| Follow-up links | 60+ |

### Full Seed Content Target

| Content Type | Target |
| --- | ---: |
| Event families | 24 |
| Core event templates | 192 minimum |
| Event variants | 480+ |
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

### Active Issue State Machine

Active issues move through explicit states.

```text
Warning -> Active -> Escalating -> Resolution
             |            |
             v            v
          Dormant      Collapse
```

| State | Meaning | Exit Conditions |
| --- | --- | --- |
| Warning | Risk is visible but not yet a crisis | Trigger improves, or threshold worsens |
| Active | Crisis is ongoing and can be acted on | Player response, timeout, or escalation threshold |
| Escalating | Crisis has worsened and will force stronger outcomes | Severity reduced, resolution event, or collapse |
| Resolution | One-time outcome applies memories, resources, and chronicle entries | Issue closes |
| Dormant | Issue cooled down but leaves memory and future weighting | Cooldown expires |
| Collapse | Settlement loss, rebellion, deaths, occupation, or major damage | Follow-up chain starts |

Each active issue tracks:

- family id.
- severity 1 to 3.
- affected settlement, road, faction, or region.
- age in seasons.
- ignored seasons.
- last player response.
- escalation threshold.
- improvement threshold.
- cooldown after resolution.
- memory tags to apply.

Escalation rule:

```text
escalation_score =
    severity * 10
  + ignored_seasons * 8
  + unmanaged_strain * 3
  + local_risk_modifier
  - relevant_player_response
```

If escalation score is 35 or higher, the issue advances one state or increases severity.

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

### Independent Settlements

Independent settlements are occupied sites that are not initially controlled by the player or rival clan. They use settlement stats, but track trust and autonomy instead of loyalty to the player.

| Stat | Meaning |
| --- | --- |
| Trust | Willingness to trade, request help, accept envoys, or consider integration |
| Autonomy | Desire to remain self-governing |
| Integration progress | Progress toward joining the player's realm |
| Rival pressure | Influence from a rival faction |
| Local need | Current need for food, safety, roads, wealth, or dispute settlement |

Independent settlement actions:

- Request aid during food, road, bandit, or disaster issues.
- Offer trade if roads and trust are high enough.
- Reject integration if autonomy or rival pressure is high.
- Ask for protection when wilderness or rival pressure rises.
- Join the rival if rival pressure is high and player trust is low.

Integration score:

```text
integration_score =
    trust * 0.4
  + road_connection_to_player * 20
  + aid_memory_bonus
  + trade_relationship_bonus
  - autonomy * 0.3
  - rival_pressure * 0.4
  - recent_threat_or_harsh_choice_penalty
```

Integration can begin when the score is 50 or higher. Peaceful integration completes after 3 successful integration seasons or one major integration event resolution. Forced integration is out of scope for the prototype except as a harsh event choice that creates grievance and instability.

### Wilderness Pressure

Wilderness pressure is the second external pressure system beside the rival clan. It is not a political faction; it is a regional danger model that creates bandit, disaster, road, migration, and settlement safety events.

Each region tracks wilderness pressure from 0 to 100.

Pressure increases through:

- High danger terrain.
- Disconnected roads.
- Logging focus near deep forest.
- Unpatrolled passes, forests, and old roads.
- Famine migration and abandoned settlements.
- Rival raids that weaken order.

Pressure decreases through:

- Connected roads.
- Fortified sites.
- Border Watch or patrol projects.
- Settlements with high stability.
- Resolving bandit, road, and disaster event chains.

Seasonal pressure update:

```text
wilderness_pressure_delta =
    average_site_danger * 0.05
  + disconnected_sites * 3
  + active_bandit_issues * 5
  + abandoned_or_ruined_sites * 2
  - connected_fortified_sites * 4
  - active_patrol_projects * 8
```

Pressure bands:

| Pressure | Behavior |
| ---: | --- |
| 0 to 24 | Quiet; only minor flavor and rare travel trouble |
| 25 to 49 | Watchful; road trouble and bandit warnings can appear |
| 50 to 74 | Dangerous; bandit pressure, migration fear, and supply disruption are common |
| 75 to 100 | Lawless; roads can be cut, settlements may lose population, and collapse events can trigger |

Player responses include patrol region, fortify pass, repair road, sponsor watch, send aid, clear bandit camp, negotiate local rights, or abandon an exposed site.

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

Every 4 seasons, a rival faction scores all available goals and chooses the highest score. Scores use 0 to 100 state values unless noted.

```text
resource_need = max(food_need, wealth_need)
target_weakness = 100 - target_defence
open_site_value = best_nearby_open_site_value
```

| Goal | Prototype Score |
| --- | --- |
| Expand | confidence * 0.4 + open_site_value * 0.4 + expansion_need * 0.3 - recent_losses * 0.3 |
| Fortify | security_need * 0.5 + fear_of_player * 0.3 + border_pressure * 0.2 |
| Raid | hostility_to_player * 0.4 + resource_need * 0.3 + target_weakness * 0.3 - fear_of_player * 0.2 |
| Trade | resource_need * 0.4 + (100 - hostility_to_player) * 0.4 + confidence * 0.1 |
| Influence | border_pressure * 0.3 + target_low_loyalty * 0.4 + hostility_to_player * 0.2 |
| Recover | recent_losses * 0.5 + resource_need * 0.2 + security_need * 0.2 |
| Confront | border_pressure * 0.5 + confidence * 0.3 + hostility_to_player * 0.2 - fear_of_player * 0.2 |
| Appease | fear_of_player * 0.5 + (100 - hostility_to_player) * 0.3 + recent_losses * 0.2 |

Tie-breakers:

1. Prefer the current goal if it is within 8 points of the best score.
2. Prefer actions with visible targets over abstract goals.
3. Prefer personality-favored goals.
4. If still tied, choose the lower-risk goal.

### Personality Modifiers

| Personality | Behavior |
| --- | --- |
| Expansionist | +20 Expand, +10 Confront |
| Vengeful | +20 Raid after player harm memory, hostility decays 50 percent slower |
| Mercantile | +20 Trade, -10 Raid unless hostility > 70 |
| Proud | +15 Confront after threats, -15 Appease |
| Cautious | +20 Fortify when fear > 40, -10 Expand after recent losses |
| Opportunist | +20 Raid or Influence against weak or disloyal settlements |
| Traditionalist | -15 Trade and Appease, +15 Fortify, agreement memory lasts longer |

Faction actions must be visible and explainable. The player should be able to understand why a rival supported separatists, claimed a pass, offered trade, or fortified a border.

### Rival Action Execution

The rival chooses a main goal every 4 seasons, but it attempts a concrete action every season if it has a legal target.

Execution loop:

1. Update faction needs, confidence, fear, hostility, and region control.
2. If the current goal is older than 4 seasons or has no legal target, rescore goals.
3. Generate candidate actions from the current goal.
4. Score candidate targets by value, weakness, distance, road access, and risk.
5. Execute the highest-scoring legal action.
6. Emit a visible faction log entry and, for major moves, a chronicle entry.
7. Apply action cooldowns so the same action is not repeated immediately.

Goal-to-action mapping:

| Goal | Concrete Actions |
| --- | --- |
| Expand | Claim open site, settle camp, pressure independent settlement |
| Fortify | Improve border defence, secure road, station clan guard |
| Raid | Attack road, raid weak settlement, steal supplies, test border |
| Trade | Offer trade, request market access, send merchants |
| Influence | Support separatists, fund local speaker, pressure independent village |
| Recover | Repair own road, restore supplies, reduce recent loss penalty |
| Confront | Demand withdrawal, close pass, threaten border settlement |
| Appease | Offer truce, tribute, shared road access, or warning information |

Candidate target score:

```text
target_score =
    target_value * 0.4
  + target_weakness * 0.3
  + region_control_value * 0.2
  + road_access * 0.1
  - action_risk * 0.3
  - distance_penalty
```

Action results should use existing systems. A raid creates a conflict resolution. Influence adds rival support or loyalty pressure. Expansion updates site ownership and region control. Trade creates an event offer or relationship modifier.

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

### Abstract Conflict Resolver

Raids, sieges, border disputes, road attacks, and armed rebellions use the same score procedure.

```text
defender_score =
    defence
  + road_supply_bonus
  + local_loyalty * 0.3
  + stability * 0.2
  + terrain_bonus
  + fortification_bonus
  + relevant_decree_bonus

attacker_score =
    attacker_strength
  + surprise
  + rival_support
  + local_grievance_bonus
  + danger_bonus
  + event_modifier

margin = defender_score - attacker_score
```

| Margin | Outcome |
| ---: | --- |
| +30 or higher | Defender holds cleanly; attacker loses confidence |
| +10 to +29 | Defender holds with minor damage |
| -9 to +9 | Stalemate; issue remains active and may escalate |
| -10 to -29 | Attacker succeeds partially; damage, loyalty loss, or road disruption |
| -30 or lower | Attacker succeeds decisively; occupation, rebellion victory, settlement loss, or major deaths |

Conflict results should expose the top 2 to 3 causes to the player, such as poor road supply, low loyalty, strong hillfort terrain, or rival support.

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
| Famine deaths | -1 per 10 deaths, capped at -100 |
| Realm collapse event | -150 |

`Rival faction contained` means the rival controls no player-founded settlement, controls no more than one contested region, and has not won a decisive conflict in the final 8 seasons.

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

### Identity Scoring

Identity tags use separate 0 to 100 style scores. The final ending displays the top 1 to 3 tags that exceed 45 points.

| Identity | Score Inputs |
| --- | --- |
| Merchant Realm | Wealth surplus, trade focus settlements, connected market roads, peaceful integration |
| Iron Realm | Fortification level, raids defeated, border sites held, low settlement loss |
| Civic Realm | Average loyalty, average stability, negotiated crises, few harsh choices |
| Frontier Realm | Dangerous sites settled, disasters survived, wilderness issues resolved |
| Roadbound Realm | Capital-connected roads, stone roads, low disconnected settlement count |
| Breadbasket Realm | Food surplus, famine prevention, farming settlements supporting others |
| Fractured Realm | Breakaways, autonomy concessions, unresolved rebellions, contested regions |
| Hungry Realm | Famine count, famine deaths, emergency aid failures, food insecurity |

Legacy Score determines survival quality. Identity scores determine the story flavor of that survival.

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

### Summary Generation Passes

Endgame chronicle generation runs in ordered passes:

1. Collect defining events: include all defining entries, top major entries, settlement losses, first town, first city, final rival outcome, and final collapse or survival state.
2. Rank settlements by importance: score founding age, population, prosperity, loyalty extremes, crisis count, rebellion status, road centrality, and chronicle mentions.
3. Identify worst and golden years: sum negative and positive importance values per year, then choose the strongest year with at least 2 notable entries.
4. Assign realm identity tags: use identity scores from the legacy system.
5. Generate opening paragraph: summarize campaign duration, final ending band, capital status, and realm identity.
6. Generate middle paragraph: summarize 2 to 4 defining settlement or faction arcs.
7. Generate ending paragraph: summarize score, losses, survival state, and the strongest legacy tag.
8. Clean repeated phrasing: avoid using the same settlement name twice in one sentence, avoid the same template family twice in one paragraph, and replace repeated subject names with "the town", "the border", "the council", or "the realm" only when the reference is unambiguous.

### Chronicle Arc Rules

The chronicle groups related entries into arcs before generating endgame prose. An arc is a cluster of entries sharing a settlement, faction, road, region, or event family within a meaningful time window.

Prototype arc types:

| Arc Type | Built From |
| --- | --- |
| Rise | Founding, growth, prosperity, upgrade, road connection |
| Rescue | Crisis warning, player aid, recovery, loyalty memory |
| Neglect | Warning, ignored seasons, escalation, grievance |
| Betrayal | Low loyalty, rival influence, tax refusal, rebellion or defection |
| Frontier Trial | Wilderness pressure, road danger, defence, survival or loss |
| Integration | Independent request, aid or trade, integration progress, joining outcome |
| Fall | Famine, raid, rebellion, abandonment, occupation, or collapse |

Chronicle variation rules:

- Similar campaigns should differ through selected arcs, identity tags, ending band, named settlements, and worst/golden years.
- Each ending band has a tone profile: Fallen Charter, Scarred Survival, Fragile Realm, Enduring Realm, and Founding Legend use different opening and closing template pools.
- A barely surviving realm should mention losses, concessions, unresolved danger, and survival cost.
- A thriving realm should mention durable institutions, strong roads, loyal settlements, and defining achievements.
- Template selection weights should prefer entries with high importance, repeated memories, settlement identity tags, or direct player decisions.
- The generator should avoid using the same event family as the main example in both the middle and ending paragraph.

Worst year score:

```text
year_negative_score =
    famine_deaths
  + settlements_lost * 40
  + rebellions_started * 25
  + decisive_conflicts_lost * 30
  + collapse_events * 60
```

Golden year score:

```text
year_positive_score =
    settlements_founded * 15
  + upgrades_completed * 20
  + rebellions_resolved_by_negotiation * 25
  + major_roads_completed * 15
  + decisive_conflicts_won * 25
  + famine_recoveries * 20
```

## 20. User Interface

The interface should feel like a strategic royal map table: readable, calm, and information-dense.

### Main Screens

Prototype UI must be smaller than the full design. The first playable version needs only:

| Screen | Purpose |
| --- | --- |
| Main Map | Terrain backdrop, regions, sites, roads, ownership, warnings, and selection |
| Selected Site Panel | Site or settlement details, valid actions, traits, memories, and active issues |
| Event Modal | Current event narrative, choices, visible consequences, and uncertainty |
| End Season Summary | Production, consumption, faction actions, event changes, and warnings |
| Chronicle Log | Running list of notable and major entries |

Full seed UI can expand into specialized screens:

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
- Prototype overlays are limited to supply, danger, and faction influence.
- Full seed overlays may add fertility, trade reach, unrest, migration, and regional claims after the simulation proves useful.

### Prototype Interaction Flow

The prototype uses immediate actions, not an action queue.

Site action flow:

1. Player selects a site, road, settlement, or event marker on the main map.
2. Selected Site Panel shows current stats, traits, memories, active issues, and valid actions.
3. Each action shows council cost, resource cost, requirements, expected effects, and disabled reasons.
4. Low-risk 1-action commands can execute immediately.
5. Destructive actions, 2+ action commands, and actions with hidden risk require confirmation.
6. On execution, resources and council actions are spent immediately, state changes apply, and a short log entry appears.
7. If an action creates or resolves a notable issue, the chronicle receives an entry.

Event flow:

1. Event Modal opens for blocking events at the start of the player action phase.
2. The modal shows cause, affected location, severity, choices, visible consequences, and uncertainty.
3. The player chooses one option or defers only if the event supports deferral.
4. Deferred events stay active and usually increase ignored seasons.
5. Resolved events apply resources, stats, memory tags, follow-up links, and chronicle entries.

End season flow:

1. End Season button is disabled while a blocking event requires resolution.
2. End Season Summary lists production, consumption, population changes, road changes, faction actions, issue changes, and new warnings.
3. The player can click summary rows to jump to the affected map object.

The UI must always answer three questions: what is wrong, why it happened, and which actions can affect it.

### Onboarding and Difficulty

The first playable campaign should include a guided opening mode.

Guided opening rules:

- First campaign starts with recommended scouting and founding prompts.
- First road, first food warning, first active issue, first independent request, and first rival action each trigger a one-time explanation.
- Tooltips explain stat causes, not just stat names.
- Warnings include plain-language causes such as "poor road supply", "food below winter need", or "rival influence nearby".
- The first 4 seasons introduce one major concept at a time: scouting, founding, supply, and crisis response.

Difficulty presets can initially be data values in `balance.json`.

| Preset | Intended Use | Main Modifiers |
| --- | --- | --- |
| Charter | Learning mode | Lower event weights, slower escalation, extra starting food and timber |
| Frontier | Default mode | Baseline values in this GDD |
| Hard March | Challenge mode | Faster wilderness growth, stronger rival, lower starting wealth |

The prototype should default to Frontier after the guided opening is understood, but Charter should remain available for first-time players.

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

Region data should include:

- id.
- name.
- site ids.
- major site ids.
- regional capital site id, if any.
- controlling faction id.
- contested status.
- per-faction control scores.
- active regional issues.
- unrest level.
- danger level.
- wilderness pressure.
- trade safety.

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
- current goal age.
- action cooldowns.
- last action summary.
- known target ids.
- memory tags.

Independent settlement data should include:

- settlement id.
- trust.
- autonomy.
- integration progress.
- rival pressure.
- local need.
- trade relationship.
- protection relationship.
- integration state.

Wilderness pressure data should include:

- region id.
- pressure.
- pressure band.
- active bandit issues.
- disconnected site count.
- abandoned or ruined site count.
- active patrol projects.
- last pressure delta.

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

Active issue data should include:

- id.
- family id.
- state.
- severity.
- affected scope and target ids.
- age in seasons.
- ignored seasons.
- last player response.
- escalation threshold.
- improvement threshold.
- cooldown remaining.
- memory tags to apply on resolution.
- follow-up event ids.

## 23. Prototype Milestones

### Milestone 1: Map and Turns

Acceptance criteria:

- Player can start a new campaign.
- Map has 30 total map sites, with at least 8 visible on turn 1.
- Site categories match the prototype count: 18 settlement-capable sites, 4 independent settlements, and 8 landmark/resource/pass sites.
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
- Upgrade requirements block invalid camp, village, town, and city upgrades.
- Founding population transfer changes the source population or migrant pool.
- Player can lose a settlement through neglect.

### Milestone 3: Roads and Isolation

Acceptance criteria:

- Player can build roads between connected sites.
- Road quality affects loyalty, trade, and event response.
- Disconnected settlements suffer measurable penalties.
- At least 3 road events can trigger.
- Player can clearly see why a settlement is isolated.
- Unmanaged strain applies stability, crisis, autonomy, and rival influence pressure.

### Milestone 4: Event Chains

Acceptance criteria:

- At least 6 event families exist.
- Each implemented family has at least 1 opening event, 1 follow-up event, 1 resolution event, and 2 chronicle templates.
- Events can create active issues.
- Active issues can escalate or resolve.
- Follow-up events can occur.
- Event choices apply memory tags.
- Chronicle records major outcomes.
- In a 30-turn automated or manual test, no identical event template appears twice for the same settlement.
- At least 3 active issues create follow-up events in a 30-turn test.
- At least 2 event chains reach resolution in a 30-turn test.

### Milestone 5: Rival Faction

Acceptance criteria:

- Rival faction has visible personality.
- Rival chooses goals based on state.
- Rival converts selected goals into concrete map actions.
- Rival can claim sites, raid, trade, or influence settlements.
- Player can understand rival action reasons.
- Rival behavior changes after player actions.
- At least 1 faction action in a 30-turn test responds to a player weakness such as low loyalty, poor road supply, or exposed border.
- At least 1 independent settlement can request aid, trade, and begin integration.
- At least 1 wilderness pressure event can escalate or reduce based on roads, patrols, or fortification.

### Milestone 6: Complete Prototype Campaign

Acceptance criteria:

- Campaign lasts 20 years.
- Player can collapse, partially survive, or earn a strong legacy.
- End summary is generated from actual campaign data.
- At least 12 settlements or locations can appear in the chronicle.
- In a 30-turn test, at least 5 chronicle entries reference different settlements or sites.
- In a 30-turn test, at least 3 active issues create follow-up events.
- In a 30-turn test, at least 2 event chains reach resolution.
- In a 30-turn test, at least 1 faction action responds to player weakness.
- The endgame summary names worst year, golden year, strongest identity tag, largest settlement, and at least one defining event.
- Guided opening prompts explain scouting, founding, roads, first crisis, independent request, and first rival action.
- Chronicle output includes at least 2 grouped arcs, not only isolated log entries.

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

## 25. Design Differentiators

Realmseed is differentiated by the way its systems turn strategic pressure into remembered history.

- Settlement memory: places remember famine aid, neglect, rebellion, autonomy, protection, and betrayal.
- Generated chronicle endings: the campaign summary is built from real events, not fixed ending text.
- Region-level politics: control emerges from sites, roads, population, unrest, and claims rather than painted tiles.
- Road dependency: roads affect supply, loyalty, trade, crisis response, faction pressure, and identity.
- Event chains: crises escalate, cool down, resolve, or collapse through active issues.
- Limited council capacity: the realm grows faster than the player's ability to manage it.
- Identity scoring: different survival styles can produce different realm identities instead of one optimal tidy kingdom.

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
- Build event content in tiers: mechanical slice first, complete prototype second, content-complete pass last.
- Keep resources to four.
- Avoid tactical combat and tile micromanagement.
- Require every future hook to have a current mechanical function.

### Risk: Action Economy Feels Punitive

Mitigation:

- Use opening pacing rules and a Year 1 bonus charter action.
- Prevent early severity 3 random events.
- Add urgency classes so not every warning demands immediate action.
- Unlock delegation after the first town.
- Test 30-turn runs for both frustration and lack of pressure.

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
- Convert goals into concrete map actions with visible targets.
- Explain rival actions in the UI.
- Let past player actions affect faction memory.

### Risk: Wilderness Pressure Is Invisible

Mitigation:

- Track wilderness pressure per region.
- Show pressure bands through warnings and danger overlay.
- Tie pressure changes to roads, patrols, fortifications, abandoned sites, and active issues.
- Ensure wilderness events can both escalate and be reduced by player action.

### Risk: Chronicle Becomes Flavor Only

Mitigation:

- Generate entries from real event data.
- Assign importance levels.
- Use chronicle templates tied to event families.
- Group entries into arcs before writing final summaries.
- Build the endgame summary from tracked campaign facts.

### Risk: Learning Curve Is Too Steep

Mitigation:

- Ship a guided opening mode.
- Use cause-focused tooltips.
- Introduce one major concept at a time during the first 4 seasons.
- Provide difficulty presets through data values.
- Make the UI answer what is wrong, why it happened, and which actions can affect it.

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
