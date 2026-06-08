//! High-level game loop, state transitions, and toolkit integration.

use crate::data::GameData;
use crate::state::{migrate_save_value, GameSession, SaveData};
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{Camera2D, Camera2DConfig, CameraBounds};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    delete_slot, get_save_slots, load_from_slot_with_migration, save_to_slot_with_version,
    slot_exists,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    camera: Camera2D,
    events: EventBus<UiAction>,
    save_exists: bool,
    save_slots: Vec<String>,
    show_chronicle: bool,
    show_factions: bool,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Realmseed embedded data failed to load: {}", err);
        });

        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.75, 0.2, 0.8, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let _ = assets.load_asset_pack("assets.zip").await;
        let loaded_assets = assets.load_texture_configs(&data.texture_manifest).await;

        let mut notifications = NotificationManager::new();
        notifications.info(format!(
            "Realmseed campaign data loaded; {} manifest textures available",
            loaded_assets
        ));

        let session = GameSession::new(&data);
        let camera = Camera2D::with_config(
            vec2(0.0, 0.0),
            1.0,
            Camera2DConfig {
                drag_button: Some(MouseButton::Right),
                min_zoom: 0.85,
                max_zoom: 1.95,
                bounds: Some(CameraBounds::new(vec2(-280.0, -190.0), vec2(280.0, 190.0))),
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
            events: EventBus::new(),
            save_exists: false,
            save_slots: Vec::new(),
            show_chronicle: false,
            show_factions: false,
        };
        game.refresh_save_state();
        game
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);

        let input = InputState::capture();
        if input.escape_pressed && self.show_chronicle {
            self.events.push(UiAction::ToggleChronicle);
        }
        if input.space_pressed {
            self.events.push(UiAction::AdvanceSeason);
        }
        if is_key_pressed(KeyCode::C) {
            self.events.push(UiAction::ToggleChronicle);
        }
        if is_key_pressed(KeyCode::E) {
            self.events.push(UiAction::ForceEvent);
        }
        if is_key_pressed(KeyCode::F) {
            self.events.push(UiAction::ToggleFactionPanel);
        }
        if is_key_pressed(KeyCode::S) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(KeyCode::L) {
            self.events.push(UiAction::Load);
        }

        self.camera.update(dt, false);

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let ctx = UiContext {
            data: &self.data,
            session: &self.session,
            save_exists: self.save_exists,
            save_slots: &self.save_slots,
            loaded_assets: self.assets.len(),
            camera_target: self.camera.target,
            camera_zoom: self.camera.zoom,
            show_chronicle: self.show_chronicle,
            show_factions: self.show_factions,
            ui: &virtual_ui,
        };

        let actions = ui::draw_game_ui(ctx);
        end_virtual_ui_frame();

        for action in actions {
            self.events.push(action);
        }

        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                ..Default::default()
            });
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => {
                self.session = GameSession::new(&self.data);
                self.show_chronicle = false;
                self.show_factions = false;
                self.notifications
                    .info("Started a fresh Realmseed campaign");
            }
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::DeleteSave => self.delete_save(),
            UiAction::SelectSite(site_id) => {
                if self.session.select_site(&self.data, &site_id) {
                    if let Some(site) = self.data.site(&site_id) {
                        self.notifications.info(format!("Selected {}", site.name));
                    }
                }
            }
            UiAction::ScoutSelectedSite => {
                let site_name = self
                    .session
                    .selected_site(&self.data)
                    .map(|site| site.name.clone())
                    .unwrap_or_else(|| "site".to_owned());
                if self.session.scout_selected_site(&self.data) {
                    self.notifications.success(format!("Scouted {}", site_name));
                    self.show_chronicle = true;
                } else {
                    self.notifications
                        .warning("Select an adjacent unknown site to scout");
                }
            }
            UiAction::FoundCamp => match self.session.found_selected_camp(&self.data) {
                Ok(message) => {
                    self.notifications.success(message);
                    self.show_chronicle = true;
                }
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::UpgradeSelectedSettlement => {
                match self.session.upgrade_selected_settlement(&self.data) {
                    Ok(message) => {
                        self.notifications.success(message);
                        self.show_chronicle = true;
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
                        self.show_chronicle = true;
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
                        self.show_chronicle = true;
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
                        self.show_chronicle = true;
                    }
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::DeferEvent => match self.session.defer_pending_event() {
                Ok(message) => self.notifications.warning(message),
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::ForceEvent => match self.session.force_next_event(&self.data) {
                Ok(message) => self.notifications.info(message),
                Err(reason) => self.notifications.warning(reason),
            },
            UiAction::OpenIndependentTrade => {
                match self.session.open_trade_with_selected_independent() {
                    Ok(message) => self.notifications.success(message),
                    Err(reason) => self.notifications.warning(reason),
                }
            }
            UiAction::BeginIndependentIntegration => {
                match self.session.begin_selected_integration(&self.data) {
                    Ok(message) => {
                        self.notifications.success(message);
                        self.show_chronicle = true;
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
                    self.show_chronicle = true;
                }
                if report.settlements_lost > 0 {
                    self.notifications.danger(format!(
                        "{} settlement lost to neglect",
                        report.settlements_lost
                    ));
                    self.show_chronicle = true;
                }
                if report.road_warnings > 0 {
                    self.notifications
                        .warning(format!("{} road warning reported", report.road_warnings));
                    self.show_chronicle = true;
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
            }
            UiAction::ToggleChronicle => {
                self.show_chronicle = !self.show_chronicle;
            }
            UiAction::ToggleFactionPanel => {
                self.show_factions = !self.show_factions;
            }
        }
    }

    fn save_game(&mut self) {
        let save = self.session.to_save(&self.data.config.version);
        match save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        ) {
            Ok(()) => {
                self.notifications.success("Saved campaign");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.danger(format!("Save failed: {}", err)),
        }
    }

    fn load_game(&mut self) {
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
                self.refresh_save_state();
            }
            Err(err) => self.notifications.warning(format!("Load failed: {}", err)),
        }
    }

    fn delete_save(&mut self) {
        match delete_slot(&self.data.config.game_name, &self.data.config.save_slot) {
            Ok(()) => {
                self.notifications.info("Deleted campaign save");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.danger(format!("Delete failed: {}", err)),
        }
    }

    fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
        self.save_slots = get_save_slots(&self.data.config.game_name);
    }
}
