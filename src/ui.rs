//! Immediate-mode UI for the Realmseed map, site panel, and chronicle.

mod map;
mod panel;
mod routes;

use crate::data::GameData;
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{RectExt, VirtualUi};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    NewGame,
    Save,
    Load,
    DeleteSave,
    SelectSite(String),
    ScoutSelectedSite,
    FoundCamp,
    UpgradeSelectedSettlement,
    SetSettlementFocus(String),
    BuildOrUpgradeRoute(String),
    CompleteRegionalProject(String),
    AdvanceSeason,
    ToggleChronicle,
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a GameSession,
    pub save_exists: bool,
    pub save_slots: &'a [String],
    pub loaded_assets: usize,
    pub camera_target: Vec2,
    pub camera_zoom: f32,
    pub show_chronicle: bool,
    pub ui: &'a VirtualUi,
}

pub fn draw_game_ui(ctx: UiContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let mouse = ctx.ui.mouse_position();
    let input_enabled = !ctx.show_chronicle;

    draw_header(&ctx);
    map::draw_map_panel(&ctx, mouse, input_enabled, &mut actions);
    panel::draw_side_panel(&ctx, mouse, input_enabled, &mut actions);
    draw_footer();

    if ctx.show_chronicle {
        panel::draw_chronicle_overlay(&ctx, mouse, &mut actions);
    }

    actions
}

fn draw_header(ctx: &UiContext<'_>) {
    let rect = Rect::new(18.0, 16.0, LOGICAL_WIDTH - 36.0, 64.0);
    let style = SurfaceStyle::new(Color::new(0.08, 0.085, 0.07, 0.98))
        .with_border(1.0, Color::new(0.52, 0.47, 0.32, 0.75))
        .with_top_highlight(2.0, Color::new(0.74, 0.64, 0.36, 0.65));
    draw_surface(rect, &style);

    draw_text_ex(
        &ctx.data.config.display_name,
        rect.x + 18.0,
        rect.y + 39.0,
        TextStyle::new(30.0, dark::TEXT_BRIGHT).params(),
    );

    let clock = &ctx.session.clock;
    draw_badge(
        Rect::new(rect.right() - 560.0, rect.y + 18.0, 124.0, 28.0),
        &format!("{} action", ctx.session.council_actions_remaining),
        Color::new(0.24, 0.23, 0.16, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(rect.right() - 426.0, rect.y + 18.0, 132.0, 28.0),
        &format!("{} migrants", ctx.session.migrant_pool),
        Color::new(0.20, 0.22, 0.24, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(rect.right() - 284.0, rect.y + 18.0, 122.0, 28.0),
        &format!("{} Y{}", clock.season.label(), clock.year),
        Color::new(0.20, 0.24, 0.18, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(rect.right() - 152.0, rect.y + 18.0, 134.0, 28.0),
        &format!("{} known", ctx.session.known_site_count()),
        Color::new(0.20, 0.28, 0.23, 1.0),
        dark::TEXT,
    );
}

fn draw_footer() {
    let rect = Rect::new(18.0, 632.0, LOGICAL_WIDTH - 36.0, 70.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.058, 0.05, 0.97))
            .with_border(1.0, Color::new(0.52, 0.47, 0.32, 0.45)),
    );
    draw_text_block(
        "Click visible markers to inspect sites, found camps, set settlement focus, and upgrade when requirements are met. Question markers are adjacent unknown sites; select one and scout it. Space advances the season. C opens the chronicle.",
        rect.x + 18.0,
        rect.y + 14.0,
        rect.w - 36.0,
        rect.h - 20.0,
        17.0,
        4.0,
        dark::TEXT_DIM,
    );
}

pub(super) fn virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    mouse: Vec2,
) -> bool {
    let style = ButtonStyle::from_tone(tone);
    let hovered = enabled && rect.contains_point(mouse);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let activated = hovered && is_mouse_button_released(MouseButton::Left);
    let fill = if !enabled {
        style.disabled
    } else if pressed {
        style.pressed
    } else if hovered {
        style.hovered
    } else {
        style.normal
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill).with_border(1.0, style.border),
    );
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + if pressed { 2.0 } else { 0.0 },
        rect.w - 16.0,
        rect.h,
        TextStyle::new(
            16.0,
            if enabled {
                style.text_color
            } else {
                dark::TEXT_DIM
            },
        ),
    );
    activated
}

pub(super) fn color_from_array(color: [f32; 4]) -> Color {
    Color::new(color[0], color[1], color[2], color[3])
}

pub(super) fn map_panel_rect() -> Rect {
    Rect::new(18.0, 96.0, 812.0, 520.0)
}
