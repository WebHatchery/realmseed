# Phase 4: Event Chains

GDD milestone: Event Chains

## Goal

Make the world push back through reusable event families and active issues. Events should create continuity: warnings become crises, crises escalate or resolve, choices apply memory, and the chronicle records meaningful outcomes.

## Key Implementation Tasks

- Define event family data schema:
  - id, type, trigger conditions.
  - severity levels.
  - opening, follow-up, and resolution template ids.
  - cooldown rules.
  - memory tags.
  - chronicle template ids.
- Define event template data schema:
  - title, narrative text.
  - visible consequences.
  - hidden consequences.
  - choices and choice effects.
  - requirements and blocked-by tags.
- Implement active issue state:
  - Warning.
  - Active.
  - Escalating.
  - Resolution.
  - Dormant.
  - Collapse.
- Implement event trigger and weighting service:
  - settlement stats.
  - road state.
  - unmanaged strain.
  - season.
  - memory tags.
  - cooldowns.
- Implement event choice resolution:
  - resource changes.
  - stat changes.
  - memory tags.
  - active issue updates.
  - follow-up scheduling.
  - chronicle entries.
- Implement repetition controls:
  - local cooldown.
  - global cooldown.
  - max appearances.
  - blocked-by memory tags.
- Add at least 6 skeleton event families:
  - each has 1 opening event, 1 follow-up event, 1 resolution event, and 2 chronicle templates.
- Implement event modal UI:
  - cause, location, severity, choices, visible consequences, uncertainty.
  - defer only where allowed.
- Implement chronicle rendering for notable and major event outcomes.
- Add test/debug controls if needed to force event families for validation.

## Riskiest Technical Unknowns

- Data-driven effects can become too generic or too brittle if designed poorly.
- Event weighting may feel random without clear cause display.
- Follow-up scheduling must avoid repeated or stale events.
- Content authoring may bottleneck implementation unless the schema is quick to iterate.
- Active issue state can become hard to debug without inspection tools.
- Chronicle templates need enough variables to feel specific without becoming a text engine project.

## Dependencies

- Depends on Phase 1 campaign clock, site selection, and chronicle base.
- Depends on Phase 2 settlement resources, stats, focus, population, and memory tags.
- Depends on Phase 3 roads, supply, isolation, unmanaged strain, and regional project state.
- Phase 5 rival and wilderness events will reuse the event and active issue systems.

## Tester Playtest Target

Give the tester this task with no extra explanation:

"Play 30 turns. Respond to some events, ignore at least one warning, and resolve at least two event chains. Keep the chronicle open after major choices and watch for repeated events."

The phase passes if the tester can:

- Encounter events from at least 6 event families.
- See events create active issues.
- See an active issue escalate when ignored.
- See an active issue resolve after a response or outcome.
- Identify memory tags or lasting consequences from event choices.
- Confirm no identical event template appears twice for the same settlement in the 30-turn run.
- Find at least 3 follow-up events and at least 2 resolved chains.
- Explain why at least one event happened.

