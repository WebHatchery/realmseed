# TODO — Realmseed

The six development-plan phases are all in the build; the 20-year prototype
campaign runs end to end. What is left is test and structure work.

## Testing

- Full-campaign season-step replay tests covering event chains, rival pressure,
  settlement growth, road effects, and endings.
- Chronicle snapshot tests so final legacy summaries reflect actual campaign
  history consistently.

## Structure

- Split map query logic from UI commands so scouting, founding, road building,
  trade, and integration can be tested as pure operations.
- Validate authored site, route, event, and faction data before campaign start
  so broken map references surface early instead of mid-run.
