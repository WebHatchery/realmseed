//! Dispatch UI intents and gameplay commands.

use super::{Game, GameScreen, SettingsReturn};
use crate::state::GameSession;
use crate::ui::{ExitWarningTarget, FactionView, UiAction};
use macroquad::prelude::set_fullscreen;

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::ToggleFocusChoices => {
                self.show_focus_choices = !self.show_focus_choices;
            }
            UiAction::NewGame => {
                self.show_focus_choices = false;
                self.session = GameSession::new(&self.data);
                self.show_chronicle = false;
                self.show_season_report = false;
                self.show_help = false;
                self.chronicle_page = 0;
                self.show_factions = false;
                self.show_realm_summary = false;
                self.show_frontier_details = false;
                self.action_review = None;
                self.screen = GameScreen::Playing;
                self.pending_exit_warning = None;
                self.last_save_at = None;
                self.notifications.info(self.data.text("game.started"));
            }
            UiAction::ContinueGame => {
                if self.load_game() {
                    self.screen = GameScreen::Playing;
                    self.show_chronicle = false;
                    self.show_season_report = false;
                    self.show_help = false;
                    self.chronicle_page = 0;
                    self.show_factions = false;
                    self.show_realm_summary = false;
                    self.show_frontier_details = false;
                    self.action_review = None;
                    self.pending_exit_warning = None;
                }
            }
            UiAction::OpenSettings => {
                let return_to = if matches!(self.screen, GameScreen::PauseMenu) {
                    SettingsReturn::PauseMenu
                } else {
                    SettingsReturn::Title
                };
                self.pending_exit_warning = None;
                self.screen = GameScreen::Settings(return_to);
            }
            UiAction::CloseSettings => {
                self.screen = match self.screen {
                    GameScreen::Settings(SettingsReturn::PauseMenu) => GameScreen::PauseMenu,
                    _ => GameScreen::Title,
                };
            }
            UiAction::OpenPauseMenu => {
                if matches!(self.screen, GameScreen::Playing) {
                    self.pending_exit_warning = None;
                    self.screen = GameScreen::PauseMenu;
                }
            }
            UiAction::ClosePauseMenu => {
                if matches!(self.screen, GameScreen::PauseMenu) {
                    self.pending_exit_warning = None;
                    self.screen = GameScreen::Playing;
                }
            }
            UiAction::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                set_fullscreen(self.fullscreen);
            }
            UiAction::ExitGame => {
                if matches!(self.screen, GameScreen::PauseMenu) {
                    self.request_guarded_exit(ExitWarningTarget::ExitGame);
                } else {
                    macroquad::miniquad::window::request_quit();
                }
            }
            UiAction::ReturnToTitle => {
                self.request_guarded_exit(ExitWarningTarget::Title);
            }
            UiAction::ConfirmPendingExit => {
                self.complete_pending_exit();
            }
            UiAction::CancelPendingExit => {
                self.pending_exit_warning = None;
            }
            UiAction::SaveAndConfirmPendingExit => {
                if self.save_game() {
                    self.complete_pending_exit();
                }
            }
            UiAction::Save => {
                self.save_game();
            }
            UiAction::Load => {
                if self.load_game() && matches!(self.screen, GameScreen::PauseMenu) {
                    self.pending_exit_warning = None;
                    self.show_chronicle = false;
                    self.show_season_report = false;
                    self.show_factions = false;
                    self.show_frontier_details = false;
                    self.action_review = None;
                    self.screen = GameScreen::Playing;
                }
            }
            UiAction::DeleteSave => self.delete_save(),
            UiAction::ToggleChronicle => {
                self.action_review = None;
                self.show_frontier_details = false;
                self.show_chronicle = !self.show_chronicle;
                if self.show_chronicle {
                    self.show_season_report = false;
                    self.chronicle_page = 0;
                    self.show_factions = false;
                    self.show_realm_summary = false;
                }
            }
            UiAction::ToggleSeasonReport => {
                self.action_review = None;
                self.show_season_report = !self.show_season_report;
                if self.show_season_report {
                    self.show_help = false;
                    self.show_chronicle = false;
                    self.show_factions = false;
                    self.show_realm_summary = false;
                    self.show_frontier_details = false;
                }
            }
            UiAction::ToggleHelp => {
                self.action_review = None;
                self.show_help = !self.show_help;
                if self.show_help {
                    self.show_chronicle = false;
                    self.show_season_report = false;
                    self.show_factions = false;
                    self.show_realm_summary = false;
                    self.show_frontier_details = false;
                }
            }
            UiAction::ChroniclePrevious => {
                self.chronicle_page = self.chronicle_page.saturating_sub(1);
            }
            UiAction::ChronicleNext => {
                let page_count = self.session.chronicle.len().div_ceil(5).max(1);
                self.chronicle_page = (self.chronicle_page + 1).min(page_count - 1);
            }
            UiAction::ToggleFactionPanel => {
                self.action_review = None;
                self.show_frontier_details = false;
                self.show_season_report = false;
                self.show_help = false;
                self.show_factions = !self.show_factions;
                if self.show_factions {
                    self.faction_view = FactionView::Pressure;
                    self.show_chronicle = false;
                    self.show_realm_summary = false;
                }
            }
            UiAction::SetFactionView(view) => {
                self.faction_view = view;
            }
            UiAction::ToggleRealmSummary => {
                self.action_review = None;
                self.show_frontier_details = false;
                self.show_season_report = false;
                self.show_help = false;
                self.show_realm_summary = !self.show_realm_summary;
                if self.show_realm_summary {
                    self.show_chronicle = false;
                    self.show_factions = false;
                }
            }
            UiAction::ToggleFrontierDetails => {
                self.action_review = None;
                self.show_frontier_details = !self.show_frontier_details;
                if self.show_frontier_details {
                    self.show_chronicle = false;
                    self.show_season_report = false;
                    self.show_factions = false;
                    self.show_realm_summary = false;
                }
            }
            UiAction::SetFrontierDetailsTab(tab) => {
                self.frontier_details_tab = tab;
            }
            UiAction::OpenActionReview(review) => {
                self.action_review = Some(review);
            }
            UiAction::ConfirmActionReview => {
                if let Some(review) = self.action_review.take() {
                    self.apply_gameplay_action(review.gameplay_action());
                }
            }
            UiAction::CancelActionReview => {
                self.action_review = None;
            }
            UiAction::EndgameNewGame => self.apply_action(UiAction::NewGame),
            UiAction::EndgameReturnToTitle => {
                self.show_chronicle = false;
                self.show_season_report = false;
                self.show_factions = false;
                self.show_realm_summary = false;
                self.show_frontier_details = false;
                self.action_review = None;
                self.pending_exit_warning = None;
                self.screen = GameScreen::Title;
            }
            UiAction::SetMapOverlay(overlay) => {
                self.map_overlay = overlay;
            }
            UiAction::ZoomMapIn => self.zoom_map(1.16),
            UiAction::ZoomMapOut => self.zoom_map(0.86),
            action => self.apply_gameplay_action(action),
        }
    }

    fn apply_gameplay_action(&mut self, action: UiAction) {
        match action {
            UiAction::SelectSite(site_id) => self.select_site(site_id),
            UiAction::ScoutSelectedSite => {
                self.notify_success_with(|session, data| session.scout_selected_site(data))
            }
            UiAction::FoundCamp => {
                self.notify_success_with(|session, data| session.found_selected_camp(data))
            }
            UiAction::UpgradeSelectedSettlement => {
                self.notify_success_with(|session, data| session.upgrade_selected_settlement(data))
            }
            UiAction::SetSettlementFocus(focus_id) => self.notify_info_with(|session, data| {
                session.set_selected_settlement_focus(data, &focus_id)
            }),
            UiAction::BuildOrUpgradeRoute(route_id) => self.notify_success_with(|session, data| {
                session.build_or_upgrade_route(data, &route_id)
            }),
            UiAction::CompleteRegionalProject(region_id) => {
                self.notify_success_with(|session, data| {
                    session.complete_regional_project(data, &region_id)
                })
            }
            UiAction::ResolveEventChoice(choice_id) => self.notify_success_with(|session, data| {
                session.resolve_pending_event_choice(data, &choice_id)
            }),
            UiAction::DeferEvent => {
                self.notify_warning_with(|session, data| session.defer_pending_event(data))
            }
            UiAction::OpenIndependentTrade => self.notify_success_with(|session, data| {
                session.open_trade_with_selected_independent(data)
            }),
            UiAction::BeginIndependentIntegration => {
                self.notify_success_with(|session, data| session.begin_selected_integration(data))
            }
            UiAction::SelectAmbition(ambition_id) => self
                .notify_success_with(|session, data| session.select_ambition(data, &ambition_id)),
            UiAction::CompleteProject(project_id) => self
                .notify_success_with(|session, data| session.complete_project(data, &project_id)),
            UiAction::ActivateInstitution(institution_id) => {
                self.notify_success_with(|session, data| {
                    session.activate_institution(data, &institution_id)
                })
            }
            UiAction::AdvanceSeason => self.advance_season(),
            _ => unreachable!("non-gameplay action routed to gameplay handler"),
        }
    }
}
