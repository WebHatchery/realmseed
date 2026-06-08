# Realmseed

Realmseed is a Rust + Macroquad fantasy realm-building prototype. Phase 1
implements the first playable shell: a small strategic map, seasonal turns,
site inspection, scouting unknown adjacent sites, and a chronicle that records
the realm's opening history.

## Current Phase

- 60 x 40 terrain map.
- 6 authored regions.
- 30 total map sites.
- 8 starting visible sites.
- 18 settlement-capable sites.
- 4 independent settlements.
- 8 landmark, resource, pass, ford, ruin, or hazard sites.
- 50 authored route links.
- Toolkit-backed save/load for the current campaign state.

## Controls

- Click a solid marker to inspect a known site.
- Click a question marker, then use `Scout Selected Site` to reveal it.
- Press `Space` or use `Advance Season` to move through Spring, Summer,
  Autumn, and Winter.
- Press `C` or use `Chronicle` to open the chronicle.
- Use right mouse drag and `+ / -` to adjust the map view.
- Use `S / L` to save or load the toolkit campaign slot.

## Validation

Use the project publisher as the validation path:

```powershell
.\publish.ps1
```
