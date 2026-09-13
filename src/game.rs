//! High-level game loop, state transitions, and toolkit integration.

use crate::data::GameData;
use crate::state::{migrate_save_value, GameSession, SaveData, SiteKnowledge};
use crate::ui::{
    self, ExitWarningTarget, MapOverlay, MapSpriteTextures, MenuContext, PauseMenuContext,
    UiAction, UiContext,
};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{Camera2D, Camera2DConfig, CameraBounds};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    draw_notification, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    delete_slot, get_save_slots, load_from_slot_with_migration, save_to_slot_with_version,
    slot_exists,
};
use macroquad_toolkit::prelude::{
    begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState, Pointer, TouchGesture,
};
use macroquad_toolkit::ui::HoverTooltip;

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    camera: Camera2D,
    hover_tooltip: HoverTooltip,
    events: EventBus<UiAction>,
    save_exists: bool,
    save_slots: Vec<String>,
    show_chronicle: bool,
    show_factions: bool,
    capture_sprite_showcase: bool,
    map_overlay: MapOverlay,
    screen: GameScreen,
    fullscreen: bool,
    pending_exit_warning: Option<ExitWarningTarget>,
    last_save_at: Option<f64>,
    touch_gesture: TouchGesture,
    touch_claimed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameScreen {
    Title,
    Playing,
    PauseMenu,
    Settings(SettingsReturn),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsReturn {
    Title,
    PauseMenu,
}

const SAVE_WARNING_WINDOW_SECS: f64 = 60.0;

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Realmseed embedded data failed to load: {}", err);
        });

        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.75, 0.2, 0.8, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        match assets.load_asset_pack("assets.zip").await {
            Ok(file_count) => println!("Realmseed asset pack loaded: {file_count} files"),
            Err(error) => eprintln!("Realmseed asset pack unavailable; using fallbacks: {error}"),
        }
        let loaded_textures = assets.load_texture_configs(&data.texture_manifest).await;
        if loaded_textures != data.texture_manifest.len() {
            eprintln!(
                "Realmseed loaded {loaded_textures}/{} configured textures; missing assets use the placeholder",
                data.texture_manifest.len()
            );
        }

        let notifications = NotificationManager::with_settings(1, 3.0);

        let session = GameSession::new(&data);
        let camera = Camera2D::with_config(
            vec2(0.0, 0.0),
            1.25,
            Camera2DConfig {
                drag_button: Some(MouseButton::Right),
                min_zoom: 0.80,
                max_zoom: 2.60,
                bounds: Some(CameraBounds::new(vec2(-520.0, -360.0), vec2(520.0, 360.0))),
                pan_speed: 240.0,
                ..Default::default()
            },
        );

        let mut game = Self {
            data,
            session,
            assets,
            notifications,
            camera,
            hover_tooltip: HoverTooltip::new(),
            events: EventBus::new(),
            save_exists: false,
            save_slots: Vec::new(),
            show_chronicle: false,
            show_factions: false,
            capture_sprite_showcase: false,
            map_overlay: MapOverlay::Realm,
            screen: GameScreen::Title,
            fullscreen: false,
            pending_exit_warning: None,
            last_save_at: None,
            touch_gesture: TouchGesture::new(),
            touch_claimed: false,
        };
        game.refresh_save_state();
        game
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_sprite_showcase = scene == "sprite_showcase";
        match scene {
            "title" | "menu" => {
                self.screen = GameScreen::Title;
            }
            "pause" => {
                self.session = GameSession::new(&self.data);
                self.screen = GameScreen::PauseMenu;
            }
            "sprite_showcase" => {
                self.session = GameSession::new(&self.data);
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
                for state in &mut self.session.site_states {
                    if showcase_sites.contains(&state.site_id.as_str()) {
                        state.knowledge = SiteKnowledge::Known;
                    }
                }
                self.screen = GameScreen::Playing;
            }
            _ => {
                // Default: gameplay. Start a fresh campaign so this works on a
                // fresh save with no prior state.
                self.session = GameSession::new(&self.data);
                self.screen = GameScreen::Playing;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);

        let input = InputState::capture();
        let touch_frame = self.touch_gesture.update();
        self.touch_claimed = touch_frame.claimed;

        match self.screen {
            GameScreen::Title => {
                if is_key_pressed(KeyCode::N) {
                    self.events.push(UiAction::NewGame);
                }
                if is_key_pressed(KeyCode::L) && self.save_exists {
                    self.events.push(UiAction::ContinueGame);
                }
            }
            GameScreen::Settings(_) => {
                if input.escape_pressed {
                    self.events.push(UiAction::CloseSettings);
                }
            }
            GameScreen::Playing => {
                let modal_open = self.session.pending_event.is_some()
                    || self.session.endgame_summary.is_some()
                    || self.show_chronicle
                    || self.show_factions;
                if input.escape_pressed && self.session.pending_event.is_none() {
                    if self.session.endgame_summary.is_some() {
                        // The completed campaign keeps its visible recovery controls.
                    } else if self.show_factions {
                        self.events.push(UiAction::ToggleFactionPanel);
                    } else if self.show_chronicle {
                        self.events.push(UiAction::ToggleChronicle);
                    } else {
                        self.events.push(UiAction::OpenPauseMenu);
                    }
                } else if !modal_open {
                    if touch_frame.claimed {
                        self.camera
                            .pan(-touch_frame.pan / (self.camera.zoom * 0.92).max(0.0001));
                        if (touch_frame.scale - 1.0).abs() > f32::EPSILON {
                            self.zoom_map(touch_frame.scale);
                        }
                    }
                    if input.space_pressed {
                        self.events.push(UiAction::AdvanceSeason);
                    }
                    if is_key_pressed(KeyCode::C) {
                        self.events.push(UiAction::ToggleChronicle);
                    }
                    if is_key_pressed(KeyCode::F) {
                        self.events.push(UiAction::ToggleFactionPanel);
                    }
                    if is_key_pressed(KeyCode::N) {
                        self.events.push(UiAction::NewGame);
                    }
                    if is_key_pressed(KeyCode::S) {
                        self.events.push(UiAction::Save);
                    }
                    if is_key_pressed(KeyCode::L) {
                        self.events.push(UiAction::Load);
                    }
                    if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::Delete) {
                        self.events.push(UiAction::DeleteSave);
                    }
                    self.camera.update(dt, false);
                }
            }
            GameScreen::PauseMenu => {
                if input.escape_pressed {
                    if self.pending_exit_warning.is_some() {
                        self.events.push(UiAction::CancelPendingExit);
                    } else {
                        self.events.push(UiAction::ClosePauseMenu);
                    }
                }
            }
        }

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);
        let virtual_ui = begin_virtual_ui_frame(screen_width().max(1.0), screen_height().max(1.0));
        let pointer = Pointer::read(|point| virtual_ui.screen_to_ui(point));
        let actions = match self.screen {
            GameScreen::Title => ui::draw_title_menu(MenuContext {
                title_texture: self.assets.get_texture("title_image"),
                save_exists: self.save_exists,
                fullscreen: self.fullscreen,
                pointer,
                ui: &virtual_ui,
            }),
            GameScreen::Settings(_) => ui::draw_settings_page(MenuContext {
                title_texture: self.assets.get_texture("title_image"),
                save_exists: self.save_exists,
                fullscreen: self.fullscreen,
                pointer,
                ui: &virtual_ui,
            }),
            GameScreen::Playing | GameScreen::PauseMenu => {
                let paused = matches!(self.screen, GameScreen::PauseMenu);
                let ctx = UiContext {
                    data: &self.data,
                    session: &self.session,
                    sprites: MapSpriteTextures {
                        capital: self.assets.get_texture("capital_keep"),
                        village: self.assets.get_texture("village_cluster"),
                        ruin: self.assets.get_texture("ruin_watchtower"),
                        resource: self.assets.get_texture("crystal_outcrop"),
                        grove: self.assets.get_texture("ancient_grove"),
                        bridge: self.assets.get_texture("river_bridge"),
                        gate: self.assets.get_texture("mountain_gate"),
                        waystone: self.assets.get_texture("road_waystone"),
                        shrine: self.assets.get_texture("marsh_shrine"),
                    },
                    camera_target: self.camera.target,
                    camera_zoom: self.camera.zoom,
                    sprite_showcase: self.capture_sprite_showcase,
                    map_overlay: self.map_overlay,
                    show_chronicle: self.show_chronicle,
                    show_factions: self.show_factions,
                    input_blocked: paused,
                    touch_claimed: self.touch_claimed,
                    pointer,
                    ui: &virtual_ui,
                };

                let mut actions = ui::draw_game_ui(ctx, &mut self.hover_tooltip);
                if paused {
                    actions.extend(ui::draw_pause_menu(PauseMenuContext {
                        save_exists: self.save_exists,
                        pending_exit_warning: self.pending_exit_warning,
                        pointer,
                        ui: &virtual_ui,
                    }));
                }
                actions
            }
        };
        end_virtual_ui_frame();

        for action in actions {
            self.events.push(action);
        }

        self.draw_bottom_message();
    }

    fn draw_bottom_message(&self) {
        let Some(notification) = self.notifications.get_notifications().last() else {
            return;
        };
        let width = (screen_width() - 32.0).clamp(300.0, 620.0);
        let config = NotificationRenderConfig {
            width,
            row_height: 40.0,
            padding: 10.0,
            font_size: 15.0,
            background: Color::new(0.02, 0.03, 0.028, 0.92),
            border_alpha: 0.82,
            ..Default::default()
        };
        draw_notification(
            notification,
            (screen_width() - width) * 0.5,
            screen_height() - 58.0,
            &config,
        );
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => {
                self.session = GameSession::new(&self.data);
                self.show_chronicle = false;
                self.show_factions = false;
                self.screen = GameScreen::Playing;
                self.pending_exit_warning = None;
                self.last_save_at = None;
                self.notifications
                    .info("Started a fresh Realmseed campaign");
            }
            UiAction::ContinueGame => {
                if self.load_game() {
                    self.screen = GameScreen::Playing;
                    self.show_chronicle = false;
                    self.show_factions = false;
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
                    self.show_factions = false;
                    self.screen = GameScreen::Playing;
                }
            }
            UiAction::DeleteSave => self.delete_save(),
            UiAction::SelectSite(site_id) => {
                if self.session.select_site(&self.data, &site_id) {
                    if let Some(site) = self.data.site(&site_id) {
                        self.notifications.info(format!("Selected {}", site.name));
                    }
                }
            }
            UiAction::ScoutSelectedSite => match self.session.scout_selected_site(&self.data) {
                Ok(message) => {
                    self.notifications.success(message);
                }
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::FoundCamp => match self.session.found_selected_camp(&self.data) {
                Ok(message) => {
                    self.notifications.success(message);
                }
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::UpgradeSelectedSettlement => {
                match self.session.upgrade_selected_settlement(&self.data) {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::SetSettlementFocus(focus_id) => {
                match self
                    .session
                    .set_selected_settlement_focus(&self.data, &focus_id)
                {
                    Ok(message) => self.notifications.info(message),
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::BuildOrUpgradeRoute(route_id) => {
                match self.session.build_or_upgrade_route(&self.data, &route_id) {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::CompleteRegionalProject(region_id) => {
                match self
                    .session
                    .complete_regional_project(&self.data, &region_id)
                {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::ResolveEventChoice(choice_id) => {
                match self
                    .session
                    .resolve_pending_event_choice(&self.data, &choice_id)
                {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::DeferEvent => match self.session.defer_pending_event() {
                Ok(message) => self.notifications.warning(message),
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::OpenIndependentTrade => {
                match self
                    .session
                    .open_trade_with_selected_independent(&self.data)
                {
                    Ok(message) => self.notifications.success(message),
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::BeginIndependentIntegration => {
                match self.session.begin_selected_integration(&self.data) {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::SelectAmbition(ambition_id) => {
                match self.session.select_ambition(&self.data, &ambition_id) {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::CompleteProject(project_id) => {
                match self.session.complete_project(&self.data, &project_id) {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::ActivateInstitution(institution_id) => {
                match self
                    .session
                    .activate_institution(&self.data, &institution_id)
                {
                    Ok(message) => {
                        self.notifications.success(message);
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::AdvanceSeason => {
                let report = self.session.advance_season(&self.data);
                self.notifications.info(format!(
                    "{} Year {}",
                    self.session.clock.season.label(),
                    self.session.clock.year
                ));
                if report.food_shortages > 0 {
                    self.notifications.warning(format!(
                        "{} settlement food shortage reported",
                        report.food_shortages
                    ));
                }
                if report.settlements_lost > 0 {
                    self.notifications.danger(format!(
                        "{} settlement lost to neglect",
                        report.settlements_lost
                    ));
                }
                if report.road_warnings > 0 {
                    self.notifications
                        .warning(format!("{} road warning reported", report.road_warnings));
                }
                if report.isolated_settlements > 0 {
                    self.notifications.warning(format!(
                        "{} settlement isolated from the capital network",
                        report.isolated_settlements
                    ));
                }
                if report.unmanaged_strain > 0 {
                    self.notifications
                        .info(format!("Unmanaged strain: {}", report.unmanaged_strain));
                }
                if report.events_triggered > 0 {
                    self.notifications
                        .info(format!("{} event pending", report.events_triggered));
                }
                if report.issues_escalated > 0 {
                    self.notifications.warning(format!(
                        "{} active issue escalated",
                        report.issues_escalated
                    ));
                }
                if report.rival_actions > 0 {
                    self.notifications.info("Rival faction acted");
                }
                if report.independent_requests > 0 {
                    self.notifications.warning(format!(
                        "{} independent request logged",
                        report.independent_requests
                    ));
                }
                if report.wilderness_changes > 0 {
                    self.notifications.warning(format!(
                        "{} wilderness pressure band changed",
                        report.wilderness_changes
                    ));
                }
                if report.campaign_finished {
                    self.notifications
                        .success("The 20-year campaign is complete");
                }
            }
            UiAction::ToggleChronicle => {
                self.show_chronicle = !self.show_chronicle;
                if self.show_chronicle {
                    self.show_factions = false;
                }
            }
            UiAction::ToggleFactionPanel => {
                self.show_factions = !self.show_factions;
                if self.show_factions {
                    self.show_chronicle = false;
                }
            }
            UiAction::EndgameNewGame => self.apply_action(UiAction::NewGame),
            UiAction::EndgameReturnToTitle => {
                self.show_chronicle = false;
                self.show_factions = false;
                self.pending_exit_warning = None;
                self.screen = GameScreen::Title;
            }
            UiAction::SetMapOverlay(overlay) => {
                self.map_overlay = overlay;
            }
            UiAction::ZoomMapIn => self.zoom_map(1.16),
            UiAction::ZoomMapOut => self.zoom_map(0.86),
        }
    }

    fn zoom_map(&mut self, factor: f32) {
        self.camera.zoom = (self.camera.zoom * factor)
            .clamp(self.camera.config.min_zoom, self.camera.config.max_zoom);
    }

    fn save_game(&mut self) -> bool {
        let save = self.session.to_save(&self.data.config.version);
        match save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        ) {
            Ok(()) => {
                self.notifications.success("Saved campaign");
                self.last_save_at = Some(get_time());
                self.refresh_save_state();
                true
            }
            Err(err) => {
                self.notifications.danger(format!("Save failed: {}", err));
                false
            }
        }
    }

    fn load_game(&mut self) -> bool {
        let loaded: Result<SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| migrate_save_value(version, value, &self.data),
        );

        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save, &self.data);
                self.notifications.success("Loaded campaign");
                self.last_save_at = Some(get_time());
                self.refresh_save_state();
                true
            }
            Err(err) => {
                self.notifications.warning(format!("Load failed: {}", err));
                false
            }
        }
    }

    fn delete_save(&mut self) {
        match delete_slot(&self.data.config.game_name, &self.data.config.save_slot) {
            Ok(()) => {
                self.notifications.info("Deleted campaign save");
                self.last_save_at = None;
                self.refresh_save_state();
            }
            Err(err) => self.notifications.danger(format!("Delete failed: {}", err)),
        }
    }

    fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
        self.save_slots = get_save_slots(&self.data.config.game_name);
    }

    fn request_guarded_exit(&mut self, target: ExitWarningTarget) {
        if self.has_recent_save() {
            self.complete_exit_target(target);
        } else {
            self.pending_exit_warning = Some(target);
            self.screen = GameScreen::PauseMenu;
        }
    }

    fn has_recent_save(&self) -> bool {
        self.last_save_at
            .map(|saved_at| get_time() - saved_at <= SAVE_WARNING_WINDOW_SECS)
            .unwrap_or(false)
    }

    fn complete_pending_exit(&mut self) {
        if let Some(target) = self.pending_exit_warning.take() {
            self.complete_exit_target(target);
        }
    }

    fn complete_exit_target(&mut self, target: ExitWarningTarget) {
        self.pending_exit_warning = None;
        match target {
            ExitWarningTarget::Title => {
                self.show_chronicle = false;
                self.show_factions = false;
                self.screen = GameScreen::Title;
            }
            ExitWarningTarget::ExitGame => {
                macroquad::miniquad::window::request_quit();
            }
        }
    }
}
