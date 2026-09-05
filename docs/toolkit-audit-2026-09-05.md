# Toolkit audit — 5 September 2026

The original review had no named finding. This audit changes all 13 content
loads in `src/data.rs` to labeled toolkit embedded parsing. Typed validation
and site/region lookups remain game-specific.

`src/game.rs` already uses toolkit Camera2D with bounds, AssetManager, EventBus,
notifications, versioned save slots and the save migration callback. Schema
conversion in `src/state.rs` remains local. UI screens use VirtualUi, shared
text/surface helpers and HoverTooltip. Main uses the capture lifecycle.

Coordinate hashes and small terrain-detail selectors generate stable authored
decoration rather than a stateful random stream. No local generic JSON loader,
storage backend, word-wrap loop, sound bank or particle engine was found.

Fixed four unrelated terrain parity lints using `is_multiple_of`, preserving
the decoration choices. Final validation: 27 checks, formatting, strict
all-target/all-feature Clippy and Rust source-size limits. Default `publish.ps1`
passed Windows/WebGL release builds, packaging with ten assets, Preview
deployment and Project Roost tracking.
