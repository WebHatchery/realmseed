//! Deterministic scene setup for visual verification captures.

use super::{Game, GameScreen};
use crate::state::{GameSession, SiteKnowledge};
use crate::ui::ActionReview;

pub(super) fn begin_capture_scene(game: &mut Game, scene: &str) {
    game.capture_sprite_showcase = scene == "sprite_showcase";
    game.show_chronicle = false;
    game.show_factions = false;
    game.show_realm_summary = false;
    game.show_frontier_details = false;
    game.action_review = None;
    match scene {
        "title" | "menu" => {
            game.screen = GameScreen::Title;
        }
        "pause" => {
            game.session = GameSession::new(&game.data);
            game.screen = GameScreen::PauseMenu;
        }
        "sprite_showcase" => {
            game.session = GameSession::new(&game.data);
            let showcase_sites = [
                "charter_hall",
                "lowmeadow",
                "crown_ruins",
                "ironroot_grove",
                "redford_crossing",
                "northwatch_gate",
                "amber_quarry",
                "dusk_mire",
                "saltwind_rocks",
                "seagate",
            ];
            for state in &mut game.session.site_states {
                if showcase_sites.contains(&state.site_id.as_str()) {
                    state.knowledge = SiteKnowledge::Known;
                }
            }
            game.screen = GameScreen::Playing;
        }
        "realm_summary" => {
            game.session = GameSession::new(&game.data);
            game.show_realm_summary = true;
            game.screen = GameScreen::Playing;
        }
        "season_report" => {
            game.session = GameSession::new(&game.data);
            game.session.advance_season(&game.data);
            game.screen = GameScreen::Playing;
        }
        "action_review" => {
            game.session = GameSession::new(&game.data);
            for state in &mut game.session.site_states {
                if state.site_id == "lowmeadow" {
                    state.knowledge = SiteKnowledge::Known;
                }
            }
            game.session.select_site(&game.data, "lowmeadow");
            game.action_review = Some(ActionReview::FoundCamp);
            game.screen = GameScreen::Playing;
        }
        "blocked_review" => {
            game.session = GameSession::new(&game.data);
            game.session.select_site(&game.data, "charter_hall");
            game.action_review = Some(ActionReview::UpgradeSettlement);
            game.screen = GameScreen::Playing;
        }
        "frontier_routes" => {
            game.session = GameSession::new(&game.data);
            let known_sites = [
                "lowmeadow",
                "redfield",
                "willowbend",
                "greenford",
                "redford_crossing",
            ];
            for state in &mut game.session.site_states {
                if known_sites.contains(&state.site_id.as_str()) {
                    state.knowledge = SiteKnowledge::Known;
                }
            }
            game.session.refresh_route_knowledge();
            game.session.select_site(&game.data, "charter_hall");
            game.show_frontier_details = true;
            game.frontier_details_tab = crate::ui::FrontierDetailsTab::Routes;
            game.screen = GameScreen::Playing;
        }
        "frontier_issues" => {
            game.session = GameSession::new(&game.data);
            game.session.select_site(&game.data, "charter_hall");
            let _ = game.session.force_next_event(&game.data);
            game.session.pending_event = None;
            game.show_frontier_details = true;
            game.frontier_details_tab = crate::ui::FrontierDetailsTab::Issues;
            game.screen = GameScreen::Playing;
        }
        _ => {
            // Default: gameplay. Start a fresh campaign so this works on a
            // fresh save with no prior state.
            game.session = GameSession::new(&game.data);
            game.screen = GameScreen::Playing;
        }
    }
}
